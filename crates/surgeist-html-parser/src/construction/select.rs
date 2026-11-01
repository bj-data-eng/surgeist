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
    /// What the last selectedness setting run left true of the list of options,
    /// while only appends of one more option since have changed that list.
    settled: Option<SettledOptions>,
}
impl SelectState {
    pub(super) fn fresh() -> Self {
        Self {
            flags: HtmlSelectFlags::default(),
            settled: None,
        }
    }
}
/// The facts about a single-select's list of options that one more option
/// appended at its end can change, as the selectedness setting algorithm
/// (HTML 877d, "selectedness setting algorithm", step 5) leaves them: the last
/// selected option, which is the only one, whether any option is enabled, and
/// the last option the run counted. An insertion that connects several options
/// runs the full algorithm for the first, which counts the later ones too, so
/// the shortcut declines for an option the summary already ends with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SettledOptions {
    selected: Option<usize>,
    enabled: bool,
    last: Option<usize>,
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
        ("select", Some(SelectMetadata::Select(SelectState::fresh()))),
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
                    .ok_or_else(|| latch_registry_row(c))?
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
                    .ok_or_else(|| latch_registry_row(c))?
                    .child_at(0)
                    .ok_or_else(|| latch_registry_row(c))?
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
                        .ok_or_else(|| latch_registry_row(c))?
                        .child_count()
                {
                    return self.write_current_effect(c, None);
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let child = self
                    .construction
                    .node_at(option)
                    .ok_or_else(|| latch_registry_row(c))?
                    .child_at(cursor)
                    .ok_or_else(|| latch_registry_row(c))?
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
        // The update visits the select's selectedcontent descendants; with no
        // selectedcontent row in the registry there are none to visit.
        if self.payloads.selectedcontent_rows == 0 {
            #[cfg(test)]
            assert!(
                self.uncharged_selectedcontent_descendants(select)
                    .is_empty()
            );
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
            if old == new {
                // The option moved without changing its select, so its place in
                // the list or its disabledness may have changed unobserved.
                if let Some(select) = old {
                    self.unsettle_select(c, select)?;
                }
            } else {
                for select in [old, new].into_iter().flatten() {
                    let appended = if !removal && detached.is_none() && old.is_none() {
                        self.selectedness_after_append(c, select, node)?
                    } else {
                        None
                    };
                    let changed = match appended {
                        Some(changed) => changed,
                        None => self.selectedness_setting(c, select, detached)?,
                    };
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
            .ok_or_else(|| latch_registry_row(c))?
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
            .ok_or_else(|| latch_registry_row(c))?
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
            self.unsettle_select(c, select)?;
            return Ok(false);
        }
        let list = self.retain_work_list(c, select, WorkListKind::Options { detached })?;
        let count = self.work_list_len(c, list)?;
        payload_write::<HtmlSelectCursor>(c, 1)?;
        let mut cursor = HtmlSelectCursor::new(HtmlSelectOperation::Normalize, false, count)
            .map_err(|_| SessionError::WrongToken)?;
        // What the run leaves true of the list, for the settled summary.
        let mut last_selected = None;
        let mut selected_by_fallback = None;
        let mut enabled = false;
        let changed = loop {
            let reply = match cursor.action() {
                HtmlSelectAction::ReadSelectedness { ordinal } => {
                    let node = self
                        .work_list_node(c, list, ordinal)?
                        .ok_or(SessionError::WrongToken)?;
                    let (_, SelectMetadata::Option(state)) = self.select_metadata(c, node)? else {
                        return Err(SessionError::SelectMetadataUnavailable);
                    };
                    if state.flags.selectedness() {
                        last_selected = Some(node);
                    }
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
                    if selected {
                        selected_by_fallback = Some(node);
                    }
                    HtmlSelectReply::Applied
                }
                HtmlSelectAction::ReadDisabled { ordinal } => {
                    let node = self
                        .work_list_node(c, list, ordinal)?
                        .ok_or(SessionError::WrongToken)?;
                    let disabled = self.option_disabled(c, node, detached)?;
                    enabled |= !disabled;
                    HtmlSelectReply::Disabled(disabled)
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
        let last = match count.checked_sub(1) {
            Some(ordinal) => self.work_list_node(c, list, ordinal)?,
            None => None,
        };
        self.release_work_list(c, list)?;
        // The run keeps the last option read selected, and the fallback writes
        // selectedness only when it read none.
        self.settle_select(
            c,
            select,
            SettledOptions {
                selected: selected_by_fallback.or(last_selected),
                enabled,
                last,
            },
        )?;
        Ok(changed)
    }
    fn settle_select(
        &mut self,
        c: &mut ParseControl,
        select: usize,
        settled: SettledOptions,
    ) -> Result<(), SessionError> {
        let (slot, SelectMetadata::Select(mut state)) = self.select_metadata(c, select)? else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        if state.settled != Some(settled) {
            state.settled = Some(settled);
            self.set_select_metadata(c, slot, SelectMetadata::Select(state))?;
        }
        Ok(())
    }
    /// Forgets the settled summary of a select whose list or options changed in
    /// a way only a full selectedness setting run can account for.
    fn unsettle_select(&mut self, c: &mut ParseControl, select: usize) -> Result<(), SessionError> {
        let (slot, SelectMetadata::Select(mut state)) = self.select_metadata(c, select)? else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        if state.settled.is_some() {
            state.settled = None;
            self.set_select_metadata(c, slot, SelectMetadata::Select(state))?;
        }
        Ok(())
    }
    /// Whether no node follows `node` in `root`'s subtree in tree order, apart
    /// from `node`'s own descendants: it ends every ancestor's children up to
    /// `root`.
    pub(super) fn ends_subtree(
        &self,
        c: &mut ParseControl,
        root: usize,
        node: usize,
    ) -> Result<bool, SessionError> {
        let mut node = node;
        for _ in 0..self.construction.storage().len() {
            let Some(parent) = self.ordinary_parent(c, node, None)? else {
                return Ok(false);
            };
            c.charge(WorkCategory::RecordVisit, 2)?;
            let record = self
                .construction
                .node_at(parent)
                .ok_or_else(|| latch_registry_row(c))?;
            let last = record
                .child_count()
                .checked_sub(1)
                .and_then(|last| record.child_at(last))
                .map(|child| child.position());
            if last != Some(node) {
                return Ok(false);
            }
            if parent == root {
                return Ok(true);
            }
            node = parent;
        }
        Err(SessionError::Cycle)
    }
    /// The selectedness setting algorithm's result after `option` was inserted
    /// as the last option of `select`'s list, from the settled summary alone
    /// (HTML 877d, steps 5 to 7: the earlier options keep their selectedness and
    /// disabledness, `option` is the last selected if it is selected, and the
    /// first enabled option matters only while none is selected). None when the
    /// summary or the placement cannot establish that result, so the full
    /// algorithm runs instead.
    pub(super) fn selectedness_after_append(
        &mut self,
        c: &mut ParseControl,
        select: usize,
        option: usize,
    ) -> Result<Option<bool>, SessionError> {
        if self.form_attribute(c, select, "multiple")?.is_some() {
            return Ok(None);
        }
        let (select_slot, SelectMetadata::Select(mut select_state)) =
            self.select_metadata(c, select)?
        else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        let Some(settled) = select_state.settled else {
            return Ok(None);
        };
        // An option's own descendants are not in the list. A summary that ends
        // with this option already counted it, with the options that connected
        // before its hook ran.
        if settled.last == Some(option) || !self.ends_subtree(c, select, option)? {
            return Ok(None);
        }
        let (option_slot, SelectMetadata::Option(mut option_state)) =
            self.select_metadata(c, option)?
        else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        #[cfg(test)]
        let before = self.uncharged_option_list(select)?;
        let mut next = SettledOptions {
            last: Some(option),
            ..settled
        };
        let mut changed = false;
        let disabled = if settled.enabled {
            None
        } else {
            Some(self.option_disabled(c, option, None)?)
        };
        #[cfg(test)]
        let mut size_read = None;
        if option_state.flags.selectedness() {
            if let Some(previous) = settled.selected {
                let (slot, SelectMetadata::Option(mut state)) =
                    self.select_metadata(c, previous)?
                else {
                    return Err(SessionError::SelectMetadataUnavailable);
                };
                state.flags.set_selectedness_for_select(false);
                self.set_select_metadata(c, slot, SelectMetadata::Option(state))?;
                changed = true;
            }
            next.selected = Some(option);
        } else if settled.selected.is_none() {
            let one = match disabled {
                // An earlier option is enabled, so the settled state with a
                // display size of one would have selected it.
                None | Some(false) => self.select_size(c, select)? == 1,
                Some(true) => false,
            };
            #[cfg(test)]
            {
                size_read = (disabled != Some(true)).then_some(one);
            }
            if disabled.is_none() && one {
                return Ok(None);
            }
            if disabled == Some(false) && one {
                option_state.flags.set_selectedness_for_select(true);
                self.set_select_metadata(c, option_slot, SelectMetadata::Option(option_state))?;
                next.selected = Some(option);
                changed = true;
            }
        }
        if let Some(disabled) = disabled {
            next.enabled |= !disabled;
        }
        if select_state.settled != Some(next) {
            select_state.settled = Some(next);
            self.set_select_metadata(c, select_slot, SelectMetadata::Select(select_state))?;
        }
        #[cfg(test)]
        self.assert_matches_full_walk(select, before, size_read, changed);
        Ok(Some(changed))
    }
}

#[cfg(test)]
impl SessionStorage {
    /// Uncharged: `node` is an HTML element named `local`.
    fn uncharged_html_local(&self, node: usize, local: &str) -> bool {
        let Some(record) = self.construction.node_at(node) else {
            return false;
        };
        let HtmlNodeKind::Element(element) = record.kind() else {
            return false;
        };
        element
            .name()
            .namespace()
            .uri()
            .is_some_and(|uri| uri.to_utf8().as_deref() == Ok("http://www.w3.org/1999/xhtml"))
            && element.name().local_name().to_utf8().as_deref() == Ok(local)
    }
    pub(super) fn uncharged_attribute(&self, node: usize, local: &str) -> bool {
        let Some(HtmlNodeKind::Element(element)) =
            self.construction.node_at(node).map(|record| record.kind())
        else {
            return false;
        };
        element.attributes().as_slice().iter().any(|attribute| {
            attribute.name().namespace().uri().is_none()
                && attribute.name().local_name().to_utf8().as_deref() == Ok(local)
        })
    }
    fn uncharged_parent(&self, node: usize) -> Option<usize> {
        self.construction
            .node_at(node)
            .and_then(|record| record.parent())
            .map(|parent| parent.position())
    }
    fn uncharged_selectedness(&self, node: usize) -> bool {
        let slot = self
            .payloads
            .node_joins
            .get(node)
            .and_then(|joins| joins.get(Join::SelectFamily))
            .expect("an option has select metadata");
        match self.payloads.values.get(slot) {
            Some(RetainedPayload::SelectFamily {
                state: SelectMetadata::Option(state),
                ..
            }) => state.flags.selectedness(),
            _ => panic!("an option's select metadata row"),
        }
    }
    /// Uncharged: HTML's option disabledness of `node`.
    fn uncharged_option_disabled(&self, node: usize) -> bool {
        if self.uncharged_attribute(node, "disabled") {
            return true;
        }
        let mut next = self.uncharged_parent(node);
        while let Some(ancestor) = next {
            for kind in [
                KnownHtmlElement::Option,
                KnownHtmlElement::Select,
                KnownHtmlElement::Datalist,
                KnownHtmlElement::Hr,
                KnownHtmlElement::Optgroup,
            ] {
                if self.uncharged_html_local(ancestor, kind.local_name()) {
                    match surgeist_html::bounded::option_disabled_ancestor(Some(kind)) {
                        surgeist_html::bounded::HtmlOptionDisabledAncestor::Enabled => {
                            return false;
                        }
                        surgeist_html::bounded::HtmlOptionDisabledAncestor::ReadOptgroupDisabled => {
                            return self.uncharged_attribute(ancestor, "disabled");
                        }
                        surgeist_html::bounded::HtmlOptionDisabledAncestor::Continue => {}
                    }
                }
            }
            next = self.uncharged_parent(ancestor);
        }
        false
    }
    /// Uncharged: the style elements of the tree at `root` in tree order.
    pub(super) fn uncharged_style_order(&self, root: usize) -> Vec<usize> {
        let mut found = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            let record = self.construction.node_at(node).expect("a retained node");
            if self.uncharged_html_local(node, "style") {
                found.push(node);
            }
            stack.extend(
                (0..record.child_count())
                    .rev()
                    .filter_map(|position| record.child_at(position).map(|child| child.position())),
            );
        }
        found
    }
    /// Uncharged: the selectedcontent elements below `select` in tree order.
    fn uncharged_selectedcontent_descendants(&self, select: usize) -> Vec<usize> {
        let mut found = Vec::new();
        let mut stack = vec![select];
        while let Some(node) = stack.pop() {
            let record = self.construction.node_at(node).expect("a retained node");
            if node != select && self.uncharged_html_local(node, "selectedcontent") {
                found.push(node);
            }
            stack.extend(
                (0..record.child_count())
                    .rev()
                    .filter_map(|position| record.child_at(position).map(|child| child.position())),
            );
        }
        found
    }
    /// Uncharged HTML "get the list of options" of `select` in tree order, with
    /// each option's selectedness and disabledness.
    fn uncharged_option_list(
        &self,
        select: usize,
    ) -> Result<Vec<(usize, bool, bool)>, SessionError> {
        let mut options = Vec::new();
        let mut stack = vec![select];
        let mut roots = true;
        while let Some(node) = stack.pop() {
            let record = self
                .construction
                .node_at(node)
                .ok_or(SessionError::ForeignNode)?;
            let mut descend = roots;
            if !roots {
                descend = if self.uncharged_html_local(node, "option") {
                    options.push((
                        node,
                        self.uncharged_selectedness(node),
                        self.uncharged_option_disabled(node),
                    ));
                    false
                } else if ["select", "hr", "datalist"]
                    .into_iter()
                    .any(|local| self.uncharged_html_local(node, local))
                {
                    false
                } else if self.uncharged_html_local(node, "optgroup") {
                    let mut inside = false;
                    let mut next = self.uncharged_parent(node);
                    while let Some(ancestor) = next.filter(|&ancestor| ancestor != select) {
                        inside |= self.uncharged_html_local(ancestor, "optgroup");
                        next = self.uncharged_parent(ancestor);
                    }
                    !inside
                } else {
                    true
                };
            }
            roots = false;
            if descend {
                for position in (0..record.child_count()).rev() {
                    stack.push(
                        record
                            .child_at(position)
                            .ok_or(SessionError::ForeignNode)?
                            .position(),
                    );
                }
            }
        }
        Ok(options)
    }
    /// Test builds: the state `selectedness_after_append` left equals the state
    /// and result of the full algorithm run on the options as they were before.
    fn assert_matches_full_walk(
        &self,
        select: usize,
        before: Vec<(usize, bool, bool)>,
        size_is_one: Option<bool>,
        changed: bool,
    ) {
        let mut selected: Vec<bool> = before.iter().map(|&(_, selected, _)| selected).collect();
        let (mut expected_changed, mut first_enabled, mut last_selected) = (false, None, None);
        for (index, &(_, _, disabled)) in before.iter().enumerate() {
            if selected[index] {
                if let Some(previous) = last_selected {
                    selected[previous] = false;
                    expected_changed = true;
                }
                last_selected = Some(index);
            }
            if first_enabled.is_none() && !disabled {
                first_enabled = Some(index);
            }
        }
        if last_selected.is_none()
            && let Some(first) = first_enabled
            && size_is_one.expect("the full walk reads the display size")
        {
            selected[first] = true;
            last_selected = Some(first);
            expected_changed = true;
        }
        assert_eq!(changed, expected_changed, "result of the full walk");
        for (&(node, ..), &expected) in before.iter().zip(&selected) {
            assert_eq!(
                self.uncharged_selectedness(node),
                expected,
                "selectedness of option {node} after the full walk"
            );
        }
        let slot = self
            .payloads
            .node_joins
            .get(select)
            .and_then(|joins| joins.get(Join::SelectFamily))
            .expect("a select has select metadata");
        let Some(RetainedPayload::SelectFamily {
            state: SelectMetadata::Select(state),
            ..
        }) = self.payloads.values.get(slot)
        else {
            panic!("a select's select metadata row");
        };
        assert_eq!(
            state.settled,
            Some(SettledOptions {
                selected: last_selected.map(|index| before[index].0),
                enabled: before.iter().any(|&(.., disabled)| !disabled),
                last: before.last().map(|&(node, ..)| node),
            }),
            "settled summary of the full walk"
        );
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

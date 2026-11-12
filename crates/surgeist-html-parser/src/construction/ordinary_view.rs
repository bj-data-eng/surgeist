//! Exact physical traversal shared by form views and output reads.

use super::*;
#[cfg(test)]
use surgeist_html::construction::ConstructionNodeKind;

/// Borrows the same storage before or after publication; it owns no records or identity.
#[derive(Clone, Copy)]
pub(super) enum ConstructionView<'a> {
    Active(&'a surgeist_html::bounded::construction::HtmlConstructionStorage),
    Frozen(&'a surgeist_html::construction::HtmlConstructionResult),
}
impl<'a> ConstructionView<'a> {
    pub(super) fn node_at(
        self,
        position: usize,
    ) -> Option<surgeist_html::bounded::construction::HtmlConstructionNode<'a>> {
        match self {
            Self::Active(value) => value.node_at(position),
            Self::Frozen(value) => value.node_at(position),
        }
    }
    pub(super) fn len(self) -> usize {
        match self {
            Self::Active(value) => value.len(),
            Self::Frozen(value) => value.len(),
        }
    }
}

// Shared exact physical traversal used by form views and output reads.
pub(super) fn ordinary_view_parent(
    records: ConstructionView<'_>,
    control: &mut ParseControl,
    node: usize,
    detached: Option<usize>,
) -> Result<Option<usize>, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let record = records.node_at(node).ok_or(SessionError::ForeignNode)?;
    Ok(if detached == Some(node) {
        None
    } else {
        record.parent().map(|node| node.position())
    })
}

pub(super) fn next_ordinary_view_node(
    records: ConstructionView<'_>,
    control: &mut ParseControl,
    root: usize,
    node: usize,
    detached: Option<usize>,
) -> Result<Option<usize>, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let record = records.node_at(node).ok_or(SessionError::ForeignNode)?;
    for position in 0..record.child_count() {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let child = record
            .child_at(position)
            .ok_or_else(|| latch_registry_row(control))?
            .position();
        if detached != Some(child) || root == child {
            control.note_sibling(node, position);
            return Ok(Some(child));
        }
    }
    next_ordinary_view_after_subtree(records, control, root, node, detached)
}

pub(super) fn next_ordinary_view_after_subtree(
    records: ConstructionView<'_>,
    control: &mut ParseControl,
    root: usize,
    mut node: usize,
    detached: Option<usize>,
) -> Result<Option<usize>, SessionError> {
    for _ in 0..records.len() {
        if node == root {
            return Ok(None);
        }
        let parent = ordinary_view_parent(records, control, node, detached)?
            .ok_or(SessionError::WrongSibling)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let children = records.node_at(parent).ok_or(SessionError::ForeignNode)?;
        let count = children.child_count();
        // The walk that reached `node` usually left its ordinal behind, so the
        // search for it starts there and costs the siblings after it, not the
        // ones before.
        let mut ordinal = None;
        if let Some(hinted) = control.sibling_hint(parent)
            && hinted < count
        {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if children.child_at(hinted).map(|child| child.position()) == Some(node) {
                ordinal = Some(hinted);
            }
        }
        if ordinal.is_none() {
            for position in 0..count {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let child = children
                    .child_at(position)
                    .ok_or_else(|| latch_registry_row(control))?
                    .position();
                if child == node {
                    ordinal = Some(position);
                    break;
                }
            }
        }
        let ordinal = ordinal.ok_or(SessionError::WrongSibling)?;
        for position in ordinal.saturating_add(1)..count {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let child = children
                .child_at(position)
                .ok_or_else(|| latch_registry_row(control))?
                .position();
            if detached != Some(child) || root == child {
                control.note_sibling(parent, position);
                return Ok(Some(child));
            }
        }
        node = parent;
    }
    Err(SessionError::Cycle)
}

#[cfg(test)]
pub(super) fn find_output_state(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<OutputState, SessionError> {
    for slot in 0..payloads.values.len() {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(RetainedPayload::Output {
            node: Some(owner),
            state,
        }) = payloads.values.get(slot)
            && *owner == node
        {
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<OutputState>() as u64,
            )?;
            return Ok(*state);
        }
    }
    Err(SessionError::OutputMetadataUnavailable)
}

#[cfg(test)]
pub(super) fn append_descendant_text(
    records: ConstructionView<'_>,
    control: &mut ParseControl,
    root: usize,
    destination: &mut Utf16Buffer,
) -> Result<(), SessionError> {
    let mut next = next_ordinary_view_node(records, control, root, root, None)?;
    while let Some(node) = next {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let ConstructionNodeKind::Text(value) = records
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        {
            destination.extend_units(
                value
                    .code_units()
                    .map_err(|error| construction_error(control, error))?,
                control,
            )?;
        }
        next = next_ordinary_view_node(records, control, root, node, None)?;
    }
    Ok(())
}

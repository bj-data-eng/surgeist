//! What the record operations read of the records' shape: owners, units, the structural checks of
//! an association and the validation of a publication.

use super::*;
use crate::internal::construction::rows::{
    BuildAssoc, BuildAssocKind, NodeFlags, NodeRow, RowKind,
};

/// The UTF-16 units a name counts: its local name, prefix and namespace URI.
pub(in crate::internal::construction) fn name_units(
    name: &QualifiedName,
) -> Result<usize, HtmlConstructionError> {
    plus(
        plus(
            name.local_name().len(),
            name.prefix().map_or(0, DocumentString::len),
        )?,
        name.namespace().uri().map_or(0, DocumentString::len),
    )
}

/// The UTF-16 units an attribute counts: its name and its value of `value_len` units.
pub(in crate::internal::construction) fn attribute_units(
    name: &QualifiedName,
    value_len: usize,
) -> Result<usize, HtmlConstructionError> {
    plus(name_units(name)?, value_len)
}

/// The record that owns `row` in its tree: its parent, or the host or template that owns the
/// fragment.
pub(in crate::internal::construction::bounded) fn owning_parent(row: &NodeRow) -> Option<NodeIx> {
    row.parent.or_else(|| row.owner())
}

/// The owning record of `index`, when the record exists and has one.
pub(in crate::internal::construction::bounded) fn structural_parent(
    arena: &BuildArena,
    index: NodeIx,
) -> Option<NodeIx> {
    arena.rows.get(index.position()).and_then(owning_parent)
}

/// The contents fragment a template owns, when the template and the fragment agree about it.
pub(in crate::internal::construction::bounded) fn ordinary_owned_contents(
    arena: &BuildArena,
    template: NodeIx,
) -> Result<NodeIx, HtmlConstructionError> {
    let row = arena.row(template)?;
    if row.kind != RowKind::Element {
        return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
    }
    let Some(BuildAssoc {
        contents,
        kind: BuildAssocKind::Owned,
        ..
    }) = arena.template_contents_of(template)
    else {
        return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
    };
    let target = arena.row(contents)?;
    if target.kind != RowKind::Fragment
        || target.shadow().is_some()
        || target.parent.is_some()
        || target.owner() != Some(template)
    {
        return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
    }
    Ok(contents)
}

/// Checks that the record `index` is a fragment with no parent and no owner, shadow contents when
/// `shadow` is set and ordinary contents otherwise.
pub(in crate::internal::construction::bounded) fn unowned_fragment(
    arena: &BuildArena,
    index: NodeIx,
    shadow: bool,
) -> Result<(), HtmlConstructionError> {
    let target = arena.row(index)?;
    if target.kind != RowKind::Fragment
        || target.parent.is_some()
        || target.shadow().is_some() != shadow
    {
        return Err(if shadow {
            HtmlConstructionCause::InvalidContentsReference.error()
        } else {
            HtmlConstructionCause::InvalidTemplateAssociation.error()
        });
    }
    if target.flags.has(NodeFlags::OWNED) {
        return Err(HtmlConstructionCause::AssociationAlreadyOwned.into());
    }
    Ok(())
}

/// Checks that the row at `index` agrees with its parent, its place among its siblings and its
/// association, as far as one row and its neighbours show. The chain of the children of a row is
/// checked by the walk that lists them.
pub(in crate::internal::construction::bounded) fn validate_row(
    arena: &BuildArena,
    index: NodeIx,
) -> Result<(), HtmlConstructionError> {
    let row = arena.row(index)?;
    if let Some(parent) = row.parent {
        let parent_row = arena.row(parent)?;
        if !parent_row.kind.is_container()
            || (parent_row.flags.has(NodeFlags::HAS_ASSOC)
                && arena.template_contents_of(parent).is_some())
        {
            return Err(HtmlConstructionCause::ParentChildConflict.into());
        }
    }
    // A record that cannot hold children lists none. The text kinds keep the room after their
    // units in the slot of the first child, which is no child.
    let holds_slot = row.kind.is_text();
    if !row.kind.is_container()
        && (row.last_child.is_some() || (!holds_slot && row.first_child.is_some()))
    {
        return Err(HtmlConstructionCause::ParentChildConflict.into());
    }
    if row.kind.is_container()
        && row.first_child.is_some()
        && row.flags.has(NodeFlags::HAS_ASSOC)
        && arena.template_contents_of(index).is_some()
    {
        return Err(HtmlConstructionCause::ParentChildConflict.into());
    }
    if row.flags.has(NodeFlags::OWNED) {
        if row.kind != RowKind::Fragment || row.parent.is_some() {
            return Err(HtmlConstructionCause::AssociationAlreadyOwned.into());
        }
        let owner = row
            .owner()
            .ok_or(HtmlConstructionCause::UnknownNode.error())?;
        let target = arena.row(owner)?;
        if row.shadow().is_none() {
            let owned = matches!(
                arena.template_contents_of(owner),
                Some(BuildAssoc {
                    contents,
                    kind: BuildAssocKind::Owned,
                    ..
                }) if contents == index
            );
            if !owned || !target.flags.has(NodeFlags::HAS_ASSOC) {
                return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
            }
        } else if arena.shadow_of(owner) != Some(index) || !target.flags.has(NodeFlags::HAS_ASSOC) {
            return Err(HtmlConstructionCause::InvalidShadowAssociation.into());
        }
    }
    Ok(())
}

/// Checks the entry `entry` of the association table: its owner and contents exist, and the owner
/// and the contents agree about it.
pub(in crate::internal::construction::bounded) fn validate_association(
    arena: &BuildArena,
    entry: &BuildAssoc,
) -> Result<(), HtmlConstructionError> {
    let owner = arena.row(entry.owner)?;
    if owner.kind != RowKind::Element || !owner.flags.has(NodeFlags::HAS_ASSOC) {
        return Err(match entry.kind {
            BuildAssocKind::Shadow => HtmlConstructionCause::InvalidShadowAssociation,
            _ => HtmlConstructionCause::InvalidTemplateAssociation,
        }
        .into());
    }
    let target = arena.row(entry.contents)?;
    match entry.kind {
        BuildAssocKind::Owned | BuildAssocKind::ShadowReference => {
            if target.kind != RowKind::Fragment || target.parent.is_some() {
                return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
            }
            if entry.kind == BuildAssocKind::Owned {
                if target.shadow().is_some() || target.owner() != Some(entry.owner) {
                    return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
                }
            } else if target.shadow().is_none() {
                return Err(HtmlConstructionCause::InvalidContentsReference.into());
            }
        }
        BuildAssocKind::Shadow => {
            if target.kind != RowKind::Fragment
                || target.shadow().is_none()
                || target.parent.is_some()
                || target.owner() != Some(entry.owner)
            {
                return Err(HtmlConstructionCause::InvalidShadowAssociation.into());
            }
        }
    }
    Ok(())
}

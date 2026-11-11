//! What the record operations read of the records' shape: owners, units, the structural checks of
//! an association and of a publication, and the walk of a subtree.

use super::*;
use crate::internal::values::Attribute;

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

/// The UTF-16 units an attribute counts: its name and its value.
pub(in crate::internal::construction) fn attribute_units(
    attribute: &Attribute,
) -> Result<usize, HtmlConstructionError> {
    plus(name_units(attribute.name())?, attribute.value().len())
}

/// The most steps a walk of an arena of `records` can take: each record is entered and left once,
/// and a record names at most three kinds of edge.
pub(in crate::internal::construction::bounded) fn walk_budget(records: usize) -> usize {
    records.saturating_mul(3).saturating_add(3)
}

/// The record that owns `record` in its tree: its parent, or the host or template that owns the
/// fragment.
pub(in crate::internal::construction::bounded) fn owning_parent(record: &Record) -> Option<NodeIx> {
    record.parent.or(match &record.owner {
        Some(HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) => {
            Some(owner.index)
        }
        None => None,
    })
}

/// The owning record of `index`, when the record exists and has one.
pub(in crate::internal::construction::bounded) fn structural_parent(
    arena: &Arena,
    index: NodeIx,
) -> Option<NodeIx> {
    arena.records.get(index.position()).and_then(owning_parent)
}

/// The next record below `node` that its walk has not entered: ordinary children in order, then
/// the shadow contents, then the owned template contents, except `skip`. Advances the node's
/// cursor past every edge it reports or skips, with one write.
pub(in crate::internal::construction::bounded) fn next_edge(
    arena: &mut Arena,
    node: NodeIx,
    skip: Option<NodeIx>,
) -> Result<Option<NodeIx>, HtmlConstructionError> {
    let record = arena
        .records
        .get_mut(node.position())
        .ok_or(HtmlConstructionCause::UnknownNode)?;
    let children = record.children.len();
    let mut cursor = record.walk_cursor;
    let edge = loop {
        let candidate = if cursor < children {
            record.children.get(cursor).copied()
        } else if cursor == children {
            record.shadow.as_ref().map(|shadow| shadow.contents.index)
        } else if cursor == children.saturating_add(1) {
            match record.template_contents {
                Some(TemplateContents::Owned(contents)) if Some(contents) != skip => Some(contents),
                _ => None,
            }
        } else {
            break None;
        };
        cursor = cursor.saturating_add(1);
        if candidate.is_some() {
            break candidate;
        }
    };
    record.walk_cursor = cursor;
    Ok(edge)
}

impl Arena {
    /// The record at `index`, or the unknown-node error.
    fn record_at_index(&self, index: NodeIx) -> Result<&Record, HtmlConstructionError> {
        self.records
            .get(index.position())
            .ok_or(HtmlConstructionCause::UnknownNode.error())
    }

    /// Counts one more record at `depth`. The bucket lives in the record at that index, which
    /// exists whenever a record sits at `depth`; a missing one is the unknown-node defect.
    pub(in crate::internal::construction::bounded) fn populate(
        &mut self,
        depth: usize,
    ) -> Result<(), HtmlConstructionError> {
        let bucket = self
            .records
            .get_mut(depth)
            .ok_or(HtmlConstructionCause::UnknownNode.error())?;
        bucket.population = plus(bucket.population, 1)?;
        Ok(())
    }

    /// Counts one record fewer at `depth`; see [`Self::populate`].
    pub(in crate::internal::construction::bounded) fn unpopulate(
        &mut self,
        depth: usize,
    ) -> Result<(), HtmlConstructionError> {
        let bucket = self
            .records
            .get_mut(depth)
            .ok_or(HtmlConstructionCause::UnknownNode.error())?;
        bucket.population = minus(bucket.population, 1)?;
        Ok(())
    }

    /// Gives the record a new depth and moves it between histogram buckets.
    pub(in crate::internal::construction::bounded) fn place(
        &mut self,
        index: NodeIx,
        depth: usize,
    ) -> Result<(), HtmlConstructionError> {
        let record = self
            .records
            .get_mut(index.position())
            .ok_or(HtmlConstructionCause::UnknownNode.error())?;
        let old = std::mem::replace(&mut record.depth, depth);
        self.unpopulate(old)?;
        self.populate(depth)
    }

    /// Places the subtree rooted at `root` so that the root sits at depth `to`, leaving out the
    /// template contents `skip`. The walk was planned, so an inconsistent arena is the only way it
    /// fails, and it is reported as the defect it is.
    pub(in crate::internal::construction::bounded) fn relocate_subtree(
        &mut self,
        root: NodeIx,
        to: usize,
        skip: Option<NodeIx>,
    ) -> Result<(), HtmlConstructionError> {
        let budget = walk_budget(self.records.len());
        self.records
            .get_mut(root.position())
            .ok_or(HtmlConstructionCause::UnknownNode.error())?
            .walk_cursor = 0;
        self.place(root, to)?;
        let (mut node, mut level) = (root, 0usize);
        for _ in 0..budget {
            match next_edge(self, node, skip)? {
                Some(child) => {
                    self.records
                        .get_mut(child.position())
                        .ok_or(HtmlConstructionCause::UnknownNode.error())?
                        .walk_cursor = 0;
                    level = plus(level, 1)?;
                    self.place(child, plus(to, level)?)?;
                    node = child;
                }
                None if node == root => return Ok(()),
                None => {
                    node = structural_parent(self, node)
                        .ok_or(HtmlConstructionCause::ParentChildConflict.error())?;
                    level = minus(level, 1)?;
                }
            }
        }
        // Every record is entered and left once, so the budget is never spent by a consistent
        // arena.
        Err(HtmlConstructionCause::Cycle.error())
    }
}

/// The contents fragment a template owns, when the template and the fragment agree about it.
pub(in crate::internal::construction::bounded) fn ordinary_owned_contents(
    arena: &Arena,
    template: NodeIx,
) -> Result<NodeIx, HtmlConstructionError> {
    let record = arena.record_at_index(template)?;
    if !matches!(record.kind, NodeContent::Element(_)) {
        return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
    }
    let Some(TemplateContents::Owned(contents)) = record.template_contents else {
        return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
    };
    let target = arena.record_at_index(contents)?;
    if !matches!(target.kind, NodeContent::Fragment)
        || target.shadow_properties.is_some()
        || target.parent.is_some()
        || !matches!(&target.owner, Some(HtmlFragmentOwner::Template(owner)) if owner.index == template && owner.tree == arena.serial)
    {
        return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
    }
    Ok(contents)
}

/// Checks that the record `index` is a fragment with no parent and no owner, shadow contents when
/// `shadow` is set and ordinary contents otherwise.
pub(in crate::internal::construction::bounded) fn unowned_fragment(
    arena: &Arena,
    index: NodeIx,
    shadow: bool,
) -> Result<(), HtmlConstructionError> {
    let target = arena.record_at_index(index)?;
    if !matches!(target.kind, NodeContent::Fragment)
        || target.parent.is_some()
        || target.shadow_properties.is_some() != shadow
    {
        return Err(if shadow {
            HtmlConstructionCause::InvalidContentsReference.error()
        } else {
            HtmlConstructionCause::InvalidTemplateAssociation.error()
        });
    }
    if target.owner.is_some() {
        return Err(HtmlConstructionCause::AssociationAlreadyOwned.into());
    }
    Ok(())
}

/// Checks that the record at `index` agrees with its parent, its children and its associations.
pub(in crate::internal::construction::bounded) fn validate_record(
    arena: &Arena,
    index: NodeIx,
) -> Result<(), HtmlConstructionError> {
    let record = arena.record_at_index(index)?;
    if let Some(parent) = record.parent {
        let parent = arena.record_at_index(parent)?;
        if !matches!(
            parent.kind,
            NodeContent::Document(_) | NodeContent::Fragment | NodeContent::Element(_)
        ) || parent.template_contents.is_some()
        {
            return Err(HtmlConstructionCause::ParentChildConflict.into());
        }
    }
    if !record.children.is_empty()
        && (!matches!(
            record.kind,
            NodeContent::Document(_) | NodeContent::Fragment | NodeContent::Element(_)
        ) || record.template_contents.is_some())
    {
        return Err(HtmlConstructionCause::ParentChildConflict.into());
    }
    if record.shadow_properties.is_some() && !matches!(record.kind, NodeContent::Fragment) {
        return Err(HtmlConstructionCause::InvalidShadowAssociation.into());
    }
    if let Some(contents) = record.template_contents {
        if !matches!(record.kind, NodeContent::Element(_)) {
            return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
        }
        let target = arena.record_at_index(contents.position())?;
        if !matches!(target.kind, NodeContent::Fragment) || target.parent.is_some() {
            return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
        }
        match contents {
            TemplateContents::Owned(_) => {
                if target.shadow_properties.is_some()
                    || !matches!(&target.owner,Some(HtmlFragmentOwner::Template(owner)) if owner.index==index && owner.tree==arena.serial)
                {
                    return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
                }
            }
            TemplateContents::ShadowReference(_) => {
                if target.shadow_properties.is_none() {
                    return Err(HtmlConstructionCause::InvalidContentsReference.into());
                }
            }
        }
    }
    if let Some(shadow) = &record.shadow {
        if !matches!(record.kind, NodeContent::Element(_))
            || shadow.host.index != index
            || shadow.host.tree != arena.serial
            || shadow.contents.tree != arena.serial
        {
            return Err(HtmlConstructionCause::InvalidShadowAssociation.into());
        }
        let target = arena.record_at_index(shadow.contents.index)?;
        if target.shadow_properties.is_none()
            || target.parent.is_some()
            || !matches!(&target.owner,Some(HtmlFragmentOwner::ShadowHost(owner)) if *owner==shadow.host)
        {
            return Err(HtmlConstructionCause::InvalidShadowAssociation.into());
        }
    }
    if let Some(owner) = &record.owner {
        if !matches!(record.kind, NodeContent::Fragment) || record.parent.is_some() {
            return Err(HtmlConstructionCause::AssociationAlreadyOwned.into());
        }
        let id = match owner {
            HtmlFragmentOwner::Template(id) | HtmlFragmentOwner::ShadowHost(id) => id,
        };
        if id.tree != arena.serial {
            return Err(HtmlConstructionCause::ForeignBuilder.into());
        }
        let target = arena.record_at_index(id.index)?;
        match owner {
            HtmlFragmentOwner::Template(_) => {
                if target.template_contents != Some(TemplateContents::Owned(index))
                    || record.shadow_properties.is_some()
                {
                    return Err(HtmlConstructionCause::InvalidTemplateAssociation.into());
                }
            }
            HtmlFragmentOwner::ShadowHost(_) => {
                if !matches!(&target.shadow,Some(shadow) if shadow.contents.index==index)
                    || record.shadow_properties.is_none()
                {
                    return Err(HtmlConstructionCause::InvalidShadowAssociation.into());
                }
            }
        }
    }
    Ok(())
}

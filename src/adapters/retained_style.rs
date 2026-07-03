//! Adapter from retained document snapshots and changes into style facts.

use crate::{retained, style};

pub struct RetainedStyleTree<'a> {
    snapshot: retained::Snapshot<'a>,
}

impl<'a> RetainedStyleTree<'a> {
    #[must_use]
    pub const fn new(snapshot: retained::Snapshot<'a>) -> Self {
        Self { snapshot }
    }
}

impl style::Tree for RetainedStyleTree<'_> {
    type Id = retained::Id;

    fn version_hint(&self) -> Option<u64> {
        Some(self.snapshot.revision().get())
    }

    fn node(&self, id: Self::Id) -> style::Result<style::Node<Self::Id>> {
        let node = self.snapshot.get(id).ok_or_else(|| {
            style::Error::new(
                style::ErrorCode::MissingNode,
                format!("missing node {id:?}"),
            )
        })?;
        Ok(style::Node {
            id,
            tag: tag_for_kind(node.kind())?,
            key: node.key().map(style_key_from_retained).transpose()?,
            classes: node
                .classes()
                .iter()
                .map(style_class_from_retained)
                .collect::<style::Result<Vec<_>>>()?,
            attributes: node
                .attributes()
                .iter()
                .map(style_attribute_from_retained)
                .collect::<style::Result<Vec<_>>>()?,
            role: style_role_from_retained(node.role())?,
            state: style_state_from_retained(node.state()),
            text: matches!(node.kind(), retained::Kind::Text),
        })
    }

    fn parent(&self, id: Self::Id, traversal: style::Traversal) -> style::Result<Option<Self::Id>> {
        let node = self.snapshot.get(id).ok_or_else(|| {
            style::Error::new(
                style::ErrorCode::MissingNode,
                format!("missing node {id:?}"),
            )
        })?;
        Ok(match traversal {
            style::Traversal::Canonical => node.parent(),
            style::Traversal::Projected => node.projected_parent().or_else(|| node.parent()),
        })
    }

    fn children(
        &self,
        id: Self::Id,
        traversal: style::Traversal,
    ) -> style::Result<impl Iterator<Item = Self::Id> + '_> {
        let children: Vec<_> = match traversal {
            style::Traversal::Canonical => self
                .snapshot
                .children(id)
                .map_err(map_retained_error)?
                .collect::<Vec<_>>(),
            style::Traversal::Projected => self
                .snapshot
                .projected_children(retained::ProjectionSlot::default(id))
                .map_err(map_retained_error)?
                .collect::<Vec<_>>(),
        };
        Ok(children.into_iter())
    }

    fn previous_sibling(
        &self,
        id: Self::Id,
        traversal: style::Traversal,
    ) -> style::Result<Option<Self::Id>> {
        let Some(parent) = self.parent(id, traversal)? else {
            return Ok(None);
        };
        let siblings: Vec<_> = style::Tree::children(self, parent, traversal)?.collect();
        Ok(siblings
            .iter()
            .position(|sibling| *sibling == id)
            .and_then(|index| index.checked_sub(1))
            .map(|index| siblings[index]))
    }
}

#[must_use]
pub fn style_change_from_retained_flags(flags: retained::ChangeFlags) -> style::Change {
    let mut change = style::Change::empty();
    if !flags.is_empty() {
        change.scope.include_node();
    }
    if flags.has_structure()
        || flags.has_kind()
        || flags.has_classes()
        || flags.has_attributes()
        || flags.has_state()
        || flags.has_focus()
        || flags.has_projection()
    {
        change.rematch = true;
    }
    if flags.has_structure() || flags.has_projection() || flags.has_presence() {
        change.scope.include_siblings();
        change.scope.include_descendants();
        change.invalidation.layout = true;
        change.invalidation.paint = true;
    }
    if flags.has_kind()
        || flags.has_classes()
        || flags.has_attributes()
        || flags.has_state()
        || flags.has_focus()
    {
        change.scope.include_descendants();
    }
    if flags.has_text() || flags.has_label() {
        change.invalidation.layout = true;
        change.invalidation.text = true;
        change.invalidation.paint = true;
    }
    if flags.has_state() || flags.has_focus() || flags.has_pointer_capture() {
        change.invalidation.paint = true;
    }
    change
}

pub fn clear_style_cache_for_retained_changes(
    resolver: &mut style::Resolver,
    changes: &retained::ChangeSet,
) {
    if !changes.inserted().is_empty()
        || !changes.removed().is_empty()
        || !changes.moved().is_empty()
        || !changes.changed_projection_slots().is_empty()
    {
        resolver.clear_cache();
        return;
    }
    let mut local_nodes = Vec::new();
    for (id, flags) in changes.changed() {
        let change = style_change_from_retained_flags(flags);
        if change.scope.siblings || change.scope.descendants {
            resolver.clear_cache();
            return;
        }
        local_nodes.push(id);
    }
    for id in local_nodes {
        resolver.clear_cache_for_node(id);
    }
}

fn tag_for_kind(kind: &retained::Kind) -> style::Result<Option<style::StyleTag>> {
    Ok(match kind {
        retained::Kind::Element(tag) | retained::Kind::Slot(tag) | retained::Kind::Widget(tag) => {
            Some(style_tag_from_retained(tag)?)
        }
        retained::Kind::Root
        | retained::Kind::Text
        | retained::Kind::Canvas
        | retained::Kind::Fragment => None,
        _ => None,
    })
}

fn style_tag_from_retained(tag: &retained::Tag) -> style::Result<style::StyleTag> {
    style::StyleTag::new(tag.as_str()).map_err(map_style_identity_error)
}

fn style_key_from_retained(key: &retained::Key) -> style::Result<style::StyleKey> {
    style::StyleKey::new(key.as_str()).map_err(map_style_identity_error)
}

fn style_class_from_retained(class: &retained::Class) -> style::Result<style::StyleClass> {
    style::StyleClass::new(class.as_str()).map_err(map_style_identity_error)
}

fn style_attribute_from_retained(
    attribute: &retained::Attribute,
) -> style::Result<style::StyleAttribute> {
    Ok(style::StyleAttribute::new(
        style::StyleAttributeName::new(attribute.name.as_str())
            .map_err(map_style_identity_error)?,
        style::StyleAttributeValue::new(attribute.value.as_str())
            .map_err(map_style_identity_error)?,
    ))
}

fn style_role_from_retained(role: retained::Role) -> style::Result<style::StyleRole> {
    Ok(match role {
        retained::Role::Generic => style::StyleRole::Generic,
        retained::Role::Application => style::StyleRole::Application,
        retained::Role::Button => style::StyleRole::Button,
        retained::Role::Text => style::StyleRole::Text,
        retained::Role::List => style::StyleRole::List,
        retained::Role::ListItem => style::StyleRole::ListItem,
        retained::Role::Checkbox => style::StyleRole::Checkbox,
        retained::Role::Textbox => style::StyleRole::Textbox,
        retained::Role::Image => style::StyleRole::Image,
        retained::Role::Canvas => style::StyleRole::Canvas,
        retained::Role::Widget => style::StyleRole::Widget,
        retained::Role::Custom(tag) => style::StyleRole::Custom(style_tag_from_retained(&tag)?),
        _ => style::StyleRole::Generic,
    })
}

fn style_state_from_retained(state: &retained::State) -> style::StyleState {
    style::StyleState::default()
        .with_disabled(state.disabled())
        .with_hovered(state.hovered())
        .with_active(state.active())
        .with_focused(state.focused())
        .with_focus_within(state.focus_within())
        .with_pointer_captured(state.pointer_captured())
        .with_selected(state.selected())
        .with_pressed(state.pressed())
        .with_checked(state.checked())
        .with_expanded(state.expanded())
}

fn map_style_identity_error(error: style::Error) -> style::Error {
    style::Error::new(style::ErrorCode::InvalidString, error.message().to_owned())
}

fn map_retained_error(error: retained::Error) -> style::Error {
    style::Error::new(style::ErrorCode::Traversal, error.to_string())
}

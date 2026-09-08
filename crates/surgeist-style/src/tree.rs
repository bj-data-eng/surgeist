use super::Result;
use crate::{StateFlag, StyleAttribute, StyleClass, StyleKey, StyleRole, StyleState, StyleTag};
use std::hash::Hash;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Traversal {
    Canonical,
    Projected,
}

pub trait Tree {
    type Id: Copy + Eq + Hash;

    fn version_hint(&self) -> Option<u64>;
    fn node(&self, id: Self::Id) -> Result<Node<Self::Id>>;
    fn parent(&self, id: Self::Id, traversal: Traversal) -> Result<Option<Self::Id>>;
    fn children(
        &self,
        id: Self::Id,
        traversal: Traversal,
    ) -> Result<impl Iterator<Item = Self::Id> + '_>;
    fn previous_sibling(&self, id: Self::Id, traversal: Traversal) -> Result<Option<Self::Id>>;
}

#[derive(Clone, Debug)]
pub struct Node<Id> {
    pub id: Id,
    pub tag: Option<StyleTag>,
    pub key: Option<StyleKey>,
    pub classes: Vec<StyleClass>,
    pub attributes: Vec<StyleAttribute>,
    pub role: StyleRole,
    pub state: StyleState,
    pub text: bool,
}

impl<Id> Node<Id> {
    #[must_use]
    pub fn has_state(&self, flag: StateFlag) -> bool {
        self.state.has_flag(flag)
    }
}

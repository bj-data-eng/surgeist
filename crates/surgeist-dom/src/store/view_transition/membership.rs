//! Exact inverses belong to one transition. Leaves name actual records, so a
//! lookup borrows the record's decoded CSS name without another name occurrence.
use super::super::transaction::exact_forest::{self, Forest, KeyView, Suffix};
use super::storage::RecordKey;
use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TransitionMembership {
    forest: Forest<RecordKey>,
    names: exact_forest::Root,
    elements: exact_forest::Root,
}

#[derive(Clone, Default)]
pub(super) struct MembershipCustody {
    forest: exact_forest::Custody<RecordKey>,
}

#[derive(Clone, Default)]
pub(super) struct OwnedMembership {
    pub value: TransitionMembership,
    pub custody: MembershipCustody,
}

fn name_key(name: &surgeist_css::CssViewTransitionIdent) -> KeyView<'_, 0> {
    KeyView {
        words: [],
        used: 0,
        suffix: Suffix::Utf8(name.as_str().as_bytes()),
    }
}
fn element_key(node: NodeId) -> KeyView<'static, 3> {
    KeyView {
        words: [0, node.slot as u64, node.generation],
        used: 3,
        suffix: Suffix::None,
    }
}

impl TransitionMembership {
    pub(in crate::store) fn name_owner<'a>(
        &self,
        name: &surgeist_css::CssViewTransitionIdent,
        mut records: impl FnMut(RecordKey) -> Result<&'a surgeist_css::CssViewTransitionIdent, DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<RecordKey>, DomError> {
        self.forest.lookup(
            self.names,
            &name_key(name),
            |key| Ok(name_key(records(key)?)),
            step,
        )
    }
    pub(in crate::store) fn element_owner(
        &self,
        node: NodeId,
        mut records: impl FnMut(RecordKey) -> Result<NodeId, DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<RecordKey>, DomError> {
        let owner = self.forest.lookup(
            self.elements,
            &element_key(node),
            |key| Ok(element_key(records(key)?)),
            step,
        )?;
        // Routing words omit the owner because one transition owns the map.
        // Qualify the complete stored identity before returning a claim.
        Ok(match owner {
            Some(key) if records(key)? == node => Some(key),
            _ => None,
        })
    }
    pub(super) fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<MembershipCustody, DomError> {
        Ok(MembershipCustody {
            forest: self.forest.adopt_custody(registry, admit, step)?,
        })
    }
    pub(super) fn heap_usage(&self) -> Result<custody::Usage, DomError> {
        self.forest.heap_usage()
    }
    #[cfg(test)]
    pub(in crate::store) fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        self.forest.inspect_graph()
    }
    #[cfg(test)]
    pub(in crate::store) fn inspect_entries(&self) -> (Vec<RecordKey>, Vec<RecordKey>) {
        fn visit(
            forest: &Forest<RecordKey>,
            index: usize,
            parent_bit: Option<usize>,
            seen: &mut std::collections::BTreeSet<usize>,
            result: &mut Vec<RecordKey>,
        ) -> std::collections::BTreeSet<usize> {
            assert!(
                seen.insert(index),
                "acyclic, disjoint exact membership roots"
            );
            match forest.node(index).expect("raw membership edge") {
                exact_forest::Node::Branch {
                    bit,
                    children,
                    representative,
                } => {
                    assert!(parent_bit.is_none_or(|parent| parent < bit));
                    let mut leaves = visit(forest, children[0], Some(bit), seen, result);
                    leaves.extend(visit(forest, children[1], Some(bit), seen, result));
                    assert!(
                        leaves.contains(&representative),
                        "representative is an actual leaf in this subtree"
                    );
                    leaves
                }
                exact_forest::Node::Leaf(record) => {
                    result.push(record);
                    std::collections::BTreeSet::from([index])
                }
            }
        }
        fn leaves(
            forest: &Forest<RecordKey>,
            root: exact_forest::Root,
            seen: &mut std::collections::BTreeSet<usize>,
        ) -> Vec<RecordKey> {
            let mut result = Vec::new();
            if let Some(root) = root {
                visit(forest, root, None, seen, &mut result);
            }
            result
        }
        let mut seen = std::collections::BTreeSet::new();
        let names = leaves(&self.forest, self.names, &mut seen);
        let elements = leaves(&self.forest, self.elements, &mut seen);
        assert_eq!(
            seen.len(),
            self.forest.inspect_summaries().0.occurrences,
            "no orphan membership nodes"
        );
        (names, elements)
    }
}

impl OwnedMembership {
    fn apply(
        &mut self,
        change: exact_forest::Change<RecordKey>,
        names: bool,
        registry: &bank::CustodyRegistry,
    ) {
        let old = std::mem::replace(&mut self.value.forest, change.forest);
        let old_custody = std::mem::replace(&mut self.custody.forest, change.custody);
        if names {
            self.value.names = change.root;
        } else {
            self.value.elements = change.root;
        }
        drop(old);
        drop(old_custody);
        registry.reconcile_releases();
    }
    pub(super) fn replace_name<'a>(
        &mut self,
        name: &surgeist_css::CssViewTransitionIdent,
        expected: Option<RecordKey>,
        next: RecordKey,
        mut records: impl FnMut(RecordKey) -> Result<&'a surgeist_css::CssViewTransitionIdent, DomError>,
        edit: exact_forest::Edit<'_>,
    ) -> Result<(), DomError> {
        let registry = edit.registry;
        let change = self.value.forest.change(
            &self.custody.forest,
            self.value.names,
            &name_key(name),
            Some(next),
            |key| Ok(name_key(records(key)?)),
            edit,
        )?;
        if change.previous != expected {
            return Err(DomError::DuplicateViewTransitionName);
        }
        self.apply(change, true, registry);
        Ok(())
    }
    pub(super) fn replace_element(
        &mut self,
        node: NodeId,
        expected: Option<RecordKey>,
        next: Option<RecordKey>,
        mut records: impl FnMut(RecordKey) -> Result<NodeId, DomError>,
        edit: exact_forest::Edit<'_>,
    ) -> Result<(), DomError> {
        let registry = edit.registry;
        let change = self.value.forest.change(
            &self.custody.forest,
            self.value.elements,
            &element_key(node),
            next,
            |key| Ok(element_key(records(key)?)),
            edit,
        )?;
        if change.previous != expected {
            return Err(DomError::InvalidViewTransitionState);
        }
        self.apply(change, false, registry);
        Ok(())
    }
}

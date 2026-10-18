//! Paired Patricia storage for exact, borrowed domain keys. Consumers own the
//! qualified representatives, key shape, roots and any semantic ordering.
use super::super::{DomError, bank, custody};
use std::mem::size_of;
#[cfg(test)]
mod tests;

pub(in crate::store) type Root = Option<usize>;

#[derive(Clone, Copy)]
pub(in crate::store) enum Suffix<'a> {
    None,
    Utf8(&'a [u8]),
    Utf16(&'a [u16]),
}
impl Suffix<'_> {
    fn shape(self) -> usize {
        match self {
            Self::None => 0,
            Self::Utf8(_) => 9,
            Self::Utf16(_) => 17,
        }
    }
    fn len(self) -> usize {
        match self {
            Self::None => 0,
            Self::Utf8(value) => value.len(),
            Self::Utf16(value) => value.len(),
        }
    }
    fn symbol(self, position: usize) -> u32 {
        match self {
            Self::None => 0,
            Self::Utf8(value) => value.get(position).map_or(0, |value| u32::from(*value) + 1),
            Self::Utf16(value) => value.get(position).map_or(0, |value| u32::from(*value) + 1),
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::store) struct KeyView<'a, const W: usize> {
    pub words: [u64; W],
    pub used: usize,
    pub suffix: Suffix<'a>,
}
impl<const W: usize> KeyView<'_, W> {
    fn width(&self) -> Result<usize, DomError> {
        if self.used > W {
            return Err(DomError::RepresentationExhausted);
        }
        let fixed = self
            .used
            .checked_mul(64)
            .ok_or(DomError::RepresentationExhausted)?;
        self.suffix
            .len()
            .checked_add(1)
            .and_then(|len| len.checked_mul(self.suffix.shape()))
            .and_then(|suffix| fixed.checked_add(suffix))
            .ok_or(DomError::RepresentationExhausted)
    }
    fn bit(&self, bit: usize) -> Result<usize, DomError> {
        if self.used > W {
            return Err(DomError::RepresentationExhausted);
        }
        let fixed = self
            .used
            .checked_mul(64)
            .ok_or(DomError::RepresentationExhausted)?;
        if bit < fixed {
            return Ok(((self.words[bit / 64] >> (63 - bit % 64)) & 1) as usize);
        }
        let width = self.suffix.shape();
        if width == 0 {
            return Ok(0);
        }
        let position = bit - fixed;
        Ok(((self.suffix.symbol(position / width) >> (width - 1 - position % width)) & 1) as usize)
    }
    fn difference(
        &self,
        other: &KeyView<'_, W>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<usize>, DomError> {
        if self.used != other.used || self.used > W || self.suffix.shape() != other.suffix.shape() {
            return Err(DomError::RepresentationExhausted);
        }
        for index in 0..self.used {
            step()?;
            let different = self.words[index] ^ other.words[index];
            if different != 0 {
                return Ok(Some(
                    index
                        .checked_mul(64)
                        .and_then(|offset| offset.checked_add(different.leading_zeros() as usize))
                        .ok_or(DomError::RepresentationExhausted)?,
                ));
            }
        }
        let width = self.suffix.shape();
        if width == 0 {
            return Ok(None);
        }
        for position in 0..=self.suffix.len().min(other.suffix.len()) {
            step()?;
            let different = self.suffix.symbol(position) ^ other.suffix.symbol(position);
            if different != 0 {
                let offset = position
                    .checked_mul(width)
                    .and_then(|v| v.checked_add(different.leading_zeros() as usize - (32 - width)))
                    .and_then(|v| {
                        self.used
                            .checked_mul(64)
                            .and_then(|fixed| fixed.checked_add(v))
                    })
                    .ok_or(DomError::RepresentationExhausted)?;
                return Ok(Some(offset));
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum Node<L> {
    Branch {
        bit: usize,
        children: [usize; 2],
        representative: usize,
    },
    Leaf(L),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::store) struct Forest<L: Copy + Eq> {
    nodes: bank::Bank<Node<L>>,
}
impl<L: Copy + Eq> Default for Forest<L> {
    fn default() -> Self {
        Self {
            nodes: bank::Bank::default(),
        }
    }
}
#[derive(Clone)]
pub(in crate::store) struct Custody<L: Copy + Eq> {
    nodes: bank::BankCustody<Node<L>>,
}
impl<L: Copy + Eq> Default for Custody<L> {
    fn default() -> Self {
        Self {
            nodes: bank::BankCustody::default(),
        }
    }
}

pub(in crate::store) struct Edit<'a> {
    pub registry: &'a bank::CustodyRegistry,
    pub admit: &'a mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    pub step: &'a mut dyn FnMut() -> Result<(), DomError>,
}

pub(in crate::store) struct Change<L: Copy + Eq> {
    pub forest: Forest<L>,
    pub custody: Custody<L>,
    pub root: Root,
    pub previous: Option<L>,
}
struct Working<L: Copy + Eq> {
    forest: Forest<L>,
    custody: Custody<L>,
}
impl<L: Copy + Eq> Drop for Working<L> {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.forest));
        drop(std::mem::take(&mut self.custody));
    }
}

impl<L: Copy + Eq> Forest<L> {
    pub fn node(&self, index: usize) -> Option<Node<L>> {
        self.nodes.get(index).copied()
    }
    fn leaf<const W: usize>(
        &self,
        mut root: usize,
        key: &KeyView<'_, W>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<(usize, L), DomError> {
        let width = key.width()?;
        loop {
            step()?;
            match self.node(root).expect("live exact-key trie path") {
                Node::Branch {
                    bit,
                    children,
                    representative,
                } => {
                    if bit >= width {
                        // All leaves below share the complete queried prefix.
                        // Their next differing bit cannot belong to this short
                        // key. One retained representative proves absence.
                        step()?;
                        let Node::Leaf(value) = self
                            .node(representative)
                            .expect("live exact-key representative")
                        else {
                            unreachable!()
                        };
                        return Ok((representative, value));
                    }
                    root = children[key.bit(bit)?];
                }
                Node::Leaf(value) => return Ok((root, value)),
            }
        }
    }
    pub fn lookup<'a, const W: usize>(
        &self,
        root: Root,
        key: &KeyView<'_, W>,
        mut resolve: impl FnMut(L) -> Result<KeyView<'a, W>, DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<L>, DomError> {
        let Some(root) = root else { return Ok(None) };
        let (_, value) = self.leaf(root, key, step)?;
        Ok(key
            .difference(&resolve(value)?, step)?
            .is_none()
            .then_some(value))
    }
    pub fn first(
        &self,
        mut root: Root,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<L>, DomError> {
        while let Some(index) = root {
            step()?;
            match self.node(index).expect("live exact-key first path") {
                Node::Branch { children, .. } => root = Some(children[0]),
                Node::Leaf(value) => return Ok(Some(value)),
            }
        }
        Ok(None)
    }
    pub fn after<const W: usize>(
        &self,
        root: Root,
        key: &KeyView<'_, W>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<L>, DomError> {
        let mut current = root;
        let mut successor = None;
        while let Some(index) = current {
            step()?;
            match self.node(index).expect("live exact-key successor path") {
                Node::Branch { bit, children, .. } => {
                    if bit >= key.width()? {
                        return Err(DomError::RepresentationExhausted);
                    }
                    let side = key.bit(bit)?;
                    if side == 0 {
                        successor = Some(children[1]);
                    }
                    current = Some(children[side]);
                }
                Node::Leaf(_) => break,
            }
        }
        self.first(successor, step)
    }
    pub fn change<'a, const W: usize>(
        &self,
        custody: &Custody<L>,
        mut root: Root,
        key: &KeyView<'_, W>,
        value: Option<L>,
        mut resolve: impl FnMut(L) -> Result<KeyView<'a, W>, DomError>,
        mut edit: Edit<'_>,
    ) -> Result<Change<L>, DomError> {
        let mut next = Working {
            forest: self.clone(),
            custody: custody.clone(),
        };
        let previous = if let Some(original) = root {
            let (index, old) = self.leaf(original, key, edit.step)?;
            let different = key.difference(&resolve(old)?, edit.step)?;
            match different {
                None => {
                    if let Some(value) = value {
                        if key.difference(&resolve(value)?, edit.step)?.is_some() {
                            return Err(DomError::RepresentationExhausted);
                        }
                        if value != old {
                            next.write(index, Node::Leaf(value), &mut edit)?;
                        }
                    } else {
                        next.delete(&mut root, key, &mut edit)?;
                    }
                    Some(old)
                }
                Some(bit) => {
                    if let Some(value) = value {
                        if key.difference(&resolve(value)?, edit.step)?.is_some() {
                            return Err(DomError::RepresentationExhausted);
                        }
                        next.insert(&mut root, key, bit, value, &mut edit)?;
                    }
                    None
                }
            }
        } else {
            if let Some(value) = value {
                if key.difference(&resolve(value)?, edit.step)?.is_some() {
                    return Err(DomError::RepresentationExhausted);
                }
                root = Some(next.allocate(Node::Leaf(value), &mut edit)?);
            }
            None
        };
        Ok(Change {
            forest: std::mem::take(&mut next.forest),
            custody: std::mem::take(&mut next.custody),
            root,
            previous,
        })
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Custody<L>, DomError> {
        let nodes = self.nodes.adopt_controlled(
            registry,
            |_| {
                step()?;
                Ok(custody::Usage::new(size_of::<Node<L>>(), 0))
            },
            admit,
        )?;
        Ok(Custody { nodes })
    }
    pub fn heap_usage(&self) -> Result<custody::Usage, DomError> {
        custody::Usage::new(self.nodes.index_charge().bytes, 0).checked_add(custody::Usage::new(
            self.nodes.totals().bytes,
            self.nodes.totals().units,
        ))
    }
    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        self.nodes.inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<Node<L>>(),
                units: 0,
                occurrences: 1,
            })
        })
    }
    #[cfg(test)]
    pub fn inspect_summaries(&self) -> (bank::Totals, bank::IndexCharge) {
        (self.nodes.totals(), self.nodes.index_charge())
    }
}

impl<L: Copy + Eq> Working<L> {
    fn write(&mut self, index: usize, value: Node<L>, edit: &mut Edit<'_>) -> Result<(), DomError> {
        (edit.step)()?;
        let record = bank::ControlledRecord::new(
            value,
            custody::Usage::new(size_of::<Node<L>>(), 0),
            edit.registry,
            &mut *edit.admit,
        )?;
        let (nodes, control) = self.forest.nodes.set_controlled(
            &self.custody.nodes,
            index,
            record,
            false,
            bank::Totals {
                bytes: size_of::<Node<L>>(),
                units: 0,
                occurrences: 1,
            },
            edit.registry,
            &mut *edit.admit,
        )?;
        let old = Working {
            forest: Forest {
                nodes: std::mem::replace(&mut self.forest.nodes, nodes),
            },
            custody: Custody {
                nodes: std::mem::replace(&mut self.custody.nodes, control),
            },
        };
        drop(old);
        edit.registry.reconcile_releases();
        Ok(())
    }
    fn allocate(&mut self, value: Node<L>, edit: &mut Edit<'_>) -> Result<usize, DomError> {
        let index = self
            .forest
            .nodes
            .lowest_vacant()
            .unwrap_or(self.forest.nodes.len());
        self.write(index, value, edit)?;
        Ok(index)
    }
    fn remove(&mut self, index: usize, edit: &mut Edit<'_>) -> Result<(), DomError> {
        (edit.step)()?;
        let (nodes, control, removed) = self.forest.nodes.remove_controlled(
            &self.custody.nodes,
            index,
            edit.registry,
            &mut *edit.admit,
        )?;
        let old = Working {
            forest: Forest {
                nodes: std::mem::replace(&mut self.forest.nodes, nodes),
            },
            custody: Custody {
                nodes: std::mem::replace(&mut self.custody.nodes, control),
            },
        };
        drop(old);
        drop(removed);
        edit.registry.reconcile_releases();
        Ok(())
    }
    fn link(
        &mut self,
        parent: Option<(usize, usize)>,
        child: usize,
        root: &mut Root,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        if let Some((index, side)) = parent {
            let Node::Branch {
                bit,
                mut children,
                representative,
            } = self.forest.node(index).expect("exact-key parent")
            else {
                unreachable!()
            };
            children[side] = child;
            self.write(
                index,
                Node::Branch {
                    bit,
                    children,
                    representative,
                },
                edit,
            )
        } else {
            *root = Some(child);
            Ok(())
        }
    }
    fn insert<const W: usize>(
        &mut self,
        root: &mut Root,
        key: &KeyView<'_, W>,
        bit: usize,
        value: L,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        let mut current = root.expect("existing exact-key root");
        let mut parent = None;
        loop {
            (edit.step)()?;
            match self.forest.node(current).expect("exact-key insertion path") {
                Node::Branch {
                    bit: previous,
                    children,
                    ..
                } if previous < bit => {
                    let side = key.bit(previous)?;
                    parent = Some((current, side));
                    current = children[side];
                }
                _ => break,
            }
        }
        let side = key.bit(bit)?;
        let leaf = self.allocate(Node::Leaf(value), edit)?;
        let mut children = [current; 2];
        children[side] = leaf;
        let branch = self.allocate(
            Node::Branch {
                bit,
                children,
                representative: leaf,
            },
            edit,
        )?;
        self.link(parent, branch, root, edit)
    }
    fn delete<const W: usize>(
        &mut self,
        root: &mut Root,
        key: &KeyView<'_, W>,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        let mut current = root.expect("existing exact-key deletion root");
        let mut parent = None;
        let mut grandparent = None;
        loop {
            (edit.step)()?;
            match self.forest.node(current).expect("exact-key deletion path") {
                Node::Branch { bit, children, .. } => {
                    grandparent = parent;
                    let side = key.bit(bit)?;
                    parent = Some((current, side));
                    current = children[side];
                }
                Node::Leaf(_) => break,
            }
        }
        if let Some((index, side)) = parent {
            let Node::Branch { children, .. } =
                self.forest.node(index).expect("exact-key deletion parent")
            else {
                unreachable!()
            };
            (edit.step)()?;
            let replacement = match self
                .forest
                .node(children[1 - side])
                .expect("exact-key sibling")
            {
                Node::Branch { representative, .. } => representative,
                Node::Leaf(_) => children[1 - side],
            };
            self.replace_representatives(
                root.expect("deletion root"),
                index,
                key,
                current,
                replacement,
                edit,
            )?;
            self.link(grandparent, children[1 - side], root, edit)?;
            self.remove(index, edit)?;
        } else {
            *root = None;
        }
        self.remove(current, edit)
    }
    fn replace_representatives<const W: usize>(
        &mut self,
        mut current: usize,
        stop: usize,
        key: &KeyView<'_, W>,
        old: usize,
        next: usize,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        while current != stop {
            (edit.step)()?;
            let Node::Branch {
                bit,
                children,
                representative,
            } = self.forest.node(current).expect("representative ancestor")
            else {
                unreachable!()
            };
            if representative == old {
                self.write(
                    current,
                    Node::Branch {
                        bit,
                        children,
                        representative: next,
                    },
                    edit,
                )?;
            }
            current = children[key.bit(bit)?];
        }
        Ok(())
    }
}

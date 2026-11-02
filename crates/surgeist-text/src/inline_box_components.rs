//! Components of supplied inline-box incidence in an already resolved visual order.
//! These local ordinals assert no Source, preparation, glyph or captured authority.
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::{InlineBoxFragmentationError, Result};
use std::ops::Range;

fn invalid() -> crate::Error {
    crate::protocol::protocol_error(crate::ProtocolError::InlineBoxFragmentation(
        InlineBoxFragmentationError::InvalidIncidence,
    ))
}
/// Whether an actual row can separate components. Metadata retains incidence
/// but does not split a component merely by interposing in visual order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBoxComponentParticipation {
    Participating,
    Metadata,
}
/// Memberships are active normal/root keys, not generated atomic leaf keys.
/// Participating rows name at least one owner; metadata may be unowned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuppliedInlineBoxComponentRow {
    pub owners: Range<usize>,
    pub participation: InlineBoxComponentParticipation,
}
/// Borrowed incidence over one supplied family. Bodies partition visual order;
/// order is a permutation of rows. The caller owns resolved ordering semantics.
///
/// Active owner keys index the original parents storage. Inactive generated
/// entries can remain there. The designated root terminates this family even
/// when its original generated parent is outside it. That external parent must
/// be inactive; a root parent that is itself active is a cycle and refuses.
#[derive(Clone, Copy, Debug)]
pub struct SuppliedInlineBoxComponentsInput<'a> {
    pub root: usize,
    pub owners: &'a [usize],
    pub parents: &'a [Option<usize>],
    pub rows: &'a [SuppliedInlineBoxComponentRow],
    pub memberships: &'a [usize],
    pub order: &'a [usize],
    pub bodies: &'a [Range<usize>],
}
#[derive(Debug, PartialEq)]
pub(crate) struct Component<O> {
    pub(crate) owner: O,
    pub(crate) body: usize,
    pub(crate) visual: Range<usize>,
    pub(crate) parent: Option<usize>,
}
#[derive(Debug, PartialEq)]
pub(crate) struct ComponentData<O> {
    pub(crate) components: Vec<Component<O>>,
    pub(crate) row_components: Vec<(usize, usize)>,
}
/// Owned component records and many-to-many row associations. No input borrow
/// or copy of the supplied parent tree is retained. Equality compares these
/// local derived records, not the identity of a source family or its inputs.
#[derive(Debug, PartialEq)]
pub struct SuppliedInlineBoxComponents {
    data: ComponentData<usize>,
}
/// A borrowed record; indices and visual ranges belong to this supplied result.
#[derive(Clone, Copy, Debug)]
pub struct SuppliedInlineBoxComponent<'a> {
    component: &'a Component<usize>,
}
impl SuppliedInlineBoxComponent<'_> {
    pub fn owner(&self) -> usize {
        self.component.owner
    }
    pub fn body(&self) -> usize {
        self.component.body
    }
    pub fn visual_range(&self) -> Range<usize> {
        self.component.visual.clone()
    }
    pub fn parent_index(&self) -> Option<usize> {
        self.component.parent
    }
}
impl SuppliedInlineBoxComponents {
    /// Validate the supplied active forest and derive its components. Failures
    /// use ProtocolError::InlineBoxFragmentation(InvalidIncidence), or the
    /// actual fallible allocation cause; no captured-source quota is invented.
    pub fn try_new(input: SuppliedInlineBoxComponentsInput<'_>) -> Result<Self> {
        let source = SuppliedSource(input);
        let mut ledger = None;
        validate(&source, &mut ledger)?;
        Ok(Self {
            data: derive(&source, &mut ledger)?,
        })
    }
    /// Indexed access to the same retained records returned by components().
    pub fn component(&self, index: usize) -> Option<SuppliedInlineBoxComponent<'_>> {
        self.data
            .components
            .get(index)
            .map(|component| SuppliedInlineBoxComponent { component })
    }
    pub fn components(&self) -> impl ExactSizeIterator<Item = SuppliedInlineBoxComponent<'_>> {
        self.data
            .components
            .iter()
            .map(|component| SuppliedInlineBoxComponent { component })
    }
    /// All component ordinals associated with this row, including ancestry.
    /// A row without associations (or outside the supplied inventory) is empty.
    pub fn row_components(&self, row: usize) -> impl Iterator<Item = usize> + '_ {
        let first = self
            .data
            .row_components
            .partition_point(|pair| pair.0 < row);
        let after = self
            .data
            .row_components
            .partition_point(|pair| pair.0 <= row);
        self.data.row_components[first..after]
            .iter()
            .map(|pair| pair.1)
    }
}

/// Only the two owning adapters implement this interface. Prepared borrows its
/// captured ancestry/overrides; supplied input borrows its admitted native map.
pub(crate) trait ComponentSource {
    type Owner: Copy + Eq;
    fn root(&self) -> Self::Owner;
    fn owners(&self) -> &[Self::Owner];
    fn parent(&self, owner: Self::Owner, ledger: &mut Ledger) -> Result<Option<Self::Owner>>;
    fn order(&self) -> &[usize];
    fn body_count(&self) -> usize;
    fn body(&self, body: usize) -> Range<usize>;
    fn content(&self, row: usize) -> bool;
    fn leaves(&self, row: usize) -> &[Self::Owner];
}
struct SuppliedSource<'a>(SuppliedInlineBoxComponentsInput<'a>);
impl ComponentSource for SuppliedSource<'_> {
    type Owner = usize;
    fn root(&self) -> usize {
        self.0.root
    }
    fn owners(&self) -> &[usize] {
        self.0.owners
    }
    fn parent(&self, owner: usize, _ledger: &mut Ledger) -> Result<Option<usize>> {
        Ok(self.0.parents[owner])
    }
    fn order(&self) -> &[usize] {
        self.0.order
    }
    fn body_count(&self) -> usize {
        self.0.bodies.len()
    }
    fn body(&self, body: usize) -> Range<usize> {
        self.0.bodies[body].clone()
    }
    fn content(&self, row: usize) -> bool {
        self.0.rows[row].participation == InlineBoxComponentParticipation::Participating
    }
    fn leaves(&self, row: usize) -> &[usize] {
        &self.0.memberships[self.0.rows[row].owners.clone()]
    }
}
fn active<S: ComponentSource>(source: &S, owner: S::Owner, ledger: &mut Ledger) -> Result<bool> {
    for &candidate in source.owners() {
        account::work(ledger, 1)?;
        if candidate == owner {
            return Ok(true);
        }
    }
    Ok(false)
}
fn validate(source: &SuppliedSource<'_>, ledger: &mut Ledger) -> Result<()> {
    let input = &source.0;
    if !active(source, input.root, ledger)? {
        return Err(invalid());
    }
    for (index, &owner) in input.owners.iter().enumerate() {
        account::work(ledger, 1)?;
        if owner >= input.parents.len() {
            return Err(invalid());
        }
        for &previous in &input.owners[..index] {
            account::work(ledger, 1)?;
            if previous == owner {
                return Err(invalid());
            }
        }
    }
    if let Some(parent) = input.parents[input.root]
        && active(source, parent, ledger)?
    {
        return Err(invalid());
    }
    for &owner in input.owners {
        let mut at = owner;
        let mut reached = false;
        for _ in 0..input.owners.len() {
            account::work(ledger, 1)?;
            if at == input.root {
                reached = true;
                break;
            }
            let parent = source.parent(at, ledger)?.ok_or_else(invalid)?;
            if !active(source, parent, ledger)? {
                return Err(invalid());
            }
            at = parent;
        }
        if !reached {
            return Err(invalid());
        }
    }
    for row in input.rows {
        account::work(ledger, 1)?;
        if row.owners.start > row.owners.end || row.owners.end > input.memberships.len() {
            return Err(invalid());
        }
        if row.participation == InlineBoxComponentParticipation::Participating
            && row.owners.is_empty()
        {
            return Err(invalid());
        }
        for &owner in &input.memberships[row.owners.clone()] {
            account::work(ledger, 1)?;
            if !active(source, owner, ledger)? {
                return Err(invalid());
            }
        }
    }
    if input.order.len() != input.rows.len() {
        return Err(invalid());
    }
    let mut seen = account::filled_fallible(ledger, input.rows.len(), false)?;
    for &row in input.order {
        account::work(ledger, 1)?;
        if row >= seen.len() || seen[row] {
            return Err(invalid());
        }
        seen[row] = true;
    }
    account::drop_vec(ledger, seen)?;
    let mut after = 0;
    for body in input.bodies {
        account::work(ledger, 1)?;
        if body.start != after || body.end < body.start || body.end > input.order.len() {
            return Err(invalid());
        }
        after = body.end;
    }
    if input.bodies.is_empty() || after != input.order.len() {
        return Err(invalid());
    }
    Ok(())
}
fn member<S: ComponentSource>(
    source: &S,
    row: usize,
    owner: S::Owner,
    ledger: &mut Ledger,
) -> Result<bool> {
    for &leaf in source.leaves(row) {
        account::work(ledger, 1)?;
        let mut at = Some(leaf);
        let mut steps = 0;
        while let Some(value) = at {
            account::work(ledger, 1)?;
            if value == owner {
                return Ok(true);
            }
            if value == source.root() {
                break;
            }
            if steps == source.owners().len() {
                return Err(invalid());
            }
            steps += 1;
            at = source.parent(value, ledger)?;
        }
    }
    Ok(false)
}
/// The one component/fragment-parent derivation. The Prepared caller has
/// already admitted its incidence; supplied input validates before entry.
pub(crate) fn derive<S: ComponentSource>(
    source: &S,
    ledger: &mut Ledger,
) -> Result<ComponentData<S::Owner>> {
    let mut components = Vec::new();
    for body in 0..source.body_count() {
        account::work(ledger, 1)?;
        let visual = source.body(body);
        account::push_fallible(
            ledger,
            &mut components,
            Component {
                owner: source.root(),
                body,
                visual: visual.clone(),
                parent: None,
            },
        )?;
        for &owner in source.owners() {
            account::work(ledger, 1)?;
            if owner == source.root() {
                continue;
            }
            let mut open = None;
            for rank in visual.clone() {
                account::work(ledger, 1)?;
                let row = source.order()[rank];
                if !source.content(row) {
                    continue;
                }
                if member(source, row, owner, ledger)? {
                    if open.is_none() {
                        open = Some(rank);
                    }
                } else if let Some(first) = open.take() {
                    account::push_fallible(
                        ledger,
                        &mut components,
                        Component {
                            owner,
                            body,
                            visual: first..rank,
                            parent: None,
                        },
                    )?;
                }
            }
            if let Some(first) = open {
                account::push_fallible(
                    ledger,
                    &mut components,
                    Component {
                        owner,
                        body,
                        visual: first..visual.end,
                        parent: None,
                    },
                )?;
            }
        }
    }
    for index in 0..components.len() {
        account::work(ledger, 1)?;
        let child = &components[index];
        if child.owner == source.root() {
            continue;
        }
        let parent = source.parent(child.owner, ledger)?.ok_or_else(invalid)?;
        let mut found = None;
        for (candidate, row) in components.iter().enumerate() {
            account::work(ledger, 1)?;
            if row.owner == parent
                && row.body == child.body
                && row.visual.start <= child.visual.start
                && child.visual.end <= row.visual.end
            {
                found = Some(candidate);
                break;
            }
        }
        components[index].parent = Some(found.ok_or_else(invalid)?);
    }
    let mut row_components = Vec::new();
    for (index, component) in components.iter().enumerate() {
        account::work(ledger, 1)?;
        for rank in component.visual.clone() {
            account::work(ledger, 1)?;
            let row = source.order()[rank];
            if member(source, row, component.owner, ledger)? {
                account::push_fallible(ledger, &mut row_components, (row, index))?;
            }
        }
    }
    crate::processing::contextual::alignment::operations::order_by(
        &mut row_components,
        ledger.as_mut(),
        |a, b| a.cmp(b),
    )?;
    Ok(ComponentData {
        components,
        row_components,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_shared_row_retains_both_sibling_memberships_and_their_ancestor() {
        // One genuine supplied row may be an indivisible shared packet. Its
        // two sibling incidences must not be reduced to an exclusive owner.
        let rows = [
            SuppliedInlineBoxComponentRow {
                owners: 0..2,
                participation: InlineBoxComponentParticipation::Participating,
            },
            SuppliedInlineBoxComponentRow {
                owners: 2..3,
                participation: InlineBoxComponentParticipation::Participating,
            },
        ];
        let result = SuppliedInlineBoxComponents::try_new(SuppliedInlineBoxComponentsInput {
            root: 0,
            owners: &[0, 1, 2],
            parents: &[None, Some(0), Some(0)],
            rows: &rows,
            memberships: &[1, 2, 0],
            order: &[0, 1],
            bodies: std::slice::from_ref(&(0..2)),
        })
        .unwrap();
        let records: Vec<_> = result
            .components()
            .map(|component| {
                (
                    component.owner(),
                    component.body(),
                    component.visual_range(),
                    component.parent_index(),
                )
            })
            .collect();
        assert_eq!(
            records,
            vec![
                (0, 0, 0..2, None),
                (1, 0, 0..1, Some(0)),
                (2, 0, 0..1, Some(0))
            ]
        );
        assert_eq!(result.row_components(0).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(result.row_components(1).collect::<Vec<_>>(), vec![0]);
        assert_eq!(result.component(2).unwrap().owner(), 2);
        assert!(result.component(3).is_none());
    }
}

#[cfg(test)]
#[path = "inline_box_components/functional_tests.rs"]
mod functional_tests;

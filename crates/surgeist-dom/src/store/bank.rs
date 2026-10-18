//! Persistent indexed records. Slot identity and generation policy belong to
//! the consuming owner; this bank owns path copying and vacancy summaries.

use std::{mem::size_of, sync::Arc};

use super::DomError;

const BITS: u32 = 4;
const WIDTH: usize = 1 << BITS;
const MASK: usize = WIDTH - 1;
const MAX_DEPTH: usize = usize::BITS.div_ceil(BITS) as usize;

/// Conservative payload charges per retained occurrence, supplied by the
/// record owner. Sharing never discounts these totals.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Totals {
    pub bytes: usize,
    pub units: usize,
    pub occurrences: usize,
}

impl Totals {
    fn checked_add(self, other: Self) -> Result<Self, DomError> {
        Ok(Self {
            bytes: checked_add(self.bytes, other.bytes)?,
            units: checked_add(self.units, other.units)?,
            occurrences: checked_add(self.occurrences, other.occurrences)?,
        })
    }
}

/// Actual Page values and their Vec backing capacities. Record allocations and
/// Arc control blocks are accounted by the owner, separately from this charge.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct IndexCharge {
    pub bytes: usize,
    pub pages: usize,
}

impl IndexCharge {
    fn checked_add(self, other: Self) -> Result<Self, DomError> {
        Ok(Self {
            bytes: checked_add(self.bytes, other.bytes)?,
            pages: checked_add(self.pages, other.pages)?,
        })
    }
}

#[derive(Debug)]
struct Entry<T> {
    value: Arc<T>,
    reusable: bool,
    totals: Totals,
}

impl<T> Clone for Entry<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            reusable: self.reusable,
            totals: self.totals,
        }
    }
}

#[derive(Debug)]
enum Contents<T> {
    Leaf(Vec<Option<Entry<T>>>),
    Branch(Vec<Option<Arc<Page<T>>>>),
}

#[derive(Clone, Copy, Debug, Default)]
struct Summary {
    records: usize,
    vacant: Option<usize>,
    totals: Totals,
    index: IndexCharge,
}

#[derive(Debug)]
struct Page<T> {
    /// Child-index shift; zero identifies a leaf page.
    shift: u32,
    contents: Contents<T>,
    summary: Summary,
}

type PageReplacement<T> = (Option<Arc<Page<T>>>, IndexCharge);

/// An immutable shared radix root with a mutable publication handle. Missing
/// slots below `len` are eligible vacancies; stored records independently mark
/// eligibility, allowing generation-exhausted vacant records to remain present.
#[derive(Debug)]
pub(crate) struct Bank<T> {
    root: Option<Arc<Page<T>>>,
    extent: usize,
}

impl<T> Clone for Bank<T> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.as_ref().map(Arc::clone),
            extent: self.extent,
        }
    }
}

impl<T> Default for Bank<T> {
    fn default() -> Self {
        Self {
            root: None,
            extent: 0,
        }
    }
}

impl<T: PartialEq> PartialEq for Bank<T> {
    /// Semantic record comparison excludes derived eligibility and charges.
    fn eq(&self, other: &Self) -> bool {
        self.extent == other.extent
            && self.record_count() == other.record_count()
            && (self.shares_root(other) || self.iter().eq(other.iter()))
    }
}

impl<T: Eq> Eq for Bank<T> {}

impl<'a, T> IntoIterator for &'a Bank<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Map<Iter<'a, T>, fn((usize, &'a T)) -> &'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().map(|(_, value)| value)
    }
}

impl<T> std::ops::Index<usize> for Bank<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("indexed bank record is present")
    }
}

impl<T> Bank<T> {
    /// Allocated slot extent, including implicit holes and retained vacancies.
    pub fn len(&self) -> usize {
        self.extent
    }

    pub fn is_empty(&self) -> bool {
        self.extent == 0
    }

    pub fn record_count(&self) -> usize {
        self.root.as_ref().map_or(0, |page| page.summary.records)
    }

    pub fn totals(&self) -> Totals {
        self.root
            .as_ref()
            .map_or(Totals::default(), |page| page.summary.totals)
    }

    pub fn index_charge(&self) -> IndexCharge {
        self.root
            .as_ref()
            .map_or(IndexCharge::default(), |page| page.summary.index)
    }

    /// Current index allocations absent from a related immutable basis. Shared
    /// subtrees stop traversal immediately; a wholly new subtree contributes
    /// its cached charge. Superseded candidate paths are not counted.
    #[cfg(test)]
    pub fn unshared_index_charge(&self, base: &Self) -> IndexCharge {
        self.root.as_ref().map_or(IndexCharge::default(), |root| {
            unshared_index(root, base.root.as_ref(), None)
        })
    }

    /// Candidate-owned existing pages that rewriting `index` would replace.
    /// This query visits only that index path, regardless of other edits.
    #[cfg(test)]
    pub fn path_index_charge_not_shared_with(&self, base: &Self, index: usize) -> IndexCharge {
        self.root
            .as_ref()
            .filter(|root| fits(index, root.shift))
            .map_or(IndexCharge::default(), |root| {
                unshared_index(root, base.root.as_ref(), Some(index))
            })
    }

    pub fn shares_root(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.get_shared(index).map(Arc::as_ref)
    }

    pub fn get_shared(&self, index: usize) -> Option<&Arc<T>> {
        self.entry(index).map(|entry| &entry.value)
    }

    pub fn totals_at(&self, index: usize) -> Option<Totals> {
        self.entry(index).map(|entry| entry.totals)
    }

    fn entry(&self, index: usize) -> Option<&Entry<T>> {
        if index >= self.extent {
            return None;
        }
        let mut page = self.root.as_deref()?;
        if !fits(index, page.shift) {
            return None;
        }
        loop {
            match &page.contents {
                Contents::Leaf(entries) => {
                    return entries[index & MASK].as_ref();
                }
                Contents::Branch(children) => {
                    page = children[(index >> page.shift) & MASK].as_deref()?;
                }
            }
        }
    }

    /// Replace or sparsely extend one slot. All fallible work precedes the root
    /// swap; errors retain the exact original root and extent. The returned
    /// charge covers newly allocated index paths, for overlap admission.
    #[cfg(test)]
    pub fn set(
        &mut self,
        index: usize,
        value: Arc<T>,
        reusable: bool,
        totals: Totals,
    ) -> Result<IndexCharge, DomError> {
        self.set_admitted(index, value, reusable, totals, |_| Ok(()))
    }

    /// Admit page values and backing before allocation, then any actual reserve
    /// excess immediately after allocation, matching the owner's Vec policy.
    /// Every returned allocation remains in the resulting root; root growth
    /// includes retained leading-zero wrappers outside the edited path.
    pub fn set_admitted(
        &mut self,
        index: usize,
        value: Arc<T>,
        reusable: bool,
        totals: Totals,
        mut admit: impl FnMut(IndexCharge) -> Result<(), DomError>,
    ) -> Result<IndexCharge, DomError> {
        let extent = checked_add(index, 1)?.max(self.extent);
        let mut root = self.root.as_ref().map(Arc::clone);
        let mut shift = root.as_ref().map_or(0, |page| page.shift);
        let mut growth = IndexCharge::default();
        while !fits(index, shift) {
            shift += BITS;
        }
        while root.as_ref().is_some_and(|page| page.shift + BITS < shift) {
            if let Some(previous) = root.take() {
                let next_shift = previous.shift + BITS;
                let mut children = empty_slots(size_of::<Page<T>>(), &mut admit)?;
                children[0] = Some(previous);
                let (page, charge) = make_page(next_shift, Contents::Branch(children))?;
                root = Some(page);
                growth = growth.checked_add(charge)?;
            }
        }
        let entry = Entry {
            value,
            reusable,
            totals,
        };
        let (root, charge) = replace(root.as_ref(), shift, index, Some(entry), &mut admit)?;
        let charge = growth.checked_add(charge)?;
        self.root = root;
        self.extent = extent;
        Ok(charge)
    }

    /// Compute a replacement from the borrowed current record without cloning
    /// unrelated records. A callback refusal preserves the original bank.
    pub fn update(
        &mut self,
        index: usize,
        change: impl FnOnce(Option<&T>) -> Result<(Arc<T>, bool, Totals), DomError>,
        admit: impl FnMut(IndexCharge) -> Result<(), DomError>,
    ) -> Result<IndexCharge, DomError> {
        let (value, reusable, totals) = change(self.get(index))?;
        self.set_admitted(index, value, reusable, totals, admit)
    }

    /// Remove a record but preserve the historical extent. Consumers retaining
    /// generation history should instead replace it with a vacancy record.
    #[cfg(test)]
    pub fn remove(&mut self, index: usize) -> Result<(Option<Arc<T>>, IndexCharge), DomError> {
        self.remove_admitted(index, |_| Ok(()))
    }

    #[cfg(test)]
    pub fn remove_admitted(
        &mut self,
        index: usize,
        mut admit: impl FnMut(IndexCharge) -> Result<(), DomError>,
    ) -> Result<(Option<Arc<T>>, IndexCharge), DomError> {
        let Some(value) = self.get_shared(index).cloned() else {
            return Ok((None, IndexCharge::default()));
        };
        let shift = self
            .root
            .as_ref()
            .expect("a present record has a root")
            .shift;
        let (root, charge) = replace(self.root.as_ref(), shift, index, None, &mut admit)?;
        self.root = root;
        Ok((Some(value), charge))
    }

    /// Lowest eligible slot within the allocated extent. Append is a separate
    /// checked operation at `len()`, so a full bank returns None here.
    pub fn lowest_vacant(&self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        let Some(root) = &self.root else {
            return Some(0);
        };
        let candidate = root
            .summary
            .vacant
            .or_else(|| 1usize.checked_shl(root.shift + BITS))?;
        (candidate < self.extent).then_some(candidate)
    }

    /// Present records in exact ascending slot order, without visiting holes.
    pub fn iter(&self) -> Iter<'_, T> {
        let mut iter = Iter {
            stack: std::array::from_fn(|_| None),
            depth: 0,
        };
        if let Some(root) = self.root.as_deref() {
            iter.stack[0] = Some(Frame {
                page: root,
                next: 0,
                base: 0,
            });
            iter.depth = 1;
        }
        iter
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|(_, value)| value)
    }
}

fn fits(index: usize, shift: u32) -> bool {
    index >> shift < WIDTH
}

#[cfg(test)]
fn unshared_index<T>(
    current: &Arc<Page<T>>,
    mut base: Option<&Arc<Page<T>>>,
    path: Option<usize>,
) -> IndexCharge {
    // A smaller current root can only overlap the leading-zero subtree of a
    // larger basis. Root growth follows the same alignment in reverse below.
    while let Some(previous) = base {
        if previous.shift <= current.shift {
            break;
        }
        let Contents::Branch(children) = &previous.contents else {
            unreachable!("larger address depth identifies a branch");
        };
        base = children[0].as_ref();
    }
    if base.is_none() && path.is_none() {
        return current.summary.index;
    }
    if base.is_some_and(|base| Arc::ptr_eq(current, base)) {
        return IndexCharge::default();
    }
    let backing = match &current.contents {
        Contents::Leaf(entries) => entries.capacity() * size_of::<Option<Entry<T>>>(),
        Contents::Branch(children) => children.capacity() * size_of::<Option<Arc<Page<T>>>>(),
    };
    // Construction already checked each own-page charge and the full subtree
    // sum. Every addition below is a disjoint subset of that validated sum.
    let mut charge = IndexCharge {
        bytes: size_of::<Page<T>>() + backing,
        pages: 1,
    };
    if let Contents::Branch(children) = &current.contents {
        for (index, child) in children.iter().enumerate() {
            if path.is_some_and(|path| (path >> current.shift) & MASK != index) {
                continue;
            }
            let Some(child) = child else { continue };
            let previous = if let Some(base) = base.filter(|base| base.shift == current.shift) {
                let Contents::Branch(old) = &base.contents else {
                    unreachable!("equal nonzero address depth identifies branches");
                };
                old[index].as_ref()
            } else if index == 0 {
                base
            } else {
                None
            };
            let child = unshared_index(child, previous, path);
            charge.bytes += child.bytes;
            charge.pages += child.pages;
        }
    }
    charge
}

fn checked_add(left: usize, right: usize) -> Result<usize, DomError> {
    left.checked_add(right)
        .ok_or(DomError::RepresentationExhausted)
}

fn empty_slots<T>(
    page_bytes: usize,
    admit: &mut impl FnMut(IndexCharge) -> Result<(), DomError>,
) -> Result<Vec<Option<T>>, DomError> {
    let requested = WIDTH
        .checked_mul(size_of::<Option<T>>())
        .ok_or(DomError::RepresentationExhausted)?;
    admit(IndexCharge {
        bytes: checked_add(page_bytes, requested)?,
        pages: 1,
    })?;
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(WIDTH)
        .map_err(|_| DomError::AllocationFailed)?;
    if slots.capacity() > WIDTH {
        let extra = (slots.capacity() - WIDTH)
            .checked_mul(size_of::<Option<T>>())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(IndexCharge {
            bytes: extra,
            pages: 0,
        })?;
    }
    slots.resize_with(WIDTH, || None);
    Ok(slots)
}

/// Recursion is bounded by machine index width / BITS, never by record count.
fn replace<T>(
    previous: Option<&Arc<Page<T>>>,
    shift: u32,
    index: usize,
    replacement: Option<Entry<T>>,
    admit: &mut impl FnMut(IndexCharge) -> Result<(), DomError>,
) -> Result<PageReplacement<T>, DomError> {
    if replacement.is_none() && previous.is_some_and(|page| page.summary.records == 1) {
        return Ok((None, IndexCharge::default()));
    }
    let (contents, descendant_charge) = if shift == 0 {
        let mut entries = empty_slots(size_of::<Page<T>>(), admit)?;
        if let Some(previous) = previous {
            let Contents::Leaf(old) = &previous.contents else {
                unreachable!("zero shift identifies a leaf");
            };
            entries.clone_from_slice(old);
        }
        entries[index & MASK] = replacement;
        (Contents::Leaf(entries), IndexCharge::default())
    } else {
        let mut children = empty_slots(size_of::<Page<T>>(), admit)?;
        if let Some(previous) = previous {
            if previous.shift == shift {
                let Contents::Branch(old) = &previous.contents else {
                    unreachable!("nonzero shift identifies a branch");
                };
                children.clone_from_slice(old);
            } else {
                // Lower leading-zero wrappers are final off-path children.
                // Build the new root directly, without allocating and copying
                // a temporary wrapper at this same depth.
                debug_assert_eq!(previous.shift + BITS, shift);
                children[0] = Some(Arc::clone(previous));
            }
        }
        let digit = (index >> shift) & MASK;
        let (child, charge) = replace(
            children[digit].as_ref(),
            shift - BITS,
            index,
            replacement,
            admit,
        )?;
        children[digit] = child;
        (Contents::Branch(children), charge)
    };
    let (page, own) = make_page(shift, contents)?;
    let charge = own.checked_add(descendant_charge)?;
    Ok((Some(page), charge))
}

fn make_page<T>(
    shift: u32,
    contents: Contents<T>,
) -> Result<(Arc<Page<T>>, IndexCharge), DomError> {
    let own = IndexCharge {
        bytes: checked_add(
            size_of::<Page<T>>(),
            match &contents {
                Contents::Leaf(entries) => entries
                    .capacity()
                    .checked_mul(size_of::<Option<Entry<T>>>())
                    .ok_or(DomError::RepresentationExhausted)?,
                Contents::Branch(children) => children
                    .capacity()
                    .checked_mul(size_of::<Option<Arc<Page<T>>>>())
                    .ok_or(DomError::RepresentationExhausted)?,
            },
        )?,
        pages: 1,
    };
    let mut summary = Summary {
        index: own,
        ..Summary::default()
    };
    match &contents {
        Contents::Leaf(entries) => {
            for (index, entry) in entries.iter().enumerate() {
                if let Some(entry) = entry {
                    summary.records = checked_add(summary.records, 1)?;
                    summary.totals = summary.totals.checked_add(entry.totals)?;
                }
                if summary.vacant.is_none() && entry.as_ref().is_none_or(|entry| entry.reusable) {
                    summary.vacant = Some(index);
                }
            }
        }
        Contents::Branch(children) => {
            for (index, child) in children.iter().enumerate() {
                let vacant = if let Some(child) = child {
                    summary.records = checked_add(summary.records, child.summary.records)?;
                    summary.totals = summary.totals.checked_add(child.summary.totals)?;
                    summary.index = summary.index.checked_add(child.summary.index)?;
                    child.summary.vacant
                } else {
                    Some(0)
                };
                if summary.vacant.is_none()
                    && let Some(vacant) = vacant
                {
                    summary.vacant = Some(checked_add(index << shift, vacant)?);
                }
            }
        }
    }
    Ok((
        Arc::new(Page {
            shift,
            contents,
            summary,
        }),
        own,
    ))
}

struct Frame<'a, T> {
    page: &'a Page<T>,
    next: usize,
    base: usize,
}

pub(crate) struct Iter<'a, T> {
    stack: [Option<Frame<'a, T>>; MAX_DEPTH],
    depth: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.depth != 0 {
            let frame = self.stack[self.depth - 1].as_mut().expect("active frame");
            if frame.next == WIDTH {
                self.stack[self.depth - 1] = None;
                self.depth -= 1;
                continue;
            }
            let index = frame.next;
            frame.next += 1;
            match &frame.page.contents {
                Contents::Leaf(entries) => {
                    if let Some(entry) = &entries[index] {
                        return Some((frame.base + index, &entry.value));
                    }
                }
                Contents::Branch(children) => {
                    if let Some(child) = children[index].as_deref() {
                        let base = frame.base + (index << frame.page.shift);
                        self.stack[self.depth] = Some(Frame {
                            page: child,
                            next: 0,
                            base,
                        });
                        self.depth += 1;
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn put(bank: &mut Bank<u64>, index: usize, value: u64, reusable: bool) -> IndexCharge {
        bank.set(
            index,
            Arc::new(value),
            reusable,
            Totals {
                bytes: 8,
                units: value as usize,
                occurrences: 1,
            },
        )
        .unwrap()
    }

    #[test]
    fn sparse_radix_boundaries_keep_literal_indices_and_order() {
        let mut bank = Bank::default();
        assert!(bank.is_empty());
        assert_eq!(bank.lowest_vacant(), None);
        assert_eq!(bank.remove(0).unwrap(), (None, IndexCharge::default()));
        for (index, value) in [
            (4096, 7),
            (0, 1),
            (256, 5),
            (15, 2),
            (4095, 6),
            (16, 3),
            (255, 4),
        ] {
            put(&mut bank, index, value, false);
        }
        assert_eq!(bank.len(), 4097);
        assert_eq!(bank.record_count(), 7);
        assert_eq!(
            bank.iter()
                .map(|(index, value)| (index, *value))
                .collect::<Vec<_>>(),
            [
                (0, 1),
                (15, 2),
                (16, 3),
                (255, 4),
                (256, 5),
                (4095, 6),
                (4096, 7)
            ]
        );
        assert_eq!(bank.get(17), None);
        assert_eq!(bank.get(4097), None);
        assert_eq!(bank[256], 5);
        assert_eq!(
            bank.totals(),
            Totals {
                bytes: 56,
                units: 28,
                occurrences: 7
            }
        );
        assert_eq!(bank.lowest_vacant(), Some(1));
    }

    #[test]
    fn snapshots_keep_prior_records_and_share_unchanged_record_backing() {
        let mut bank = Bank::default();
        put(&mut bank, 0, 10, false);
        put(&mut bank, 16, 20, false);
        let captured = bank.clone();
        assert!(bank.shares_root(&captured));
        let retained = Arc::clone(bank.get_shared(16).unwrap());
        bank.update(
            0,
            |old| {
                assert_eq!(old, Some(&10));
                Ok((
                    Arc::new(30),
                    false,
                    Totals {
                        bytes: 8,
                        units: 30,
                        occurrences: 1,
                    },
                ))
            },
            |_| Ok(()),
        )
        .unwrap();
        assert!(!bank.shares_root(&captured));
        assert_eq!(captured.get(0), Some(&10));
        assert_eq!(bank.get(0), Some(&30));
        assert_eq!(captured.totals().units, 30);
        assert_eq!(bank.totals().units, 50);
        assert!(Arc::ptr_eq(bank.get_shared(16).unwrap(), &retained));
        assert!(Arc::ptr_eq(captured.get_shared(16).unwrap(), &retained));
    }

    #[test]
    fn vacancies_skip_ineligible_records_and_return_lowest_reusable_slot() {
        // Existing records model occupied slots or generation-exhausted holes;
        // the bank must skip both when their owner marks them ineligible.
        let mut bank = Bank::default();
        for index in 0..34 {
            put(&mut bank, index, index as u64, false);
        }
        assert_eq!(bank.lowest_vacant(), None);
        put(&mut bank, 33, 33, true);
        put(&mut bank, 16, 16, true);
        put(&mut bank, 2, 2, true);
        assert_eq!(bank.lowest_vacant(), Some(2));
        put(&mut bank, 2, 40, false);
        assert_eq!(bank.lowest_vacant(), Some(16));
        put(&mut bank, 16, 41, false);
        assert_eq!(bank.lowest_vacant(), Some(33));
        put(&mut bank, 33, 42, false);
        let (removed, _) = bank.remove(15).unwrap();
        assert_eq!(removed.as_deref(), Some(&15));
        assert_eq!(bank.len(), 34);
        assert_eq!(bank.lowest_vacant(), Some(15));
        put(&mut bank, 15, 43, false);
        assert_eq!(bank.lowest_vacant(), None);
    }

    #[test]
    fn removal_keeps_extent_and_implicit_holes_after_all_pages_are_reclaimed() {
        let mut bank = Bank::default();
        put(&mut bank, 4096, 1, false);
        bank.remove(4096).unwrap();
        assert_eq!(bank.len(), 4097);
        assert_eq!(bank.record_count(), 0);
        assert_eq!(bank.index_charge(), IndexCharge::default());
        assert_eq!(bank.totals(), Totals::default());
        assert_eq!(bank.lowest_vacant(), Some(0));
        for index in 0..16 {
            put(&mut bank, index, 1, false);
        }
        assert_eq!(bank.lowest_vacant(), Some(16));
        assert_eq!(bank.get(32), None);
        assert_eq!(bank.iter().count(), 16);
    }

    #[test]
    fn maximum_extent_remains_sparse_and_overflow_refuses_without_root_change() {
        let mut bank = Bank::default();
        let charge = put(&mut bank, usize::MAX - 1, 99, false);
        assert_eq!(bank.len(), usize::MAX);
        assert_eq!(bank.get(usize::MAX - 1), Some(&99));
        assert_eq!(bank.get(usize::MAX), None);
        assert_eq!(
            bank.iter()
                .map(|(index, value)| (index, *value))
                .collect::<Vec<_>>(),
            [(usize::MAX - 1, 99)]
        );
        assert_eq!(bank.lowest_vacant(), Some(0));
        assert!(charge.pages <= MAX_DEPTH);
        let captured = bank.clone();
        assert_eq!(
            bank.set(usize::MAX, Arc::new(1), false, Totals::default()),
            Err(DomError::RepresentationExhausted)
        );
        assert!(bank.shares_root(&captured));
        assert_eq!(bank.len(), captured.len());
    }

    #[test]
    fn refused_callback_or_overflowed_totals_retains_exact_captured_root() {
        let mut bank = Bank::default();
        bank.set(
            0,
            Arc::new(1),
            false,
            Totals {
                bytes: usize::MAX,
                units: 0,
                occurrences: 1,
            },
        )
        .unwrap();
        let captured = bank.clone();
        assert_eq!(
            bank.update(0, |_| Err(DomError::Cancelled), |_| Ok(())),
            Err(DomError::Cancelled)
        );
        assert!(bank.shares_root(&captured));
        assert_eq!(
            bank.set(
                16,
                Arc::new(2),
                false,
                Totals {
                    bytes: 1,
                    units: 0,
                    occurrences: 1,
                }
            ),
            Err(DomError::RepresentationExhausted)
        );
        assert!(bank.shares_root(&captured));
        assert_eq!(bank.len(), 1);
        assert_eq!(bank.get(0), Some(&1));
        assert_eq!(bank.totals().bytes, usize::MAX);
    }

    #[test]
    fn page_admission_refusal_retains_original_records_and_exact_root() {
        let mut bank = Bank::default();
        put(&mut bank, 0, 10, false);
        put(&mut bank, 256, 20, false);
        let captured = bank.clone();
        let mut pages = 0;
        let refused = bank.set_admitted(1, Arc::new(30), false, Totals::default(), |charge| {
            pages += charge.pages;
            if pages == 2 {
                Err(DomError::Cancelled)
            } else {
                Ok(())
            }
        });
        assert_eq!(refused, Err(DomError::Cancelled));
        assert!(bank.shares_root(&captured));
        assert_eq!(bank.get(1), None);
        assert_eq!(bank.totals(), captured.totals());
        let refused = bank.remove_admitted(0, |_| Err(DomError::Cancelled));
        assert_eq!(refused, Err(DomError::Cancelled));
        assert!(bank.shares_root(&captured));
        assert_eq!(bank.get(0), Some(&10));
    }

    #[test]
    fn local_replacement_allocates_only_bounded_index_paths() {
        let mut bank = Bank::default();
        for index in 0..8192 {
            put(&mut bank, index, 1, false);
        }
        let captured = bank.clone();
        let unchanged = Arc::clone(bank.get_shared(8000).unwrap());
        let all_pages = bank.index_charge().pages;
        let charge = put(&mut bank, 17, 2, false);
        assert!(charge.pages <= MAX_DEPTH);
        assert!(charge.pages < all_pages);
        assert!(charge.bytes < bank.index_charge().bytes);
        assert!(Arc::ptr_eq(bank.get_shared(8000).unwrap(), &unchanged));
        assert_eq!(captured.get(17), Some(&1));
        assert_eq!(bank.get(17), Some(&2));
        assert_eq!(
            bank.totals(),
            Totals {
                bytes: 65536,
                units: 8193,
                occurrences: 8192
            }
        );
    }

    #[test]
    fn candidate_index_charge_excludes_shared_and_superseded_paths_across_root_growth() {
        let mut bank = Bank::default();
        put(&mut bank, 0, 1, false);
        put(&mut bank, 16, 2, false);
        let basis = bank.clone();
        assert_eq!(bank.unshared_index_charge(&basis), IndexCharge::default());
        put(&mut bank, 0, 3, false);
        let first = bank.unshared_index_charge(&basis);
        put(&mut bank, 0, 4, false);
        assert_eq!(bank.unshared_index_charge(&basis), first);
        assert!(first.bytes < bank.index_charge().bytes);
        put(&mut bank, 4096, 5, false);
        let grown = bank.unshared_index_charge(&basis);
        assert!(grown.bytes > first.bytes);
        assert!(grown.bytes < bank.index_charge().bytes);
        assert_eq!(
            bank.unshared_index_charge(&Bank::default()),
            bank.index_charge()
        );
        let mut small = basis.clone();
        small.remove(0).unwrap();
        small.remove(16).unwrap();
        put(&mut small, 0, 6, false);
        assert_eq!(small.unshared_index_charge(&bank), small.index_charge());
        assert_eq!(basis.get(0), Some(&1));
        assert_eq!(basis.get(16), Some(&2));
    }

    #[test]
    fn path_charge_deltas_match_current_candidate_allocations_including_growth() {
        let mut bank = Bank::default();
        put(&mut bank, 0, 1, false);
        let basis = bank.clone();
        let mut live = IndexCharge::default();
        for (index, value) in [(0, 2), (4096, 3), (0, 4), (256, 5), (4096, 6), (15, 7)] {
            let replaced = bank.path_index_charge_not_shared_with(&basis, index);
            let allocated = put(&mut bank, index, value, false);
            live.bytes = live.bytes + allocated.bytes - replaced.bytes;
            live.pages = live.pages + allocated.pages - replaced.pages;
            assert_eq!(live, bank.unshared_index_charge(&basis));
        }
        for index in [4096, 15, 0, 256] {
            let replaced = bank.path_index_charge_not_shared_with(&basis, index);
            let (_, allocated) = bank.remove(index).unwrap();
            live.bytes = live.bytes + allocated.bytes - replaced.bytes;
            live.pages = live.pages + allocated.pages - replaced.pages;
            assert_eq!(live, bank.unshared_index_charge(&basis));
        }
        assert_eq!(live, IndexCharge::default());
    }

    #[test]
    fn ordered_bulk_allocation_consumes_vacancies_without_prefix_scanning() {
        let mut bank = Bank::default();
        for index in 0..4096 {
            put(&mut bank, index, 0, false);
        }
        for index in [4000, 17, 255, 16] {
            put(&mut bank, index, 0, true);
        }
        let mut selected = Vec::new();
        for value in 1..=5 {
            let index = bank.lowest_vacant().unwrap_or(bank.len());
            selected.push(index);
            let charge = put(&mut bank, index, value, false);
            assert!(charge.pages <= 2 * MAX_DEPTH);
        }
        assert_eq!(selected, [16, 17, 255, 4000, 4096]);
        assert_eq!(bank.len(), 4097);
    }

    #[test]
    fn dropping_snapshot_releases_only_records_no_longer_retained() {
        #[derive(Debug)]
        struct Observed(Arc<AtomicUsize>);
        impl Drop for Observed {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let drops = Arc::new(AtomicUsize::new(0));
        let mut bank = Bank::default();
        for index in [0, 1, 256] {
            bank.set(
                index,
                Arc::new(Observed(Arc::clone(&drops))),
                false,
                Totals::default(),
            )
            .unwrap();
        }
        let captured = bank.clone();
        bank.set(
            0,
            Arc::new(Observed(Arc::clone(&drops))),
            false,
            Totals::default(),
        )
        .unwrap();
        assert_eq!(drops.load(Ordering::Relaxed), 0);
        drop(captured);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        assert_eq!(bank.record_count(), 3);
        drop(bank);
        assert_eq!(drops.load(Ordering::Relaxed), 4);
    }
}

//! Exclusive mutable use of the Bank radix pages. No page/record alias, weak
//! registration, shared frontend or conversion escapes this facade. Reservations
//! contain only empty allocations; resident values never enter a candidate.
use super::*;

struct Slot<T> {
    value: Option<T>,
    held: bool,
    active: bool,
    _producer: Option<crate::hosted::Retained>,
}
struct Spare<T> {
    prefix: usize,
    shift: u32,
    page: Arc<Page<Slot<T>>>,
    charge: IndexCharge,
}
/// Empty storage admitted before a caller's irreversible issuer transition.
pub(crate) struct EmptyPath<T> {
    index: usize,
    record: Option<Record<Slot<T>>>,
    pages: [Option<Spare<T>>; MAX_DEPTH * 2],
    charge: IndexCharge,
    moved: IndexCharge,
    minimum_shift: u32,
}
impl<T> EmptyPath<T> {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn charge(&self) -> IndexCharge {
        self.charge
    }
    fn page(&mut self, index: usize, shift: u32) -> Arc<Page<Slot<T>>> {
        let prefix = prefix(index, shift);
        let spare = self
            .pages
            .iter_mut()
            .find(|p| {
                p.as_ref()
                    .is_some_and(|p| p.shift == shift && p.prefix == prefix)
            })
            .and_then(Option::take)
            .expect("admitted complete destination page");
        self.moved = self
            .moved
            .checked_add(spare.charge)
            .expect("moved admitted subset");
        spare.page
    }
}
/// A taken value pins its existing slot and all ancestors until restoration.
pub(crate) struct Taken<T> {
    index: usize,
    value: T,
    reusable: bool,
}
impl<T> Taken<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
/// Deliberately not Clone: Page AND Entry.value remain uniquely owned. Raw Bank,
/// Arc and custody interfaces are inaccessible to consuming owners.
pub(crate) struct OwnedBank<T> {
    bank: Bank<Slot<T>>,
}
impl<T> Default for OwnedBank<T> {
    fn default() -> Self {
        Self {
            bank: Bank::default(),
        }
    }
}
impl<T> std::fmt::Debug for OwnedBank<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedBank")
            .field("extent", &self.bank.extent)
            .finish_non_exhaustive()
    }
}
fn prefix(index: usize, shift: u32) -> usize {
    index.checked_shr(shift + BITS).unwrap_or(0)
}
#[cfg(test)]
fn visited() {
    metric(Event::Entered(Domain::Bank));
    metric(Event::BankPageVisited);
}
fn empty_page<T, E: From<DomError>>(
    index: usize,
    shift: u32,
    admit: &mut impl FnMut(IndexCharge) -> Result<Option<crate::hosted::Retained>, E>,
) -> Result<Spare<T>, E> {
    let identity = MaterializationId::fresh()?;
    let (contents, producer) = if shift == 0 {
        let rows = empty_slots_with(size_of::<Page<Slot<T>>>(), admit)?;
        (Contents::Leaf(rows.slots), rows.producer)
    } else {
        let children = empty_slots_with(size_of::<Page<Slot<T>>>(), admit)?;
        (Contents::Branch(children.slots), children.producer)
    };
    let (page, charge) = make_page_with_producer(identity, shift, contents, None, producer)?;
    Ok(Spare {
        prefix: prefix(index, shift),
        shift,
        page,
        charge,
    })
}
fn unique<T>(page: &mut Arc<Page<Slot<T>>>) -> &mut Page<Slot<T>> {
    Arc::get_mut(page).expect("owned facade never exposes strong or weak Page aliases")
}
fn slot_mut<T>(page: &mut Arc<Page<Slot<T>>>, index: usize) -> Option<&mut Entry<Slot<T>>> {
    #[cfg(test)]
    visited();
    let page = unique(page);
    match &mut page.contents {
        Contents::Leaf(rows) => {
            #[cfg(test)]
            metric(Event::CanonicalRecordRead);
            rows[index & MASK].as_mut()
        }
        Contents::Branch(children) => {
            slot_mut(children[(index >> page.shift) & MASK].as_mut()?, index)
        }
    }
}
fn value_mut<T>(entry: &mut Entry<Slot<T>>) -> &mut Slot<T> {
    Arc::get_mut(&mut entry.value)
        .expect("owned facade never exposes strong or weak record aliases")
}
// The sums cover only distinct already-admitted allocations. Recalculation is
// bounded by fanout; address/aggregate headroom was checked at reservation.
fn refresh<T>(page: &mut Page<Slot<T>>) {
    #[cfg(test)]
    visited();
    let bytes = size_of::<Page<Slot<T>>>()
        + match &page.contents {
            Contents::Leaf(v) => v.capacity() * size_of::<Option<Entry<Slot<T>>>>(),
            Contents::Branch(v) => v.capacity() * size_of::<Option<Arc<Page<Slot<T>>>>>(),
        };
    let mut summary = Summary {
        index: IndexCharge { bytes, pages: 1 },
        ..Default::default()
    };
    match &page.contents {
        Contents::Leaf(rows) => {
            for (i, row) in rows.iter().enumerate() {
                #[cfg(test)]
                metric(Event::CanonicalRecordRead);
                if let Some(row) = row {
                    summary.records += 1;
                    summary.totals = summary
                        .totals
                        .checked_add(row.totals)
                        .expect("reserved aggregate headroom");
                    // This facade has no logical payload units. Its private
                    // units coordinate is the active subset cardinality;
                    // records/occurrences continue counting all retained rows.
                    let slot = &row.value;
                    if slot.active && !slot.held && slot.value.is_some() {
                        summary.totals.units += 1;
                    }
                }
                if summary.vacant.is_none() && row.as_ref().is_none_or(|r| r.reusable) {
                    summary.vacant = Some(i);
                }
            }
        }
        Contents::Branch(children) => {
            for (i, child) in children.iter().enumerate() {
                let vacant = if let Some(child) = child {
                    summary.records += child.summary.records;
                    summary.totals = summary
                        .totals
                        .checked_add(child.summary.totals)
                        .expect("reserved aggregate headroom");
                    summary.index = summary
                        .index
                        .checked_add(child.summary.index)
                        .expect("reserved index headroom");
                    child.summary.vacant
                } else {
                    Some(0)
                };
                if summary.vacant.is_none() {
                    summary.vacant = vacant.map(|v| (i << page.shift) + v);
                }
            }
        }
    }
    page.summary = summary;
}
fn refresh_path<T>(root: &mut Arc<Page<Slot<T>>>, index: usize) {
    let page = unique(root);
    if let Contents::Branch(children) = &mut page.contents
        && let Some(child) = &mut children[(index >> page.shift) & MASK]
    {
        refresh_path(child, index);
    }
    refresh(page);
}
impl<T> OwnedBank<T> {
    pub fn len(&self) -> usize {
        self.bank.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bank.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.bank.get(index)?.value.as_ref()
    }
    pub fn is_held(&self, index: usize) -> bool {
        self.bank.get(index).is_some_and(|slot| slot.held)
    }
    /// Owned-only subset count; never a canonical/payload unit receipt.
    pub fn active_count(&self) -> usize {
        self.bank.root.as_ref().map_or(0, |root| {
            #[cfg(test)]
            visited();
            root.summary.totals.units
        })
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.bank.extent {
            return None;
        }
        value_mut(slot_mut(self.bank.root.as_mut()?, index)?)
            .value
            .as_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.bank
            .iter()
            .filter_map(|(i, s)| s.value.as_ref().map(|v| (i, v)))
    }
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|(_, v)| v)
    }
    pub fn lowest_vacant(&self) -> Option<usize> {
        self.bank.lowest_vacant()
    }
    pub fn reserve<E: From<DomError>>(
        &self,
        index: usize,
        mut admit: impl FnMut(IndexCharge) -> Result<Option<crate::hosted::Retained>, E>,
    ) -> Result<EmptyPath<T>, E> {
        index
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let mut plan = EmptyPath {
            index,
            record: None,
            pages: std::array::from_fn(|_| None),
            charge: IndexCharge::default(),
            moved: IndexCharge::default(),
            minimum_shift: 0,
        };
        let mut count = 0;
        let mut add = |shift: u32, at: usize, plan: &mut EmptyPath<T>| -> Result<(), E> {
            let spare = empty_page(at, shift, &mut admit)?;
            plan.charge = plan.charge.checked_add(spare.charge)?;
            plan.pages[count] = Some(spare);
            count += 1;
            Ok(())
        };
        let old_shift = self.bank.root.as_ref().map(|r| {
            #[cfg(test)]
            visited();
            r.shift
        });
        let mut shift = old_shift.unwrap_or(0);
        while !fits(index, shift) {
            shift += BITS;
        }
        if let Some(old) = old_shift {
            for grow in ((old + BITS)..=shift).step_by(BITS as usize) {
                add(grow, 0, &mut plan)?;
            }
        }
        let mut current = self.bank.root.as_deref().filter(|p| fits(index, p.shift));
        let path_top = if current.is_none() && old_shift.is_some() && shift > old_shift.unwrap() {
            shift - BITS
        } else {
            shift
        };
        let mut at = path_top;
        loop {
            if current.is_none() {
                add(at, index, &mut plan)?;
            }
            if at == 0 {
                break;
            }
            current = current.and_then(|p| {
                #[cfg(test)]
                visited();
                match &p.contents {
                    Contents::Branch(children) => children[(index >> p.shift) & MASK].as_deref(),
                    Contents::Leaf(_) => None,
                }
            });
            at -= BITS;
        }
        if self.bank.entry(index).is_none() {
            let reserved = Record::<Slot<T>>::reserve()?;
            let bytes = size_of::<Slot<T>>();
            let producer = admit(IndexCharge { bytes, pages: 0 })?;
            plan.record = Some(reserved.materialize(Slot {
                value: None,
                held: false,
                active: true,
                _producer: producer,
            }));
            plan.charge.bytes = checked_add(plan.charge.bytes, bytes)?;
        }
        self.bank.index_charge().checked_add(plan.charge)?;
        if plan.record.is_some() {
            self.bank.totals().checked_add(Totals {
                bytes: size_of::<Slot<T>>(),
                units: 0,
                occurrences: 1,
            })?;
            self.bank
                .record_count()
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(plan)
    }
    pub fn insert(&mut self, mut plan: EmptyPath<T>, value: T, reusable: bool) -> IndexCharge {
        let index = plan.index;
        let mut shift = self
            .bank
            .root
            .as_ref()
            .map_or(plan.minimum_shift, |r| r.shift.max(plan.minimum_shift));
        while !fits(index, shift) {
            shift += BITS;
        }
        if self.bank.root.is_none() {
            self.bank.root = Some(plan.page(index, shift));
        } else {
            while self.bank.root.as_ref().unwrap().shift < shift {
                let next = self.bank.root.as_ref().unwrap().shift + BITS;
                let mut wrapper = plan.page(0, next);
                let Contents::Branch(children) = &mut unique(&mut wrapper).contents else {
                    unreachable!("growth branch")
                };
                children[0] = self.bank.root.take();
                refresh(unique(&mut wrapper));
                self.bank.root = Some(wrapper);
            }
        }
        fn put<T>(
            root: &mut Arc<Page<Slot<T>>>,
            index: usize,
            plan: &mut EmptyPath<T>,
            value: T,
            reusable: bool,
        ) {
            let page = unique(root);
            match &mut page.contents {
                Contents::Leaf(rows) => {
                    let entry = rows[index & MASK].get_or_insert_with(|| {
                        let record = plan
                            .record
                            .take()
                            .expect("empty destination record admitted");
                        plan.moved.bytes = checked_add(plan.moved.bytes, size_of::<Slot<T>>())
                            .expect("moved admitted record subset");
                        Entry {
                            value: record.value,
                            identity: record.identity,
                            reusable,
                            totals: Totals {
                                bytes: size_of::<Slot<T>>(),
                                units: 0,
                                occurrences: 1,
                            },
                        }
                    });
                    let slot = value_mut(entry);
                    assert!(
                        slot.value.is_none() && !slot.held,
                        "original vacant destination"
                    );
                    slot.value = Some(value);
                    slot.active = true;
                    entry.reusable = reusable;
                }
                Contents::Branch(children) => {
                    let child = &mut children[(index >> page.shift) & MASK];
                    if child.is_none() {
                        *child = Some(plan.page(index, page.shift - BITS));
                    }
                    put(child.as_mut().unwrap(), index, plan, value, reusable);
                }
            }
            refresh(page);
        }
        put(
            self.bank.root.as_mut().unwrap(),
            index,
            &mut plan,
            value,
            reusable,
        );
        self.bank.extent = self.bank.extent.max(index + 1);
        let moved = plan.moved;
        drop(plan); // Unused physical spares die before reported Source shrink.
        moved
    }
    /// Active membership is separate from vacancy and retained storage. A
    /// dormant row retains its value and every original receipt.
    pub fn set_active(&mut self, index: usize, active: bool) {
        let entry = slot_mut(self.bank.root.as_mut().expect("present active root"), index)
            .expect("present active row");
        value_mut(entry).active = active;
        refresh_path(self.bank.root.as_mut().unwrap(), index);
    }
    pub fn first_active(&self) -> Option<(usize, &T)> {
        fn first<T>(page: &Page<Slot<T>>, prefix: usize) -> Option<(usize, &T)> {
            #[cfg(test)]
            visited();
            if page.summary.totals.units == 0 {
                return None;
            }
            match &page.contents {
                Contents::Leaf(rows) => {
                    for (offset, entry) in rows.iter().enumerate() {
                        #[cfg(test)]
                        metric(Event::CanonicalRecordRead);
                        if let Some(entry) = entry {
                            let slot = &entry.value;
                            if slot.active
                                && !slot.held
                                && let Some(value) = &slot.value
                            {
                                return Some((prefix + offset, value));
                            }
                        }
                    }
                    None
                }
                Contents::Branch(children) => {
                    for (offset, child) in children.iter().enumerate() {
                        if let Some(child) = child {
                            #[cfg(test)]
                            visited();
                            if child.summary.totals.units != 0 {
                                return first(child, prefix + (offset << page.shift));
                            }
                        }
                    }
                    None
                }
            }
        }
        self.bank.root.as_deref().and_then(|root| first(root, 0))
    }
    pub fn set_reusable(&mut self, index: usize, reusable: bool) {
        let entry = slot_mut(self.bank.root.as_mut().expect("present slot root"), index)
            .expect("present slot");
        assert!(
            !value_mut(entry).held,
            "held slots are unavailable for reuse"
        );
        entry.reusable = reusable;
        refresh_path(self.bank.root.as_mut().unwrap(), index);
    }
    pub fn take(&mut self, index: usize) -> Option<Taken<T>> {
        if index >= self.bank.extent {
            return None;
        }
        let entry = slot_mut(self.bank.root.as_mut()?, index)?;
        let reusable = entry.reusable;
        let slot = value_mut(entry);
        let value = slot.value.take()?;
        slot.held = true;
        entry.reusable = false;
        refresh_path(self.bank.root.as_mut().unwrap(), index);
        Some(Taken {
            index,
            value,
            reusable,
        })
    }
    pub fn restore(&mut self, taken: Taken<T>) {
        let entry = slot_mut(self.bank.root.as_mut().expect("pinned root"), taken.index)
            .expect("pinned slot");
        let slot = value_mut(entry);
        assert!(slot.held && slot.value.is_none(), "same held slot");
        slot.value = Some(taken.value);
        slot.held = false;
        entry.reusable = taken.reusable;
        refresh_path(self.bank.root.as_mut().unwrap(), taken.index);
    }
    /// End the original held occurrence; destination generation policy remains
    /// with the owner. The empty retained slot is available for that policy.
    pub fn finish_taken(&mut self, taken: Taken<T>) -> T {
        let entry = slot_mut(self.bank.root.as_mut().expect("pinned root"), taken.index)
            .expect("pinned slot");
        let slot = value_mut(entry);
        assert!(slot.held && slot.value.is_none(), "same held slot");
        slot.held = false;
        entry.reusable = true;
        refresh_path(self.bank.root.as_mut().unwrap(), taken.index);
        taken.value
    }
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.bank.extent {
            return None;
        }
        let entry = slot_mut(self.bank.root.as_mut()?, index)?;
        let slot = value_mut(entry);
        assert!(!slot.held, "cannot retire another guard's held slot");
        let value = slot.value.take();
        entry.reusable = true;
        refresh_path(self.bank.root.as_mut().unwrap(), index);
        value
    }
}
/// Qualified native chunks have a fixed four-level (16-bit) address domain.
/// Unlike the stable role/row arena, this mode may permanently purge paths;
/// every reservation therefore owns a complete independent empty spine.
pub(crate) struct ChunkBank<T> {
    inner: OwnedBank<T>,
}
impl<T> Default for ChunkBank<T> {
    fn default() -> Self {
        Self {
            inner: OwnedBank::default(),
        }
    }
}
impl<T> ChunkBank<T> {
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }
    pub fn first_active(&self) -> Option<(usize, &T)> {
        self.inner.first_active()
    }
    pub fn active_count(&self) -> usize {
        self.inner.active_count()
    }
    pub fn retained_count(&self) -> usize {
        #[cfg(test)]
        if self.inner.bank.root.is_some() {
            visited();
        }
        self.inner.bank.record_count()
    }
    pub fn set_active(&mut self, index: usize, active: bool) {
        self.inner.set_active(index, active);
    }
    pub fn reserve<E: From<DomError>>(
        &self,
        index: u16,
        mut admit: impl FnMut(IndexCharge) -> Result<Option<crate::hosted::Retained>, E>,
    ) -> Result<EmptyPath<T>, E> {
        let index = usize::from(index);
        let top = (u16::BITS - 1) / BITS * BITS;
        let mut plan = EmptyPath {
            index,
            record: None,
            pages: std::array::from_fn(|_| None),
            charge: IndexCharge::default(),
            moved: IndexCharge::default(),
            minimum_shift: top,
        };
        for (offset, shift) in (0..=top).step_by(BITS as usize).enumerate() {
            let spare = empty_page(index, shift, &mut admit)?;
            plan.charge = plan.charge.checked_add(spare.charge)?;
            plan.pages[offset] = Some(spare);
        }
        let record = Record::<Slot<T>>::reserve()?;
        let bytes = size_of::<Slot<T>>();
        let producer = admit(IndexCharge { bytes, pages: 0 })?;
        plan.record = Some(record.materialize(Slot {
            value: None,
            held: false,
            active: true,
            _producer: producer,
        }));
        plan.charge.bytes = checked_add(plan.charge.bytes, bytes)?;
        self.inner.bank.index_charge().checked_add(plan.charge)?;
        self.inner.bank.totals().checked_add(Totals {
            bytes,
            units: 1,
            occurrences: 1,
        })?;
        Ok(plan)
    }
    pub fn insert(&mut self, plan: EmptyPath<T>, value: T) -> IndexCharge {
        self.inner.insert(plan, value, false)
    }
    /// Permanent retirement, never temporary take: a complete pending spine can
    /// reconstruct any purged prefix without allocation or old aliases.
    pub fn purge(&mut self, index: u16) {
        let index = usize::from(index);
        fn remove<T>(root: &mut Arc<Page<Slot<T>>>, index: usize) -> bool {
            #[cfg(test)]
            visited();
            let page = unique(root);
            match &mut page.contents {
                Contents::Leaf(rows) => {
                    drop(rows[index & MASK].take());
                }
                Contents::Branch(children) => {
                    let child = &mut children[(index >> page.shift) & MASK];
                    if child.as_mut().is_some_and(|child| remove(child, index)) {
                        drop(child.take());
                    }
                }
            }
            refresh(page);
            page.summary.records == 0
        }
        if self
            .inner
            .bank
            .root
            .as_mut()
            .is_some_and(|root| remove(root, index))
        {
            drop(self.inner.bank.root.take());
        }
    }
    #[cfg(test)]
    pub fn inspect_complete_reservation_bytes() -> usize {
        let mut leaf = Vec::<Option<Entry<Slot<T>>>>::new();
        leaf.try_reserve_exact(WIDTH).unwrap();
        let mut branch = Vec::<Option<Arc<Page<Slot<T>>>>>::new();
        branch.try_reserve_exact(WIDTH).unwrap();
        size_of::<Slot<T>>()
            + 4 * size_of::<Page<Slot<T>>>()
            + leaf.capacity() * size_of::<Option<Entry<Slot<T>>>>()
            + 3 * branch.capacity() * size_of::<Option<Arc<Page<Slot<T>>>>>()
    }
    #[cfg(test)]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.inner.values()
    }
    #[cfg(test)]
    pub fn inspect_storage_bytes(&self) -> usize {
        self.inner.inspect_storage_bytes()
    }
}
impl<T> std::ops::Index<usize> for ChunkBank<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.get(index).expect("present chunk")
    }
}
impl<T> std::ops::IndexMut<usize> for ChunkBank<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index).expect("present chunk")
    }
}

impl<T> std::ops::Index<usize> for OwnedBank<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        self.get(i).expect("present owned row")
    }
}
impl<T> std::ops::IndexMut<usize> for OwnedBank<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        self.get_mut(i).expect("present owned row")
    }
}

#[cfg(test)]
impl<T> EmptyPath<T> {
    pub fn inspect_storage_bytes(&self) -> usize {
        self.pages
            .iter()
            .flatten()
            .map(|p| {
                assert_eq!(Arc::strong_count(&p.page), 1);
                assert_eq!(Arc::weak_count(&p.page), 0);
                size_of::<Page<Slot<T>>>()
                    + match &p.page.contents {
                        Contents::Leaf(rows) => {
                            rows.capacity() * size_of::<Option<Entry<Slot<T>>>>()
                        }
                        Contents::Branch(children) => {
                            children.capacity() * size_of::<Option<Arc<Page<Slot<T>>>>>()
                        }
                    }
            })
            .sum::<usize>()
            + self.record.as_ref().map_or(0, |record| {
                assert_eq!(Arc::strong_count(&record.value), 1);
                assert_eq!(Arc::weak_count(&record.value), 0);
                size_of::<Slot<T>>()
            })
    }
}

#[cfg(test)]
impl<T> OwnedBank<T> {
    /// Raw first-leaf layout witness, independently allocated for exact admission tests.
    pub fn inspect_first_reservation_bytes() -> usize {
        let mut backing = Vec::<Option<Entry<Slot<T>>>>::new();
        backing.try_reserve_exact(WIDTH).unwrap();
        size_of::<Page<Slot<T>>>()
            + backing.capacity() * size_of::<Option<Entry<Slot<T>>>>()
            + size_of::<Slot<T>>()
    }
    pub fn inspect_leaf_width() -> usize {
        WIDTH
    }
    /// Independent raw enumeration; never reads cached Summary/IndexCharge.
    pub fn inspect_storage_bytes(&self) -> usize {
        fn walk<T>(page: &Arc<Page<Slot<T>>>) -> usize {
            assert_eq!(Arc::strong_count(page), 1, "exclusive Page");
            assert_eq!(Arc::weak_count(page), 0, "no Page registration");
            size_of::<Page<Slot<T>>>()
                + match &page.contents {
                    Contents::Leaf(rows) => {
                        rows.capacity() * size_of::<Option<Entry<Slot<T>>>>()
                            + rows
                                .iter()
                                .flatten()
                                .map(|row| {
                                    assert_eq!(
                                        Arc::strong_count(&row.value),
                                        1,
                                        "exclusive Entry.value"
                                    );
                                    assert_eq!(
                                        Arc::weak_count(&row.value),
                                        0,
                                        "no record registration"
                                    );
                                    size_of::<Slot<T>>()
                                })
                                .sum::<usize>()
                    }
                    Contents::Branch(children) => {
                        children.capacity() * size_of::<Option<Arc<Page<Slot<T>>>>>()
                            + children.iter().flatten().map(walk).sum::<usize>()
                    }
                }
        }
        self.bank.root.as_ref().map_or(0, walk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosted::{
        DomHostedError, DomHostedLimits, DomHostedRecordLimits, DomHostedSource, DomHostedSourceId,
    };
    use std::sync::atomic::AtomicBool;
    fn source() -> DomHostedSource {
        DomHostedSource::new(
            DomHostedSourceId {
                owner: crate::DomOwnerId::allocate().unwrap(),
                serial: 0,
            },
            DomHostedLimits::new(
                DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            &AtomicBool::new(false),
        )
        .unwrap()
    }
    #[test]
    fn inactive_members_remain_retained_and_held_slots_never_reenter_active_selection() {
        let source = source();
        let cancel = AtomicBool::new(false);
        let mut bank = OwnedBank::default();
        for index in [0, 65536] {
            let plan = bank
                .reserve(index, |c| source.retain(c.bytes, &cancel).map(Some))
                .unwrap();
            bank.insert(plan, index, false);
        }
        let bytes = bank.inspect_storage_bytes();
        bank.set_active(0, false);
        assert_eq!(bank.first_active().map(|(i, _)| i), Some(65536));
        assert_eq!(bank.values().copied().collect::<Vec<_>>(), vec![0, 65536]);
        assert_eq!(bank.bank.record_count(), 2);
        assert_eq!(bank.bank.totals().occurrences, 2);
        assert_eq!(bank.inspect_storage_bytes(), bytes);
        let taken = bank.take(65536).unwrap();
        assert!(bank.first_active().is_none());
        bank.restore(taken);
        assert_eq!(bank.first_active().map(|(i, _)| i), Some(65536));
        let taken = bank.take(65536).unwrap();
        assert_eq!(bank.finish_taken(taken), 65536);
        assert_eq!(bank.get(65536), None);
        assert!(bank.first_active().is_none());
        assert_eq!(bank.inspect_storage_bytes(), bytes);
        assert_eq!(bank.bank.record_count(), 2);
    }
    #[test]
    fn empty_reservation_does_not_alias_or_change_resident_values_and_late_growth_restores_held_slot()
     {
        let source = source();
        let cancel = AtomicBool::new(false);
        let before = source.retained_storage_bytes();
        let mut rows = OwnedBank::default();
        for (index, value) in [(0, 10usize), (3, 30)] {
            let plan = rows
                .reserve(index, |c| source.retain(c.bytes, &cancel).map(Some))
                .unwrap();
            rows.insert(plan, value, index == 0);
        }
        assert_eq!(
            source.retained_storage_bytes() - before,
            rows.inspect_storage_bytes()
        );
        let pending = rows
            .reserve(256, |c| source.retain(c.bytes, &cancel).map(Some))
            .unwrap();
        assert_eq!(rows.get(0), Some(&10));
        *rows.get_mut(3).unwrap() = 31;
        rows.inspect_storage_bytes();
        let taken = rows.take(0).unwrap();
        assert_eq!(*taken.value(), 10);
        assert_eq!(rows.get(0), None);
        assert_ne!(rows.lowest_vacant(), Some(0));
        rows.insert(pending, 256, false);
        rows.restore(taken);
        assert_eq!(rows.get(0), Some(&10));
        assert_eq!(rows.get(3), Some(&31));
        assert_eq!(rows.get(256), Some(&256));
        assert_eq!(
            source.retained_storage_bytes() - before,
            rows.inspect_storage_bytes()
        );
        drop(rows);
        assert_eq!(source.retained_storage_bytes(), before);
    }
    #[test]
    fn one_short_actual_empty_page_and_record_refuses_without_changing_owner_or_spent_work() {
        let mut source = source();
        let cancel = AtomicBool::new(false);
        let work = source.work(&cancel).unwrap();
        let before = source.retained_storage_bytes();
        let mut raw = Vec::<Option<Entry<Slot<usize>>>>::new();
        raw.try_reserve_exact(WIDTH).unwrap();
        let actual = size_of::<Page<Slot<usize>>>()
            + raw.capacity() * size_of::<Option<Entry<Slot<usize>>>>()
            + size_of::<Slot<usize>>();
        drop(raw);
        let policy = source.limits();
        source
            .set_limits(
                DomHostedLimits::new(
                    policy.records(),
                    before + actual - 1,
                    policy.scratch_bytes(),
                    policy.work(),
                ),
                &cancel,
            )
            .unwrap();
        let mut rows = OwnedBank::<usize>::default();
        let spent = work.spent.load(std::sync::atomic::Ordering::Relaxed);
        let result = rows.reserve(0, |charge| {
            work.step(&cancel)?;
            source.retain(charge.bytes, &cancel).map(Some)
        });
        assert!(matches!(
            result,
            Err(DomHostedError::Limit {
                resource: crate::hosted::DomHostedResource::RetainedBytes,
                ..
            })
        ));
        assert_eq!(source.retained_storage_bytes(), before);
        assert!(work.spent.load(std::sync::atomic::Ordering::Relaxed) > spent);
        assert_eq!(rows.len(), 0);
        source
            .set_limits(
                DomHostedLimits::new(
                    policy.records(),
                    before + actual,
                    policy.scratch_bytes(),
                    policy.work(),
                ),
                &cancel,
            )
            .unwrap();
        let plan = rows
            .reserve(0, |charge| {
                work.step(&cancel)?;
                source.retain(charge.bytes, &cancel).map(Some)
            })
            .unwrap();
        assert_eq!(plan.charge().bytes, actual);
        rows.insert(plan, 7, false);
        assert_eq!(rows.inspect_storage_bytes(), actual);
        assert_eq!(source.retained_storage_bytes(), before + actual);
        drop(rows);
        assert_eq!(source.retained_storage_bytes(), before);
    }
    #[test]
    fn cancelled_missing_path_releases_only_empty_new_storage_and_preserves_taken_value() {
        let source = source();
        let cancel = AtomicBool::new(false);
        let work = source.work(&cancel).unwrap();
        let mut rows = OwnedBank::default();
        let plan = rows
            .reserve(0, |c| source.retain(c.bytes, &cancel).map(Some))
            .unwrap();
        rows.insert(plan, 99usize, false);
        let taken = rows.take(0).unwrap();
        let before = source.retained_storage_bytes();
        let mut calls = 0;
        let result = rows.reserve(65536, |charge| {
            calls += 1;
            if calls == 2 {
                cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            work.step(&cancel)?;
            source.retain(charge.bytes, &cancel).map(Some)
        });
        assert!(matches!(
            result,
            Err(DomHostedError::Dom(DomError::Cancelled))
        ));
        assert_eq!(source.retained_storage_bytes(), before);
        assert_eq!(rows.get(0), None);
        rows.restore(taken);
        assert_eq!(rows.get(0), Some(&99));
        rows.inspect_storage_bytes();
        assert_eq!(source.retained_storage_bytes(), before);
    }
    #[test]
    fn complete_chunk_reservation_survives_prefix_retirement_and_releases_unused_spares() {
        let source = source();
        let cancel = AtomicBool::new(false);
        let before = source.retained_storage_bytes();
        let mut rows = ChunkBank::default();
        for (index, value) in [(0x1234, 10usize), (0x1235, 20)] {
            let plan = rows
                .reserve(index, |c| source.retain(c.bytes, &cancel).map(Some))
                .unwrap();
            rows.insert(plan, value);
        }
        rows.set_active(0x1234, false);
        let pending = rows
            .reserve(0x1236, |c| source.retain(c.bytes, &cancel).map(Some))
            .unwrap();
        let pending_bytes = pending.inspect_storage_bytes();
        rows.purge(0x1235);
        assert_eq!(rows.retained_count(), 1);
        assert_eq!(rows.get(0x1234), Some(&10));
        assert!(rows.first_active().is_none());
        assert_eq!(
            source.retained_storage_bytes() - before,
            rows.inspect_storage_bytes() + pending_bytes
        );
        rows.purge(0x1234);
        assert_eq!(rows.inspect_storage_bytes(), 0);
        assert_eq!(source.retained_storage_bytes() - before, pending_bytes);
        let moved = rows.insert(pending, 30);
        assert_eq!(rows.get(0x1236), Some(&30));
        assert_eq!(moved.bytes, rows.inspect_storage_bytes());
        assert_eq!(
            source.retained_storage_bytes() - before,
            rows.inspect_storage_bytes()
        );
        let old = rows.inspect_storage_bytes();
        let pending = rows
            .reserve(0x1237, |c| source.retain(c.bytes, &cancel).map(Some))
            .unwrap();
        let reserved = pending.inspect_storage_bytes();
        let moved = rows.insert(pending, 40);
        assert!(
            moved.bytes < reserved,
            "existing prefix needs only its new record"
        );
        assert_eq!(rows.inspect_storage_bytes(), old + moved.bytes);
        assert_eq!(
            source.retained_storage_bytes() - before,
            rows.inspect_storage_bytes()
        );
        rows.purge(0x1236);
        assert_eq!(rows.get(0x1237), Some(&40));
        rows.purge(0x1237);
        assert_eq!(rows.inspect_storage_bytes(), 0);
        assert_eq!(source.retained_storage_bytes(), before);
    }
    #[test]
    fn complete_chunk_raw_boundary_refuses_one_short_and_reclaims_on_final_purge() {
        let mut source = source();
        let cancel = AtomicBool::new(false);
        let before = source.retained_storage_bytes();
        let raw = ChunkBank::<usize>::inspect_complete_reservation_bytes();
        let policy = source.limits();
        let mut rows = ChunkBank::<usize>::default();
        for (limit, success) in [(before + raw - 1, false), (before + raw, true)] {
            source
                .set_limits(
                    DomHostedLimits::new(
                        policy.records(),
                        limit,
                        policy.scratch_bytes(),
                        policy.work(),
                    ),
                    &cancel,
                )
                .unwrap();
            let result = rows.reserve(u16::MAX, |c| source.retain(c.bytes, &cancel).map(Some));
            if success {
                let plan = result.unwrap();
                assert_eq!(plan.inspect_storage_bytes(), raw);
                assert_eq!(source.retained_storage_bytes(), before + raw);
                assert_eq!(rows.insert(plan, 7).bytes, raw);
                assert_eq!(rows.get(usize::from(u16::MAX)), Some(&7));
                rows.purge(u16::MAX);
            } else {
                assert!(matches!(
                    result,
                    Err(DomHostedError::Limit {
                        resource: crate::hosted::DomHostedResource::RetainedBytes,
                        ..
                    })
                ));
            }
            assert_eq!(source.retained_storage_bytes(), before);
            assert_eq!(rows.inspect_storage_bytes(), 0);
        }
    }
}

//! Shared finite storage policy for the two concrete value owners.

use super::*;

const BUCKETS: usize = 32;
const CANDIDATES: usize = 4;
const PAGE: usize = 8;
const PAGES: usize = BUCKETS * CANDIDATES / PAGE;
const MARKS: usize = 16;
const READ_BATCH: usize = 32;
pub(super) const MAX_UNITS: usize = 8192;
pub(super) const MAX_ATTRIBUTES: usize = 64;
const MAX_PAYLOAD_BYTES: usize = 256 * 1024;
const MAX_METADATA_BYTES: usize = 256 * 1024;

pub(super) trait Value: Sized {
    type Item;
    fn shared(&self) -> &Arc<Vec<Self::Item>>;
    fn from_shared(value: Arc<Vec<Self::Item>>) -> Self;
    fn eligible(&self) -> bool;
    fn read(&self, cursor: &mut Cursor) -> Result<Read, HtmlValuePoolError>;
    fn candidate_cleanup_work() -> HtmlValuePoolWork;
}
struct Entry<T: Value> {
    weak: Weak<Vec<T::Item>>,
    hash: u64,
    payload: HtmlValuePoolUsage,
    generation: u64,
}
struct Slot<T: Value> {
    entry: Option<Entry<T>>,
}
#[cfg(test)]
impl<T: Value> Drop for Slot<T> {
    fn drop(&mut self) {
        tests::observe_slot_drop();
    }
}
#[cfg(test)]
impl<T: Value> Drop for Entry<T> {
    fn drop(&mut self) {
        tests::observe_entry_drop();
    }
}
// A checkpoint keeps only the first value displaced from each slot. Repeated
// writes in that checkpoint retire the superseded current Weak immediately.
enum Before<T: Value> {
    Untouched,
    Saved(Option<Entry<T>>),
}
struct UndoPage<T: Value> {
    cells: Vec<Before<T>>,
    cursors: [Option<usize>; PAGE / CANDIDATES],
}
struct Frame<T: Value> {
    before_pages: u16,
    // Empty checkpoints retain only this thin header. The directory and each
    // selected page are formed lazily under the mutation's actual admission.
    pages: Vec<Option<UndoPage<T>>>,
    hot_before: u128,
    hot_touched: u128,
}
impl<T: Value> Frame<T> {
    fn new(before_pages: u16) -> Self {
        Self {
            before_pages,
            pages: Vec::new(),
            hot_before: 0,
            hot_touched: 0,
        }
    }
    fn page(&self, index: usize) -> Option<&UndoPage<T>> {
        self.pages.get(index).and_then(Option::as_ref)
    }
    fn remember_hot(&mut self, mask: u128, hot: u128) {
        let first = mask & !self.hot_touched;
        self.hot_before = (self.hot_before & !first) | (hot & first);
        self.hot_touched |= mask;
    }
}
struct Mark<T: Value> {
    id: u64,
    frame: Frame<T>,
}
struct Root<T: Value> {
    id: u64,
    frozen: bool,
    suspended: bool,
    base: Frame<T>,
    marks: [Option<Mark<T>>; MARKS],
    current: Option<usize>,
}
impl<T: Value> Root<T> {
    fn frame(&self) -> &Frame<T> {
        self.current.map_or(&self.base, |index| {
            &self.marks[index].as_ref().expect("current mark").frame
        })
    }
    fn frame_mut(&mut self) -> &mut Frame<T> {
        match self.current {
            Some(index) => &mut self.marks[index].as_mut().expect("current mark").frame,
            None => &mut self.base,
        }
    }
    fn frame_id(&self) -> u64 {
        self.current.map_or(self.id, |index| {
            self.marks[index].as_ref().expect("current mark").id
        })
    }
    fn refresh_current(&mut self) {
        self.current = self
            .marks
            .iter()
            .enumerate()
            .filter_map(|(index, mark)| mark.as_ref().map(|mark| (mark.id, index)))
            .max()
            .map(|(_, index)| index);
    }
}
struct Prepared<T: Value> {
    entry: Entry<T>,
    page: Option<Vec<Slot<T>>>,
    undo_page: Option<UndoPage<T>>,
    directory: Option<Vec<Option<UndoPage<T>>>>,
    frame: u64,
    usage: HtmlValuePoolUsage,
}

pub(super) struct Pool<T: Value> {
    owner: Option<u64>,
    next: u64,
    disabled: bool,
    root: Option<Root<T>>,
    action: Option<(u64, u64)>,
    operation: Option<u64>,
    pages: [Option<Vec<Slot<T>>>; PAGES],
    cursors: [usize; BUCKETS],
    page_bits: u16,
    hot: u128,
    usage: HtmlValuePoolUsage,
    metadata_limit: usize,
    #[cfg(test)]
    collision: bool,
    #[cfg(test)]
    refuse_checkpoint_reserve: bool,
}
pub(super) struct Operation<'a, T: Value> {
    input: &'a T,
    owner: u64,
    session: Option<u64>,
    id: u64,
    serial: u64,
    offer: bool,
    pub(super) phase: Phase,
    hash: u64,
    payload: HtmlValuePoolUsage,
    left: Cursor,
    right: Cursor,
    candidate_index: usize,
    slot: usize,
    generation: Option<u64>,
    vacant: Option<usize>,
    clear_hot: u128,
    prepared: Option<Prepared<T>>,
    pub(super) candidate: Option<T>,
    candidate_usage: HtmlValuePoolUsage,
    pub(super) change: HtmlValuePoolChange,
}
impl<T: Value> Pool<T> {
    pub(super) fn new() -> Self {
        let owner = owner_id();
        Self {
            owner,
            next: 1,
            disabled: owner.is_none(),
            root: None,
            action: None,
            operation: None,
            pages: std::array::from_fn(|_| None),
            cursors: [0; BUCKETS],
            page_bits: 0,
            hot: 0,
            usage: HtmlValuePoolUsage::default(),
            metadata_limit: MAX_METADATA_BYTES,
            #[cfg(test)]
            collision: false,
            #[cfg(test)]
            refuse_checkpoint_reserve: false,
        }
    }
    pub(super) fn usage(&self) -> HtmlValuePoolUsage {
        self.usage
    }
    fn issue(&mut self) -> Option<u64> {
        if self.disabled {
            return None;
        }
        let id = self.next;
        let Some(next) = id.checked_add(1) else {
            self.disabled = true;
            return None;
        };
        self.next = next;
        Some(id)
    }
    fn session(&self) -> Option<u64> {
        self.root.as_ref().map(|root| root.id)
    }
    fn entry(&self, slot: usize) -> Option<&Entry<T>> {
        self.pages[slot / PAGE]
            .as_ref()
            .and_then(|page| page[slot % PAGE].entry.as_ref())
    }
    fn entry_mut(&mut self, slot: usize) -> &mut Option<Entry<T>> {
        &mut self.pages[slot / PAGE].as_mut().expect("prepared page")[slot % PAGE].entry
    }
    fn occurrence(&self, slot: usize, entry: &Entry<T>) -> HtmlValuePoolOccurrence {
        HtmlValuePoolOccurrence {
            owner: self.owner.expect("active owner"),
            slot,
            generation: entry.generation,
        }
    }
    fn target_bytes() -> usize {
        size_of::<Vec<T::Item>>()
    }
    fn metadata_delta(
        page_capacity: usize,
        undo_capacity: usize,
        directory_capacity: usize,
    ) -> Result<usize, HtmlValuePoolError> {
        page_capacity
            .checked_mul(size_of::<Slot<T>>())
            .and_then(|n| {
                undo_capacity
                    .checked_mul(size_of::<Before<T>>())
                    .and_then(|m| n.checked_add(m))
            })
            .and_then(|n| {
                directory_capacity
                    .checked_mul(size_of::<Option<UndoPage<T>>>())
                    .and_then(|m| n.checked_add(m))
            })
            .and_then(|n| n.checked_add(Self::target_bytes()))
            .ok_or(HtmlValuePoolError::RepresentationExhausted)
    }
    fn valid_operation(&self, op: &Operation<'_, T>) -> Result<bool, HtmlValuePoolError> {
        if self.owner != Some(op.owner) {
            return Err(HtmlValuePoolError::ForeignOwner);
        }
        Ok(op.session == self.session() && self.operation == Some(op.id))
    }
    pub(super) fn operation<'a>(&mut self, input: &'a T, offer: bool) -> Operation<'a, T> {
        let id = self.issue();
        self.operation = id;
        Operation {
            input,
            owner: self.owner.unwrap_or(0),
            session: self.session(),
            id: id.unwrap_or(0),
            serial: 0,
            offer,
            phase: if id.is_some() {
                Phase::Hash
            } else {
                Phase::Bypassed
            },
            hash: 0xcbf2_9ce4_8422_2325,
            payload: HtmlValuePoolUsage::default(),
            left: Cursor::default(),
            right: Cursor::default(),
            candidate_index: 0,
            slot: 0,
            generation: None,
            vacant: None,
            clear_hot: 0,
            prepared: None,
            candidate: None,
            candidate_usage: HtmlValuePoolUsage::default(),
            change: HtmlValuePoolChange::default(),
        }
    }
    pub(super) fn offer_miss(
        &mut self,
        op: &mut Operation<'_, T>,
    ) -> Result<bool, HtmlValuePoolError> {
        // Completed results grant no mutation authority. In particular an
        // optional owner-ID exhaustion already produced Bypassed (owner 0),
        // and immutable Hits survive later operation supersession.
        if matches!(op.phase, Phase::Hit | Phase::Offered | Phase::Bypassed) {
            return if op.owner == self.owner.unwrap_or(0) {
                Ok(false)
            } else {
                Err(HtmlValuePoolError::ForeignOwner)
            };
        }
        if !self.valid_operation(op)? {
            op.bypass();
            return Ok(false);
        }
        if op.phase != Phase::Miss {
            return Err(HtmlValuePoolError::NotReady);
        }
        op.offer = true;
        op.phase = Phase::Prepare;
        Ok(true)
    }
    fn remember_hot(&mut self, mask: u128) {
        if let Some(root) = &mut self.root {
            root.frame_mut().remember_hot(mask, self.hot);
        }
    }
    pub(super) fn next(
        &mut self,
        op: &mut Operation<'_, T>,
    ) -> Result<Option<HtmlValuePoolAction>, HtmlValuePoolError> {
        if matches!(
            op.phase,
            Phase::Hit | Phase::Miss | Phase::Offered | Phase::Bypassed
        ) {
            return Ok(None);
        }
        if !self.valid_operation(op)? {
            op.bypass();
            return Ok(None);
        }
        #[cfg(test)]
        if self.collision && matches!(op.phase, Phase::Inspect | Phase::Prepare) {
            op.hash = 0;
        }
        if op.phase == Phase::Touch
            && !self
                .root
                .as_ref()
                .is_some_and(|root| !root.frozen && !root.suspended)
        {
            // The acquired immutable value is still a Hit. Retention hints are
            // mutations and require the same active authority as insertion.
            op.phase = Phase::Hit;
            return Ok(None);
        }
        let Some(serial) = self.issue() else {
            op.bypass();
            return Ok(None);
        };
        op.serial = serial;
        let mut layout = HtmlValuePoolLayout {
            pool: self.usage,
            candidate: op.candidate_usage,
            ..HtmlValuePoolLayout::default()
        };
        let reads = match op.phase {
            Phase::Hash => read_count(op.left),
            Phase::Compare if op.left.units && op.right.units => {
                read_count(op.left).min(read_count(op.right))
            }
            _ => 1,
        };
        match op.phase {
            Phase::Hash => layout.work = read_work(op.left, reads),
            Phase::Compare => {
                let left = read_work(op.left, reads);
                let right = read_work(op.right, reads);
                layout.work = HtmlValuePoolWork {
                    record_visits: left.record_visits + right.record_visits,
                    inspected_bytes: left.inspected_bytes + right.inspected_bytes,
                    ..HtmlValuePoolWork::default()
                };
            }
            Phase::Touch => {
                // An active root journals this bounded retention hint.
                layout.work.record_visits = 1;
                layout.closing_work.record_visits = 1;
            }
            Phase::Inspect => {
                if op.candidate_index == CANDIDATES {
                    op.phase = if op.offer {
                        Phase::Prepare
                    } else {
                        Phase::Miss
                    };
                    return self.next(op);
                }
                op.slot = bucket(op.hash) * CANDIDATES + op.candidate_index;
                op.generation = self.entry(op.slot).map(|entry| entry.generation);
                layout.work.record_visits = 1;
            }
            Phase::Upgrade => {
                let Some(entry) = self
                    .entry(op.slot)
                    .filter(|entry| Some(entry.generation) == op.generation)
                else {
                    op.bypass();
                    return Ok(None);
                };
                layout.candidate = entry.payload;
                layout.work.record_visits = 1;
                layout.work.provider_calls = 1;
                // The upgraded alias may become the last strong owner at any
                // later comparison, error, abandonment or output transfer.
                layout.closing_work = T::candidate_cleanup_work();
            }
            Phase::Prepare => {
                let Some(root) = &self.root else {
                    op.bypass();
                    return Ok(None);
                };
                if root.frozen || root.suspended {
                    op.bypass();
                    return Ok(None);
                }
                op.clear_hot = 0;
                let mut scanned = 0;
                op.slot = if let Some(vacant) = op.vacant {
                    vacant
                } else {
                    let start = self.cursors[bucket(op.hash)];
                    let mut selected = bucket(op.hash) * CANDIDATES + start;
                    for offset in 0..CANDIDATES {
                        let slot = bucket(op.hash) * CANDIDATES + (start + offset) % CANDIDATES;
                        scanned += 1;
                        if self.hot & (1 << slot) == 0 {
                            selected = slot;
                            break;
                        }
                        op.clear_hot |= 1 << slot;
                    }
                    selected
                };
                op.generation = self.entry(op.slot).map(|entry| entry.generation);
                let page = if self.pages[op.slot / PAGE].is_none() {
                    PAGE
                } else {
                    0
                };
                let undo = if root.frame().page(op.slot / PAGE).is_none() {
                    PAGE
                } else {
                    0
                };
                let directory = if root.frame().pages.is_empty() {
                    PAGES
                } else {
                    0
                };
                layout.pool = self.usage.add(HtmlValuePoolUsage {
                    bytes: Self::metadata_delta(page, undo, directory)?,
                    utf16_units: 0,
                })?;
                if layout.pool.bytes > self.metadata_limit {
                    op.bypass();
                    return Ok(None);
                }
                layout.work.copied_bytes = page * size_of::<Slot<T>>()
                    + undo * size_of::<Before<T>>()
                    + directory * size_of::<Option<UndoPage<T>>>();
                layout.work.record_visits = 1 + scanned;
                layout.work.provider_calls = 1
                    + usize::from(page != 0)
                    + usize::from(undo != 0)
                    + usize::from(directory != 0);
                layout.work.reserve_bytes = page * size_of::<Slot<T>>()
                    + undo * size_of::<Before<T>>()
                    + directory * size_of::<Option<UndoPage<T>>>();
                // Charge before either reserve: if the second reserve fails,
                // the initialized page already needs destruction. On success
                // the same allowance follows this Weak/page/checkpoint backing through their
                // provisional or committed owner until they are destroyed.
                layout.closing_work = HtmlValuePoolWork {
                    record_visits: 1 + page + (undo + directory) * (MARKS + 2),
                    provider_calls: 1
                        + usize::from(page != 0)
                        + (MARKS + 2) * (usize::from(undo != 0) + usize::from(directory != 0)),
                    copied_bytes: (MARKS + 1)
                        * (usize::from(undo != 0) * size_of::<Option<UndoPage<T>>>()
                            + usize::from(directory != 0) * size_of::<Vec<Option<UndoPage<T>>>>()),
                    ..HtmlValuePoolWork::default()
                };
            }
            Phase::Commit => {
                layout.pool = op.prepared.as_ref().expect("prepared mutation").usage;
                layout.work.copied_bytes = size_of::<Before<T>>() + size_of::<Slot<T>>();
                // A first-before cell can move through at most MARKS older
                // surviving checkpoints before restoration or final seal.
                // Pay those actual cell visits/moves and possible Weak drops
                // now; closure never requests work from a stopped consumer.
                layout.closing_work = HtmlValuePoolWork {
                    record_visits: 2 * (MARKS + 1),
                    provider_calls: MARKS + 2,
                    copied_bytes: (MARKS + 1) * (size_of::<Before<T>>() + size_of::<Slot<T>>()),
                    ..HtmlValuePoolWork::default()
                };
            }
            _ => unreachable!("terminal actions handled above"),
        }
        self.action = Some((op.id, serial));
        Ok(Some(HtmlValuePoolAction {
            owner: op.owner,
            session: op.session,
            operation: op.id,
            serial,
            slot: op.slot,
            generation: op.generation,
            phase: op.phase,
            reads,
            layout,
        }))
    }
    pub(super) fn perform(
        &mut self,
        op: &mut Operation<'_, T>,
        action: HtmlValuePoolAction,
    ) -> Result<HtmlValuePoolProgress, HtmlValuePoolError> {
        if !op.check(&action)? {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        if action.kind() != HtmlValuePoolActionKind::Pool {
            return Err(HtmlValuePoolError::WrongAction);
        }
        if self.owner != Some(action.owner) {
            return Err(HtmlValuePoolError::ForeignOwner);
        }
        if self.action != Some((action.operation, action.serial))
            || action.session != self.session()
            || self.operation != Some(op.id)
        {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        self.action = None;
        if self.entry(action.slot).map(|entry| entry.generation) != action.generation {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        match action.phase {
            Phase::Inspect => {
                match self.entry(action.slot) {
                    Some(entry) if entry.hash == op.hash => op.phase = Phase::Upgrade,
                    Some(_) => op.advance_candidate(),
                    None => {
                        op.vacant.get_or_insert(action.slot);
                        op.advance_candidate();
                    }
                }
                Ok(HtmlValuePoolProgress::Continue)
            }
            Phase::Upgrade => {
                let entry = self.entry(action.slot).expect("qualified entry");
                let Some(shared) = entry.weak.upgrade() else {
                    op.vacant.get_or_insert(action.slot);
                    op.advance_candidate();
                    return Ok(HtmlValuePoolProgress::CandidateReleased);
                };
                let same = Arc::ptr_eq(&shared, op.input.shared());
                op.candidate_usage = entry.payload;
                op.candidate = Some(T::from_shared(shared));
                op.left = Cursor::default();
                op.right = Cursor::default();
                op.phase = if same { Phase::Touch } else { Phase::Compare };
                Ok(HtmlValuePoolProgress::CandidateAcquired)
            }
            Phase::Touch => {
                if self
                    .root
                    .as_ref()
                    .is_some_and(|root| !root.frozen && !root.suspended)
                {
                    self.remember_hot(1 << action.slot);
                    self.hot |= 1 << action.slot;
                }
                op.phase = Phase::Hit;
                Ok(HtmlValuePoolProgress::Continue)
            }
            Phase::Prepare => self.prepare(op),
            Phase::Commit => self.commit(op),
            _ => Err(HtmlValuePoolError::WrongAction),
        }
    }
    fn prepare(
        &mut self,
        op: &mut Operation<'_, T>,
    ) -> Result<HtmlValuePoolProgress, HtmlValuePoolError> {
        let Some(generation) = self.issue() else {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        };
        let Some(root) = &self.root else {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        };
        if root.frozen || root.suspended {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        let page = if self.pages[op.slot / PAGE].is_none() {
            let mut page = Vec::new();
            if page.try_reserve_exact(PAGE).is_err() {
                op.bypass();
                return Ok(HtmlValuePoolProgress::Finished);
            }
            for _ in 0..PAGE {
                page.push(Slot { entry: None });
            }
            Some(page)
        } else {
            None
        };
        let frame = root.frame_id();
        let directory = if root.frame().pages.is_empty() {
            let mut values = Vec::new();
            if values.try_reserve_exact(PAGES).is_err() {
                op.bypass();
                return Ok(HtmlValuePoolProgress::Finished);
            }
            values.resize_with(PAGES, || None);
            Some(values)
        } else {
            None
        };
        let undo_page = if root.frame().page(op.slot / PAGE).is_none() {
            let mut values = Vec::new();
            // Capacity-refusal injection follows real Slot/directory initialization.
            #[cfg(test)]
            let requested = if self.refuse_checkpoint_reserve {
                usize::MAX
            } else {
                PAGE
            };
            #[cfg(not(test))]
            let requested = PAGE;
            if values.try_reserve_exact(requested).is_err() {
                op.bypass();
                return Ok(HtmlValuePoolProgress::Finished);
            }
            for _ in 0..PAGE {
                values.push(Before::Untouched);
            }
            Some(UndoPage {
                cells: values,
                cursors: [None; PAGE / CANDIDATES],
            })
        } else {
            None
        };
        let usage = self.usage.add(HtmlValuePoolUsage {
            bytes: Self::metadata_delta(
                page.as_ref().map_or(0, Vec::capacity),
                undo_page.as_ref().map_or(0, |page| page.cells.capacity()),
                directory.as_ref().map_or(0, Vec::capacity),
            )?,
            utf16_units: 0,
        })?;
        // No strong alias is created here. The actual Weak occurrence and known
        // target allocation space were included in the preparation layout.
        op.prepared = Some(Prepared {
            entry: Entry {
                weak: Arc::downgrade(op.input.shared()),
                hash: op.hash,
                payload: op.payload,
                generation,
            },
            page,
            undo_page,
            directory,
            frame,
            usage,
        });
        op.phase = Phase::Commit;
        Ok(HtmlValuePoolProgress::Continue)
    }
    fn commit(
        &mut self,
        op: &mut Operation<'_, T>,
    ) -> Result<HtmlValuePoolProgress, HtmlValuePoolError> {
        let Some(root) = &self.root else {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        };
        if root.frozen || root.suspended {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        let prepared = op.prepared.take().expect("commit has prepared owner");
        if prepared.usage.bytes > self.metadata_limit {
            drop(prepared);
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        if root.frame_id() != prepared.frame {
            drop(prepared);
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        let before = self.usage;
        let page_index = op.slot / PAGE;
        if let Some(page) = prepared.page {
            self.pages[page_index] = Some(page);
            self.page_bits |= 1 << page_index;
        }
        let inserted = self.occurrence(op.slot, &prepared.entry);
        let old = self.entry_mut(op.slot).replace(prepared.entry);
        let old_occurrence = old.as_ref().map(|entry| self.occurrence(op.slot, entry));
        let bucket = bucket(op.hash);
        let changed_hot = op.clear_hot | (1 << op.slot);
        self.remember_hot(changed_hot);
        self.hot &= !changed_hot;
        let frame = self.root.as_mut().expect("active root").frame_mut();
        if let Some(directory) = prepared.directory {
            frame.pages = directory;
        }
        if let Some(page) = prepared.undo_page {
            frame.pages[page_index] = Some(page);
        }
        let undo_page = frame.pages[page_index]
            .as_mut()
            .expect("prepared before page");
        let cell = &mut undo_page.cells[op.slot % PAGE];
        let first = matches!(cell, Before::Untouched);
        let released = usize::from(!first && old.is_some());
        if first {
            *cell = Before::Saved(old);
        } else {
            drop(old);
        }
        undo_page.cursors[bucket % (PAGE / CANDIDATES)].get_or_insert(self.cursors[bucket]);
        self.cursors[bucket] = (op.slot % CANDIDATES + 1) % CANDIDATES;
        self.usage = prepared.usage;
        self.usage.bytes -= released * Self::target_bytes();
        op.change = HtmlValuePoolChange {
            before,
            after: self.usage,
            inserted: Some(inserted),
            moved_to_undo: if first { old_occurrence } else { None },
            released,
        };
        op.phase = Phase::Offered;
        Ok(HtmlValuePoolProgress::Finished)
    }
    pub(super) fn begin_root(&mut self) -> Option<HtmlValuePoolRoot> {
        if self.root.is_some() {
            return None;
        }
        let id = self.issue()?;
        self.root = Some(Root {
            id,
            frozen: false,
            suspended: false,
            base: Frame::new(self.page_bits),
            marks: std::array::from_fn(|_| None),
            current: None,
        });
        self.action = None;
        self.operation = None;
        Some(HtmlValuePoolRoot {
            owner: self.owner?,
            id,
        })
    }
    fn root_matches(&self, owner: u64, id: u64) -> Result<bool, HtmlValuePoolError> {
        if self.owner != Some(owner) {
            return Err(HtmlValuePoolError::ForeignOwner);
        }
        Ok(self.session() == Some(id))
    }
    pub(super) fn mark(
        &mut self,
        token: &HtmlValuePoolRoot,
    ) -> Result<Option<HtmlValuePoolMark>, HtmlValuePoolError> {
        if !self.root_matches(token.owner, token.id)? {
            return Ok(None);
        }
        let root = self.root.as_ref().expect("matching root");
        if root.frozen || root.suspended {
            return Ok(None);
        }
        let index = root.marks.iter().position(Option::is_none);
        let id = index.and_then(|_| self.issue());
        let root = self.root.as_mut().expect("authority survives exhaustion");
        let (Some(index), Some(id)) = (index, id) else {
            root.suspended = true;
            self.action = None;
            self.operation = None;
            return Ok(None);
        };
        root.marks[index] = Some(Mark {
            id,
            frame: Frame::new(self.page_bits),
        });
        root.current = Some(index);
        self.action = None;
        self.operation = None;
        Ok(Some(HtmlValuePoolMark {
            owner: token.owner,
            root: token.id,
            id,
        }))
    }
    pub(super) fn accept(&mut self, token: HtmlValuePoolMark) -> Result<(), HtmlValuePoolError> {
        if !self.root_matches(token.owner, token.root)? {
            return Ok(());
        }
        let root = self.root.as_mut().expect("matching root");
        let Some(index) = root
            .marks
            .iter()
            .position(|mark| mark.as_ref().is_some_and(|mark| mark.id == token.id))
        else {
            return Ok(());
        };
        if root.frozen {
            return Err(HtmlValuePoolError::NotReady);
        }
        let mut source = root.marks[index].take().expect("selected mark").frame;
        let older = root
            .marks
            .iter()
            .enumerate()
            .filter_map(|(i, mark)| {
                mark.as_ref()
                    .filter(|mark| mark.id < token.id)
                    .map(|mark| (mark.id, i))
            })
            .max()
            .map(|(_, i)| i);
        let destination = match older {
            Some(i) => &mut root.marks[i].as_mut().expect("older mark").frame,
            None => &mut root.base,
        };
        let new_hot = source.hot_touched & !destination.hot_touched;
        destination.hot_before =
            (destination.hot_before & !new_hot) | (source.hot_before & new_hot);
        destination.hot_touched |= source.hot_touched;
        if destination.pages.is_empty() {
            destination.pages = std::mem::take(&mut source.pages);
        } else {
            for (source_page, destination_page) in
                source.pages.iter_mut().zip(&mut destination.pages)
            {
                let Some(mut page) = source_page.take() else {
                    continue;
                };
                if let Some(destination) = destination_page {
                    for (source, destination) in page.cells.iter_mut().zip(&mut destination.cells) {
                        if matches!(destination, Before::Untouched) {
                            *destination = std::mem::replace(source, Before::Untouched);
                        } else if matches!(source, Before::Saved(Some(_))) {
                            self.usage.bytes -= Self::target_bytes();
                        }
                    }
                    for (source, destination) in
                        page.cursors.into_iter().zip(&mut destination.cursors)
                    {
                        if destination.is_none() {
                            *destination = source;
                        }
                    }
                    self.usage.bytes -= page.cells.capacity() * size_of::<Before<T>>();
                    drop(page);
                } else {
                    *destination_page = Some(page);
                }
            }
            self.usage.bytes -= source.pages.capacity() * size_of::<Option<UndoPage<T>>>();
        }
        root.refresh_current();
        self.action = None;
        self.operation = None;
        Ok(())
    }
    pub(super) fn suspend(&mut self, token: &HtmlValuePoolRoot) -> Result<(), HtmlValuePoolError> {
        if self.root_matches(token.owner, token.id)? {
            let root = self.root.as_mut().expect("matching root");
            if root.frozen {
                return Err(HtmlValuePoolError::NotReady);
            }
            root.suspended = true;
            self.action = None;
            self.operation = None;
        }
        Ok(())
    }
    pub(super) fn restore(
        &mut self,
        token: HtmlValuePoolMark,
    ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
        if !self.root_matches(token.owner, token.root)? {
            return Ok(self.unchanged());
        }
        let root = self.root.as_ref().expect("matching root");
        if !root.marks.iter().flatten().any(|mark| mark.id == token.id) {
            return Ok(self.unchanged());
        }
        if root.frozen {
            return Err(HtmlValuePoolError::NotReady);
        }
        let before = self.usage;
        let mut released = 0;
        loop {
            let root = self.root.as_mut().expect("root retained");
            let Some(index) = root
                .marks
                .iter()
                .enumerate()
                .filter_map(|(i, mark)| {
                    mark.as_ref()
                        .filter(|mark| mark.id >= token.id)
                        .map(|mark| (mark.id, i))
                })
                .max()
                .map(|(_, i)| i)
            else {
                break;
            };
            let frame = root.marks[index].take().expect("selected mark").frame;
            released += self.rollback_frame(frame);
        }
        self.root.as_mut().expect("root retained").refresh_current();
        self.action = None;
        self.operation = None;
        Ok(HtmlValuePoolChange {
            before,
            after: self.usage,
            released,
            ..HtmlValuePoolChange::default()
        })
    }
    fn unchanged(&self) -> HtmlValuePoolChange {
        HtmlValuePoolChange {
            before: self.usage,
            after: self.usage,
            ..HtmlValuePoolChange::default()
        }
    }
    fn rollback_frame(&mut self, mut frame: Frame<T>) -> usize {
        let mut released = 0;
        for (index, saved) in frame.pages.iter_mut().enumerate() {
            let Some(mut page) = saved.take() else {
                continue;
            };
            for (offset, cell) in page.cells.iter_mut().enumerate() {
                if let Before::Saved(old) = std::mem::replace(cell, Before::Untouched) {
                    let current = std::mem::replace(self.entry_mut(index * PAGE + offset), old);
                    if current.is_some() {
                        drop(current);
                        released += 1;
                        self.usage.bytes -= Self::target_bytes();
                    }
                }
            }
            self.usage.bytes -= page.cells.capacity() * size_of::<Before<T>>();
            for (offset, saved) in page.cursors.into_iter().enumerate() {
                if let Some(saved) = saved {
                    self.cursors[index * (PAGE / CANDIDATES) + offset] = saved;
                }
            }
            drop(page);
            if frame.before_pages & (1 << index) == 0 {
                let page = self.pages[index].take().expect("introduced page");
                self.usage.bytes -= page.capacity() * size_of::<Slot<T>>();
                self.page_bits &= !(1 << index);
                drop(page);
            }
        }
        self.usage.bytes -= frame.pages.capacity() * size_of::<Option<UndoPage<T>>>();
        self.hot = (self.hot & !frame.hot_touched) | (frame.hot_before & frame.hot_touched);
        released
    }
    fn discard_frame(&mut self, mut frame: Frame<T>) -> usize {
        let mut released = 0;
        for page in frame.pages.iter_mut().filter_map(Option::take) {
            released += page
                .cells
                .iter()
                .filter(|cell| matches!(cell, Before::Saved(Some(_))))
                .count();
            self.usage.bytes -= page.cells.capacity() * size_of::<Before<T>>();
            drop(page);
        }
        self.usage.bytes -= frame.pages.capacity() * size_of::<Option<UndoPage<T>>>();
        self.usage.bytes -= released * Self::target_bytes();
        released
    }
    pub(super) fn abort(
        &mut self,
        token: HtmlValuePoolRoot,
    ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
        if !self.root_matches(token.owner, token.id)? {
            return Ok(self.unchanged());
        }
        let before = self.usage;
        let mut released = 0;
        loop {
            let root = self.root.as_mut().expect("matching root");
            let Some(index) = root
                .marks
                .iter()
                .enumerate()
                .filter_map(|(i, mark)| mark.as_ref().map(|mark| (mark.id, i)))
                .max()
                .map(|(_, i)| i)
            else {
                break;
            };
            let frame = root.marks[index].take().expect("selected mark").frame;
            released += self.rollback_frame(frame);
        }
        let root = self.root.take().expect("matching root");
        released += self.rollback_frame(root.base);
        self.action = None;
        self.operation = None;
        Ok(HtmlValuePoolChange {
            before,
            after: self.usage,
            released,
            ..HtmlValuePoolChange::default()
        })
    }
    pub(super) fn prepare_seal(
        &mut self,
        token: HtmlValuePoolRoot,
    ) -> Result<HtmlValuePoolSeal, (HtmlValuePoolError, HtmlValuePoolRoot)> {
        match self.root_matches(token.owner, token.id) {
            Ok(true)
                if self
                    .root
                    .as_ref()
                    .is_some_and(|root| root.marks.iter().all(Option::is_none) && !root.frozen) =>
            {
                self.root.as_mut().expect("matching root").frozen = true;
                self.action = None;
                self.operation = None;
                Ok(HtmlValuePoolSeal { root: token })
            }
            Ok(_) => Err((HtmlValuePoolError::NotReady, token)),
            Err(error) => Err((error, token)),
        }
    }
    pub(super) fn validate_seal(&self, seal: &HtmlValuePoolSeal) -> Result<(), HtmlValuePoolError> {
        if self.root_matches(seal.root.owner, seal.root.id)?
            && self.root.as_ref().is_some_and(|root| root.frozen)
        {
            Ok(())
        } else {
            Err(HtmlValuePoolError::NotReady)
        }
    }
    pub(super) fn seal(
        &mut self,
        seal: HtmlValuePoolSeal,
    ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
        self.validate_seal(&seal)?;
        let before = self.usage;
        let root = self.root.take().expect("validated seal");
        let released = self.discard_frame(root.base);
        self.action = None;
        self.operation = None;
        Ok(HtmlValuePoolChange {
            before,
            after: self.usage,
            released,
            ..HtmlValuePoolChange::default()
        })
    }
}

impl<T: Value> Operation<'_, T> {
    fn check(&self, action: &HtmlValuePoolAction) -> Result<bool, HtmlValuePoolError> {
        if action.owner != self.owner {
            return Err(HtmlValuePoolError::ForeignOwner);
        }
        if action.operation != self.id {
            return Err(HtmlValuePoolError::WrongAction);
        }
        if action.serial < self.serial {
            return Ok(false);
        }
        if action.serial != self.serial || action.phase != self.phase {
            return Err(HtmlValuePoolError::WrongAction);
        }
        Ok(true)
    }
    fn bypass(&mut self) {
        // Data owners are destroyed before the caller releases the action's
        // separately held reservation. Input remains borrowed and untouched.
        self.prepared = None;
        self.candidate = None;
        self.candidate_usage = HtmlValuePoolUsage::default();
        self.phase = Phase::Bypassed;
    }
    fn advance_candidate(&mut self) {
        self.candidate_index += 1;
        self.phase = Phase::Inspect;
    }
    pub(super) fn read(
        &mut self,
        action: HtmlValuePoolAction,
    ) -> Result<HtmlValuePoolProgress, HtmlValuePoolError> {
        if !self.check(&action)? {
            self.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        match action.phase {
            Phase::Hash => {
                #[cfg(test)]
                if !self.left.started {
                    tests::observe_execution_eligibility_check();
                }
                if !self.left.started && !self.input.eligible() {
                    #[cfg(test)]
                    tests::observe_execution_eligibility_bypass();
                    self.bypass();
                    return Ok(HtmlValuePoolProgress::Finished);
                }
                for _ in 0..action.reads {
                    #[cfg(test)]
                    let unit = self.left.units;
                    let read = self.input.read(&mut self.left)?;
                    #[cfg(test)]
                    tests::observe_execution_hash(unit);
                    self.payload = self.payload.add(read.payload)?;
                    if self.payload.utf16_units > MAX_UNITS
                        || self.payload.bytes > MAX_PAYLOAD_BYTES
                    {
                        #[cfg(test)]
                        tests::observe_execution_payload_bypass();
                        self.bypass();
                        return Ok(HtmlValuePoolProgress::Finished);
                    }
                    for byte in std::iter::once(read.word.tag).chain(read.word.value.to_le_bytes())
                    {
                        self.hash ^= u64::from(byte);
                        self.hash = self.hash.wrapping_mul(0x100_0000_01b3);
                    }
                }
                if self.left.done {
                    self.phase = Phase::Inspect;
                }
                Ok(HtmlValuePoolProgress::Continue)
            }
            Phase::Compare => {
                let mut equal = true;
                // Execute the entire length-qualified prepaid run, including
                // its suffix after a mismatch: inspected-byte work is literal.
                for _ in 0..action.reads {
                    #[cfg(test)]
                    let left_unit = self.left.units;
                    let left = self.input.read(&mut self.left)?;
                    #[cfg(test)]
                    tests::observe_execution_compare_input(left_unit);
                    #[cfg(test)]
                    let right_unit = self.right.units;
                    let right = self
                        .candidate
                        .as_ref()
                        .expect("admitted candidate")
                        .read(&mut self.right)?;
                    #[cfg(test)]
                    {
                        tests::observe_execution_compare_candidate(right_unit);
                        tests::observe_execution_compare_pair(left_unit, right_unit);
                    }
                    equal &= left.word == right.word;
                }
                if !equal {
                    self.candidate = None;
                    self.candidate_usage = HtmlValuePoolUsage::default();
                    self.advance_candidate();
                    return Ok(HtmlValuePoolProgress::CandidateReleased);
                }
                if self.left.done && self.right.done {
                    self.phase = Phase::Touch;
                }
                Ok(HtmlValuePoolProgress::Continue)
            }
            _ => Err(HtmlValuePoolError::WrongAction),
        }
    }
}
fn read_count(cursor: Cursor) -> usize {
    if cursor.units {
        (cursor.len - cursor.offset).min(READ_BATCH)
    } else {
        1
    }
}
fn read_work(cursor: Cursor, reads: usize) -> HtmlValuePoolWork {
    if cursor.units {
        HtmlValuePoolWork {
            inspected_bytes: reads * size_of::<u16>(),
            ..HtmlValuePoolWork::default()
        }
    } else {
        HtmlValuePoolWork {
            record_visits: 1,
            ..HtmlValuePoolWork::default()
        }
    }
}
fn bucket(hash: u64) -> usize {
    hash as usize % BUCKETS
}
pub(super) fn root_work<T: Value>() -> HtmlValuePoolWork {
    HtmlValuePoolWork {
        record_visits: (MARKS + 1) * (PAGES + BUCKETS + 1),
        copied_bytes: 2 * size_of::<Root<T>>(),
        ..HtmlValuePoolWork::default()
    }
}
pub(super) fn seal_work() -> HtmlValuePoolWork {
    // Preparation visits at most MARKS readiness cells plus the four fixed
    // owner/root/readiness/freeze-and-invalidation metadata stages. It wraps
    // existing root authority; it neither copies a Root nor allocates backing.
    // root_work already prepays fixed closing/header moves, and each mutation
    // prepays its variable inverse/storage work. A refused preparation keeps
    // its root unchanged; another attempt must pay these readiness visits anew.
    HtmlValuePoolWork {
        record_visits: MARKS + 4,
        ..HtmlValuePoolWork::default()
    }
}
pub(super) fn disposal_work() -> HtmlValuePoolWork {
    let slots = PAGES * PAGE;
    let saved = slots * (MARKS + 1);
    // Initialized live Slots and first-before cells, plus each possible Weak.
    // Capacity is storage only; no spare capacity elements are traversed.
    HtmlValuePoolWork {
        record_visits: 2 * (slots + saved) + (MARKS + 1) * (PAGES + BUCKETS + 1),
        provider_calls: slots + saved + PAGES * (MARKS + 2) + MARKS + 1,
        ..HtmlValuePoolWork::default()
    }
}
pub(super) fn mark_work<T: Value>() -> HtmlValuePoolWork {
    HtmlValuePoolWork {
        record_visits: 3 * MARKS + (MARKS + 1) * (PAGES + BUCKETS),
        copied_bytes: (MARKS + 1) * size_of::<Mark<T>>(),
        ..HtmlValuePoolWork::default()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "retention_tests.rs"]
mod retention_tests;

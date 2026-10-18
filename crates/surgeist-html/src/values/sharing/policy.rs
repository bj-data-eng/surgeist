//! Shared finite storage policy for the two concrete value owners.

use super::*;

const BUCKETS: usize = 32;
const CANDIDATES: usize = 4;
const PAGE: usize = 8;
const PAGES: usize = BUCKETS * CANDIDATES / PAGE;
const LOG: usize = 128;
const MARKS: usize = 16;
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
struct Undo<T: Value> {
    slot: usize,
    old: Option<Entry<T>>,
    cursor: usize,
    new_page: bool,
}
#[derive(Clone, Copy)]
struct Mark {
    id: u64,
    len: usize,
    had_log: bool,
}
struct Root {
    id: u64,
    frozen: bool,
    suspended: bool,
    had_log: bool,
    marks: [Option<Mark>; MARKS],
}
struct Prepared<T: Value> {
    entry: Entry<T>,
    page: Option<Vec<Slot<T>>>,
    log: Option<Vec<Undo<T>>>,
    usage: HtmlValuePoolUsage,
}

pub(super) struct Pool<T: Value> {
    owner: Option<u64>,
    next: u64,
    disabled: bool,
    root: Option<Root>,
    action: Option<(u64, u64)>,
    operation: Option<u64>,
    pages: [Option<Vec<Slot<T>>>; PAGES],
    cursors: [usize; BUCKETS],
    log: Vec<Undo<T>>,
    usage: HtmlValuePoolUsage,
    metadata_limit: usize,
    #[cfg(test)]
    collision: bool,
    #[cfg(test)]
    refuse_log_reserve: bool,
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
            log: Vec::new(),
            usage: HtmlValuePoolUsage::default(),
            metadata_limit: MAX_METADATA_BYTES,
            #[cfg(test)]
            collision: false,
            #[cfg(test)]
            refuse_log_reserve: false,
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
        log_capacity: usize,
    ) -> Result<usize, HtmlValuePoolError> {
        page_capacity
            .checked_mul(size_of::<Slot<T>>())
            .and_then(|n| {
                log_capacity
                    .checked_mul(size_of::<Undo<T>>())
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
            prepared: None,
            candidate: None,
            candidate_usage: HtmlValuePoolUsage::default(),
            change: HtmlValuePoolChange::default(),
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
        match op.phase {
            Phase::Hash => layout.work = read_work(op.left),
            Phase::Compare => {
                let left = read_work(op.left);
                let right = read_work(op.right);
                layout.work = HtmlValuePoolWork {
                    record_visits: left.record_visits + right.record_visits,
                    inspected_bytes: left.inspected_bytes + right.inspected_bytes,
                    copied_bytes: 0,
                    ..HtmlValuePoolWork::default()
                };
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
                if root.frozen || root.suspended || self.log.len() == LOG {
                    op.bypass();
                    return Ok(None);
                }
                op.slot = op.vacant.unwrap_or_else(|| {
                    bucket(op.hash) * CANDIDATES + self.cursors[bucket(op.hash)]
                });
                op.generation = self.entry(op.slot).map(|entry| entry.generation);
                let page = if self.pages[op.slot / PAGE].is_none() {
                    PAGE
                } else {
                    0
                };
                let log = if self.log.capacity() == 0 { LOG } else { 0 };
                layout.pool = self.usage.add(HtmlValuePoolUsage {
                    bytes: Self::metadata_delta(page, log)?,
                    utf16_units: 0,
                })?;
                if layout.pool.bytes > self.metadata_limit {
                    op.bypass();
                    return Ok(None);
                }
                layout.work.copied_bytes = page * size_of::<Slot<T>>();
                layout.work.record_visits = 1;
                layout.work.provider_calls = 1 + usize::from(page != 0) + usize::from(log != 0);
                layout.work.reserve_bytes =
                    page * size_of::<Slot<T>>() + log * size_of::<Undo<T>>();
                // Charge before either reserve: if the second reserve fails,
                // the initialized page already needs destruction. On success
                // the same allowance follows this Weak/page/log through their
                // provisional or committed owner until they are destroyed.
                layout.closing_work = HtmlValuePoolWork {
                    record_visits: 1 + page,
                    provider_calls: 1 + usize::from(page != 0) + usize::from(log != 0),
                    ..HtmlValuePoolWork::default()
                };
            }
            Phase::Commit => {
                layout.pool = op.prepared.as_ref().expect("prepared mutation").usage;
                layout.work.copied_bytes = size_of::<Undo<T>>() + size_of::<Slot<T>>();
                layout.closing_work = HtmlValuePoolWork {
                    record_visits: 1,
                    copied_bytes: size_of::<Undo<T>>() + size_of::<Slot<T>>(),
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
                op.phase = if same { Phase::Hit } else { Phase::Compare };
                Ok(HtmlValuePoolProgress::CandidateAcquired)
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
        if root.frozen || root.suspended || self.log.len() == LOG {
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
        let log = if self.log.capacity() == 0 {
            let mut log = Vec::new();
            // The private test controller produces std's capacity error after
            // the real page was initialized; it does not simulate host OOM.
            #[cfg(test)]
            let requested = if self.refuse_log_reserve {
                usize::MAX
            } else {
                LOG
            };
            #[cfg(not(test))]
            let requested = LOG;
            if log.try_reserve_exact(requested).is_err() {
                op.bypass();
                return Ok(HtmlValuePoolProgress::Finished);
            }
            Some(log)
        } else {
            None
        };
        let usage = self.usage.add(HtmlValuePoolUsage {
            bytes: Self::metadata_delta(
                page.as_ref().map_or(0, Vec::capacity),
                log.as_ref().map_or(0, Vec::capacity),
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
            log,
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
        if root.frozen || root.suspended || self.log.len() == LOG {
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        let prepared = op.prepared.take().expect("commit has prepared owner");
        if prepared.usage.bytes > self.metadata_limit {
            drop(prepared);
            op.bypass();
            return Ok(HtmlValuePoolProgress::Finished);
        }
        let before = self.usage;
        let new_page = prepared.page.is_some();
        if let Some(page) = prepared.page {
            self.pages[op.slot / PAGE] = Some(page);
        }
        if let Some(log) = prepared.log {
            self.log = log;
        }
        let inserted = self.occurrence(op.slot, &prepared.entry);
        let old = self.entry_mut(op.slot).replace(prepared.entry);
        let moved_to_undo = old.as_ref().map(|entry| self.occurrence(op.slot, entry));
        let bucket = bucket(op.hash);
        self.log.push(Undo {
            slot: op.slot,
            old,
            cursor: self.cursors[bucket],
            new_page,
        });
        self.cursors[bucket] = (op.slot % CANDIDATES + 1) % CANDIDATES;
        self.usage = prepared.usage;
        op.change = HtmlValuePoolChange {
            before,
            after: self.usage,
            inserted: Some(inserted),
            moved_to_undo,
            released: 0,
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
            had_log: self.log.capacity() != 0,
            marks: [None; MARKS],
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
        let root = self
            .root
            .as_mut()
            .expect("existing authority survives exhaustion");
        let (Some(index), Some(id)) = (index, id) else {
            root.suspended = true;
            return Ok(None);
        };
        root.marks[index] = Some(Mark {
            id,
            len: self.log.len(),
            had_log: self.log.capacity() != 0,
        });
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
        if let Some(index) = root
            .marks
            .iter()
            .position(|mark| mark.is_some_and(|mark| mark.id == token.id))
        {
            root.marks[index] = None;
        }
        Ok(())
    }
    pub(super) fn suspend(&mut self, token: &HtmlValuePoolRoot) -> Result<(), HtmlValuePoolError> {
        if self.root_matches(token.owner, token.id)? {
            self.root.as_mut().expect("matching root").suspended = true;
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
        let Some(mark) = self
            .root
            .as_ref()
            .expect("matching root")
            .marks
            .iter()
            .flatten()
            .find(|mark| mark.id == token.id)
            .copied()
        else {
            return Ok(self.unchanged());
        };
        let mut change = self.rollback(mark.len);
        if !mark.had_log {
            let log = std::mem::take(&mut self.log);
            let bytes = log.capacity() * size_of::<Undo<T>>();
            drop(log);
            self.usage.bytes -= bytes;
            change.after = self.usage;
        }
        for mark in &mut self.root.as_mut().expect("root retained").marks {
            if mark.is_some_and(|mark| mark.id >= token.id) {
                *mark = None;
            }
        }
        self.action = None;
        self.operation = None;
        Ok(change)
    }
    fn unchanged(&self) -> HtmlValuePoolChange {
        HtmlValuePoolChange {
            before: self.usage,
            after: self.usage,
            ..HtmlValuePoolChange::default()
        }
    }
    fn rollback(&mut self, len: usize) -> HtmlValuePoolChange {
        let before = self.usage;
        let mut released = 0;
        while self.log.len() > len {
            let Undo {
                slot,
                old,
                cursor,
                new_page,
            } = self.log.pop().expect("known suffix");
            let current = std::mem::replace(self.entry_mut(slot), old);
            if current.is_some() {
                drop(current);
                released += 1;
                self.usage.bytes -= Self::target_bytes();
            }
            self.cursors[slot / CANDIDATES] = cursor;
            if new_page {
                let page = self.pages[slot / PAGE].take().expect("introduced page");
                let bytes = page.capacity() * size_of::<Slot<T>>();
                drop(page);
                self.usage.bytes -= bytes;
            }
        }
        HtmlValuePoolChange {
            before,
            after: self.usage,
            released,
            ..HtmlValuePoolChange::default()
        }
    }
    pub(super) fn abort(
        &mut self,
        token: HtmlValuePoolRoot,
    ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
        if !self.root_matches(token.owner, token.id)? {
            return Ok(self.unchanged());
        }
        let before = self.usage;
        let mut change = self.rollback(0);
        let root = self.root.take().expect("matching root");
        if !root.had_log {
            let log = std::mem::take(&mut self.log);
            let bytes = log.capacity() * size_of::<Undo<T>>();
            drop(log);
            self.usage.bytes -= bytes;
        }
        self.action = None;
        self.operation = None;
        change.before = before;
        change.after = self.usage;
        Ok(change)
    }
    pub(super) fn prepare_seal(
        &mut self,
        token: HtmlValuePoolRoot,
    ) -> Result<HtmlValuePoolSeal, (HtmlValuePoolError, HtmlValuePoolRoot)> {
        match self.root_matches(token.owner, token.id) {
            Ok(true) => {
                self.root.as_mut().expect("matching root").frozen = true;
                self.action = None;
                Ok(HtmlValuePoolSeal { root: token })
            }
            Ok(false) => Err((HtmlValuePoolError::NotReady, token)),
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
        let mut released = 0;
        while let Some(undo) = self.log.pop() {
            if undo.old.is_some() {
                released += 1;
            }
            drop(undo);
        }
        self.usage.bytes -= released * Self::target_bytes();
        self.root = None;
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
                if !self.left.started && !self.input.eligible() {
                    self.bypass();
                    return Ok(HtmlValuePoolProgress::Finished);
                }
                let read = self.input.read(&mut self.left)?;
                self.payload = self.payload.add(read.payload)?;
                if self.payload.utf16_units > MAX_UNITS || self.payload.bytes > MAX_PAYLOAD_BYTES {
                    self.bypass();
                    return Ok(HtmlValuePoolProgress::Finished);
                }
                for byte in std::iter::once(read.word.tag).chain(read.word.value.to_le_bytes()) {
                    self.hash ^= u64::from(byte);
                    self.hash = self.hash.wrapping_mul(0x100_0000_01b3);
                }
                if self.left.done {
                    self.phase = Phase::Inspect;
                }
                Ok(HtmlValuePoolProgress::Continue)
            }
            Phase::Compare => {
                let left = self.input.read(&mut self.left)?;
                let right = self
                    .candidate
                    .as_ref()
                    .expect("admitted candidate")
                    .read(&mut self.right)?;
                if left.word != right.word {
                    self.candidate = None;
                    self.candidate_usage = HtmlValuePoolUsage::default();
                    self.advance_candidate();
                    return Ok(HtmlValuePoolProgress::CandidateReleased);
                }
                if self.left.done && self.right.done {
                    self.phase = Phase::Hit;
                }
                Ok(HtmlValuePoolProgress::Continue)
            }
            _ => Err(HtmlValuePoolError::WrongAction),
        }
    }
}
fn read_work(cursor: Cursor) -> HtmlValuePoolWork {
    if cursor.units {
        HtmlValuePoolWork {
            inspected_bytes: size_of::<u16>(),
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
pub(super) fn root_work() -> HtmlValuePoolWork {
    HtmlValuePoolWork {
        record_visits: 2,
        copied_bytes: 2 * size_of::<Root>(),
        ..HtmlValuePoolWork::default()
    }
}
pub(super) fn disposal_work() -> HtmlValuePoolWork {
    let slots = PAGES * PAGE;
    // Final Drop traverses initialized Slots and Undo records, each retaining
    // at most one Entry/Weak. Spare Vec capacity is not initialized work. Each
    // Weak can release one target allocation; each nonempty backing can free
    // once. No live payload or element buffer is retained by a Weak.
    HtmlValuePoolWork {
        record_visits: 2 * (slots + LOG),
        provider_calls: slots + LOG + PAGES + 1,
        ..HtmlValuePoolWork::default()
    }
}
pub(super) fn mark_work() -> HtmlValuePoolWork {
    HtmlValuePoolWork {
        record_visits: 3 * MARKS,
        copied_bytes: (MARKS + 1) * size_of::<Option<Mark>>(),
        ..HtmlValuePoolWork::default()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "retention_tests.rs"]
mod retention_tests;

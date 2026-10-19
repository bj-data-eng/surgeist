//! Optional, owner-local immutable value reuse. No semantic identity lives here.
//!
//! Every callback runs outside the operational mutex. A session owns only weak
//! cleanup authority, and an occupied root makes a second session bypass.
#[cfg(test)]
use super::scale_metrics::Event;
use super::{DomError, custody};
use crate::name::AttributeStorage;
use crate::{DocumentString, DomAttribute};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use surgeist_html::bounded::{
    HtmlStringPool, HtmlValuePoolMark, HtmlValuePoolRoot, HtmlValuePoolSeal,
};

mod banks;
#[cfg(test)]
pub(in crate::store) mod fixture;
mod strings;
#[cfg(test)]
mod tests;

/// Original-ledger admission and the original cumulative work owner. Neither
/// callback is retained or invoked while holding the pool mutex.
pub(in crate::store) struct Admission<'a> {
    pub global: &'a mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    pub step: &'a mut dyn FnMut() -> Result<(), DomError>,
}
impl Admission<'_> {
    fn work(&mut self, amount: usize) -> Result<(), DomError> {
        for _ in 0..amount {
            (self.step)()?;
        }
        Ok(())
    }
}

pub(in crate::store) struct Owner(Arc<Mutex<State>>);
struct State {
    owner: crate::DomOwnerId,
    strings: HtmlStringPool,
    banks: banks::Banks,
    roots: Option<Roots>,
    ledger: custody::Ledger,
    // All represented data precedes its original affine credit.
    storage: custody::Debit,
}
struct Roots {
    id: u64,
    strings: HtmlValuePoolRoot,
    marks: usize,
    suspended: bool,
}

/// Non-clone affine root authority. No canonical State or strong Owner handle.
pub(in crate::store) struct Session {
    owner: Weak<Mutex<State>>,
    phase: Mutex<Phase>,
    _authority: Option<custody::Debit>,
}
#[derive(Clone, Copy)]
enum Phase {
    Dormant,
    Acquiring,
    Active(u64),
    Bypassed,
}
// The phase is published before callbacks, but no phase/owner guard survives
// one. Errors and unwinds restore the cold attachment without poisoning a
// mutex or retaining a partial root. No fresh authority is needed to abort.
struct Acquisition<'a> {
    session: &'a Session,
    partial: Option<u64>,
    complete: bool,
}
impl Acquisition<'_> {
    fn finish(mut self, phase: Phase) {
        *self.session.phase.lock().expect("session phase") = phase;
        self.complete = true;
    }
}
impl Drop for Acquisition<'_> {
    fn drop(&mut self) {
        if self.complete {
            return;
        }
        if let (Some(owner), Some(id)) = (self.session.owner.upgrade(), self.partial) {
            let mut state = lock(&owner);
            if state.active(id) {
                state.abort(id);
            } else {
                state.banks.abort(id);
            }
        }
        *self.session.phase.lock().expect("session phase") = Phase::Dormant;
    }
}
struct Marks {
    strings: HtmlValuePoolMark,
    bank: banks::Mark,
}
pub(in crate::store) struct Checkpoint {
    owner: Weak<Mutex<State>>,
    root: Option<u64>,
    marks: Mutex<Option<Marks>>,
    _authority: Option<custody::Debit>,
}
/// Checked existing authority and prepaid closing work; no future allocator or
/// qualifier issue is hidden behind this value.
pub(in crate::store) struct CloseReady {
    owner: usize,
    root: Option<u64>,
    marks: usize,
}
pub(in crate::store) struct Prepared {
    owner: Weak<Mutex<State>>,
    root: Option<u64>,
    strings: Option<HtmlValuePoolSeal>,
    _authority: Option<custody::Debit>,
}

pub(in crate::store) enum BankProbe {
    Hit(SelectedBank),
    Miss(BankMiss),
    Bypassed,
}
/// A miss is only a qualified optional offer optimization, never semantic proof.
pub(in crate::store) struct BankMiss {
    owner: crate::DomOwnerId,
    root: u64,
    sequence: u64,
    hash: u64,
}
pub(in crate::store) struct SelectedBank(custody::Held<AttributeStorage>);
impl SelectedBank {
    pub(in crate::store) fn storage(&self) -> &AttributeStorage {
        self.0.value()
    }
    pub(in crate::store) fn into_held(self) -> custody::Held<AttributeStorage> {
        self.0
    }
}
pub(in crate::store) struct SelectedString(custody::Held<DocumentString>);
impl SelectedString {
    pub(in crate::store) fn value(&self) -> &DocumentString {
        self.0.value()
    }
    pub(in crate::store) fn into_held(self) -> custody::Held<DocumentString> {
        self.0
    }
}

fn lock(owner: &Mutex<State>) -> MutexGuard<'_, State> {
    owner.lock().expect("exclusive optional value owner")
}
fn protocol<T>(result: Result<T, surgeist_html::bounded::HtmlValuePoolError>) -> T {
    result.expect("owned optional value protocol")
}
fn html_work(work: surgeist_html::bounded::HtmlValuePoolWork) -> Result<usize, DomError> {
    [
        work.record_visits(),
        work.inspected_bytes(),
        work.copied_bytes(),
        work.provider_calls(),
        work.reserve_bytes(),
    ]
    .into_iter()
    .try_fold(0usize, |sum, n| {
        sum.checked_add(n).ok_or(DomError::RepresentationExhausted)
    })
}
fn html_usage(value: surgeist_html::bounded::HtmlValuePoolUsage) -> custody::Usage {
    custody::Usage::new(value.bytes(), value.utf16_units())
}
#[cfg(test)]
fn metric(event: Event) {
    if super::scale_metrics::ENABLED {
        super::scale_metrics::record(event);
    }
}
#[cfg(test)]
fn admitted_html(work: surgeist_html::bounded::HtmlValuePoolWork, closing: bool) {
    if super::scale_metrics::ENABLED {
        metric(Event::SharingHtmlAdmitted {
            closing,
            coordinates: [
                work.record_visits(),
                work.inspected_bytes(),
                work.copied_bytes(),
                work.provider_calls(),
                work.reserve_bytes(),
            ],
        });
    }
}
impl State {
    fn active(&self, id: u64) -> bool {
        self.roots.as_ref().is_some_and(|root| root.id == id)
    }
    fn mutable(&self, id: u64) -> bool {
        self.roots
            .as_ref()
            .is_some_and(|root| root.id == id && !root.suspended)
            && !self.banks.frozen()
    }
    fn usage(&self) -> Result<custody::Usage, DomError> {
        custody::Usage::new(size_of::<Mutex<State>>(), 0)
            .checked_add(html_usage(self.strings.heap_usage()))?
            .checked_add(self.banks.usage())
    }
    fn reconcile(&mut self) {
        let excess = self
            .storage
            .usage()
            .checked_sub(
                self.usage()
                    .expect("actual bounded optional storage extent"),
            )
            .expect("all actual optional storage was preadmitted");
        drop(
            self.storage
                .split(excess)
                .expect("owned optional storage subset"),
        );
    }
    fn abort(&mut self, id: u64) {
        if !self.active(id) {
            return;
        }
        let root = self.roots.take().expect("matching root");
        protocol(self.strings.abort_root(root.strings));
        self.banks.abort(id);
        #[cfg(test)]
        metric(Event::SharingPairAborted);
        self.reconcile();
    }
}

impl Owner {
    pub(in crate::store) fn new(
        owner: crate::DomOwnerId,
        ledger: custody::Ledger,
        admission: &mut Admission<'_>,
    ) -> Result<Self, DomError> {
        admission.work(html_work(HtmlStringPool::disposal_work())?)?;
        #[cfg(test)]
        admitted_html(HtmlStringPool::disposal_work(), true);
        admission.work(banks::DISPOSAL_WORK)?;
        let storage = ledger.retain(
            custody::Usage::new(size_of::<Mutex<State>>(), 0),
            &mut *admission.global,
        )?;
        let result = Self(Arc::new(Mutex::new(State {
            owner,
            strings: HtmlStringPool::new(),
            banks: banks::Banks::new(),
            roots: None,
            ledger,
            storage,
        })));
        #[cfg(test)]
        allocation(size_of::<Mutex<State>>());
        Ok(result)
    }
    pub(in crate::store) fn attach(
        &self,
        admission: &mut Admission<'_>,
    ) -> Result<Session, DomError> {
        let bypass = || Session {
            owner: Weak::new(),
            phase: Mutex::new(Phase::Bypassed),
            _authority: None,
        };
        if lock(&self.0).banks.occupied() {
            return Ok(bypass());
        }
        let ledger = lock(&self.0).ledger.clone();
        let authority = ledger.retain(
            custody::Usage::new(size_of::<Mutex<State>>(), 0),
            &mut *admission.global,
        )?;
        if lock(&self.0).banks.occupied() {
            return Ok(bypass());
        }
        Ok(Session {
            owner: Arc::downgrade(&self.0),
            phase: Mutex::new(Phase::Dormant),
            _authority: Some(authority),
        })
    }
}

impl Session {
    #[cfg(test)]
    pub(in crate::store) fn fixture_strings_enabled(&self) -> bool {
        fixture::layers(self) == fixture::Layers::Combined
    }

    fn active_id(&self) -> Option<u64> {
        match *self.phase.lock().expect("session phase") {
            Phase::Active(id) => Some(id),
            _ => None,
        }
    }
    pub(in crate::store) fn is_dormant(&self) -> bool {
        matches!(*self.phase.lock().expect("session phase"), Phase::Dormant)
    }
    /// Only the owning transaction's validated cold checkpoint may reset this
    /// attachment. Existing active/suspended checkpoints restore their tokens.
    pub(in crate::store) fn reset_dormant(&self) {
        let previous = {
            let mut phase = self.phase.lock().expect("session phase");
            assert!(
                !matches!(*phase, Phase::Acquiring),
                "cold rollback cannot run inside acquisition admission"
            );
            std::mem::replace(&mut *phase, Phase::Dormant)
        };
        if let (Some(owner), Phase::Active(id)) = (self.owner.upgrade(), previous) {
            lock(&owner).abort(id);
        }
    }
    // A cold attachment must not inspect/probe through another open or frozen
    // root. Reentry during acquisition bypasses that attempt without changing
    // the acquisition phase. No optional work callback runs in either case.
    fn may_attempt(&self) -> bool {
        let phase = *self.phase.lock().expect("session phase");
        match phase {
            Phase::Acquiring | Phase::Bypassed => false,
            Phase::Active(id) => self
                .owner
                .upgrade()
                .is_some_and(|owner| lock(&owner).mutable(id)),
            Phase::Dormant => {
                let Some(owner) = self.owner.upgrade() else {
                    return false;
                };
                if lock(&owner).banks.occupied() {
                    let mut phase = self.phase.lock().expect("session phase");
                    if matches!(*phase, Phase::Dormant) {
                        *phase = Phase::Bypassed;
                    }
                    return false;
                }
                true
            }
        }
    }
    fn ensure_active(&self, admission: &mut Admission<'_>) -> Result<Option<u64>, DomError> {
        {
            let mut phase = self.phase.lock().expect("session phase");
            match *phase {
                Phase::Active(id) => return Ok(Some(id)),
                Phase::Acquiring | Phase::Bypassed => return Ok(None),
                Phase::Dormant => *phase = Phase::Acquiring,
            }
        }
        let mut acquiring = Acquisition {
            session: self,
            partial: None,
            complete: false,
        };
        let Some(owner) = self.owner.upgrade() else {
            acquiring.finish(Phase::Bypassed);
            return Ok(None);
        };
        if lock(&owner).banks.occupied() {
            acquiring.finish(Phase::Bypassed);
            return Ok(None);
        }
        admission.work(html_work(HtmlStringPool::root_work())?)?;
        #[cfg(test)]
        admitted_html(HtmlStringPool::root_work(), false);
        admission.work(banks::ROOT_WORK)?;
        let mut state = lock(&owner);
        if state.banks.occupied() {
            drop(state);
            acquiring.finish(Phase::Bypassed);
            return Ok(None);
        }
        let Some(id) = state.banks.begin() else {
            drop(state);
            acquiring.finish(Phase::Bypassed);
            return Ok(None);
        };
        acquiring.partial = Some(id);
        let Some(strings) = state.strings.begin_root() else {
            state.banks.abort(id);
            acquiring.partial = None;
            drop(state);
            acquiring.finish(Phase::Bypassed);
            return Ok(None);
        };
        state.roots = Some(Roots {
            id,
            strings,
            marks: 0,
            suspended: false,
        });
        #[cfg(test)]
        metric(Event::SharingPairActivated);
        drop(state);
        acquiring.finish(Phase::Active(id));
        Ok(Some(id))
    }
    pub(in crate::store) fn checkpoint(
        &self,
        admission: &mut Admission<'_>,
    ) -> Result<Checkpoint, DomError> {
        let empty = || Checkpoint {
            owner: Weak::new(),
            root: None,
            marks: Mutex::new(None),
            _authority: None,
        };
        let (Some(owner), Some(id)) = (self.owner.upgrade(), self.active_id()) else {
            return Ok(empty());
        };
        if !lock(&owner).mutable(id) {
            return Ok(empty());
        }
        admission.work(html_work(HtmlStringPool::mark_work())?)?;
        #[cfg(test)]
        admitted_html(HtmlStringPool::mark_work(), false);
        admission.work(banks::MARK_WORK)?;
        let ledger = lock(&owner).ledger.clone();
        let authority = ledger.retain(
            custody::Usage::new(size_of::<Mutex<State>>(), 0),
            &mut *admission.global,
        )?;
        let mut state = lock(&owner);
        if !state.mutable(id) {
            return Ok(empty());
        }
        let bank = state.banks.mark(id);
        let strings = {
            let State { strings, roots, .. } = &mut *state;
            protocol(strings.mark(&roots.as_ref().expect("active root").strings))
        };
        match (bank, strings) {
            (Some(bank), Some(strings)) => {
                state.roots.as_mut().expect("active root").marks += 1;
                #[cfg(test)]
                metric(Event::SharingMarkCreated);
                Ok(Checkpoint {
                    owner: self.owner.clone(),
                    root: Some(id),
                    marks: Mutex::new(Some(Marks { strings, bank })),
                    _authority: Some(authority),
                })
            }
            (bank, strings) => {
                if let Some(bank) = bank {
                    state.banks.accept(bank);
                }
                if let Some(strings) = strings {
                    protocol(state.strings.accept_mark(strings));
                }
                let State { strings, roots, .. } = &mut *state;
                let root = roots.as_mut().expect("active root");
                protocol(strings.suspend_mutations(&root.strings));
                root.suspended = true;
                Ok(empty())
            }
        }
    }
    pub(in crate::store) fn prepare_close(
        &self,
        marks: &[&Checkpoint],
        admission: &mut Admission<'_>,
    ) -> Result<CloseReady, DomError> {
        let (Some(owner), Some(id)) = (self.owner.upgrade(), self.active_id()) else {
            return Ok(CloseReady {
                owner: self.owner.as_ptr() as usize,
                root: None,
                marks: 0,
            });
        };
        let live = lock(&owner)
            .roots
            .as_ref()
            .filter(|root| root.id == id)
            .expect("owned open session")
            .marks;
        // Price both verification passes and the protected close selection
        // before traversing nominations. Only scalar shape is read above.
        admission.work(html_work(HtmlStringPool::seal_work())?)?;
        #[cfg(test)]
        admitted_html(HtmlStringPool::seal_work(), true);
        admission.work(banks::close_work(marks.len(), live)?)?;
        let verify = |state: &State| {
            let root = state
                .roots
                .as_ref()
                .filter(|root| root.id == id)
                .expect("owned open session");
            let mut count = 0;
            for (i, mark) in marks.iter().enumerate() {
                #[cfg(test)]
                metric(Event::SharingCoordinationVisit);
                assert!(
                    marks[..i].iter().all(|previous| {
                        #[cfg(test)]
                        metric(Event::SharingCoordinationVisit);
                        !std::ptr::eq(*previous, *mark)
                    }),
                    "each pool mark nominated once"
                );
                assert!(
                    mark.root.is_none() || Weak::ptr_eq(&mark.owner, &self.owner),
                    "same optional owner"
                );
                if mark.root == Some(id)
                    && mark
                        .marks
                        .lock()
                        .expect("mark state")
                        .as_ref()
                        .is_some_and(|tokens| state.banks.mark_is_live(&tokens.bank))
                {
                    count += 1;
                }
            }
            assert_eq!(
                count, root.marks,
                "resolve every live pool mark before freeze"
            );
            assert_eq!(root.marks, live, "protected close mark shape");
            count
        };
        let count = verify(&lock(&owner));
        assert_eq!(
            verify(&lock(&owner)),
            count,
            "exclusive protected close preparation"
        );
        #[cfg(test)]
        metric(Event::SharingClosePrepared);
        Ok(CloseReady {
            owner: self.owner.as_ptr() as usize,
            root: Some(id),
            marks: count,
        })
    }
    pub(in crate::store) fn close(mut self, ready: CloseReady, marks: &[&Checkpoint]) -> Prepared {
        assert_eq!(
            ready.owner,
            self.owner.as_ptr() as usize,
            "same close owner"
        );
        assert!(
            self.active_id() == ready.root || self.owner.upgrade().is_none(),
            "same close session"
        );
        let mut prepared = Prepared {
            owner: std::mem::take(&mut self.owner),
            root: None,
            strings: None,
            _authority: self._authority.take(),
        };
        let (Some(owner), Some(id)) = (prepared.owner.upgrade(), ready.root) else {
            *self.phase.lock().expect("session phase") = Phase::Bypassed;
            return prepared;
        };
        assert_eq!(
            self.active_id(),
            Some(id),
            "close authority belongs to this session"
        );
        let mut state = lock(&owner);
        assert_eq!(state.roots.as_ref().expect("owned root").marks, ready.marks);
        // Deepest-first acceptance keeps the fixed parent inverse capacity.
        while state.roots.as_ref().expect("owned root").marks != 0 {
            let mark = marks
                .iter()
                .filter(|mark| {
                    #[cfg(test)]
                    metric(Event::SharingCoordinationVisit);
                    mark.root == Some(id)
                })
                .filter_map(|mark| {
                    mark.marks
                        .lock()
                        .expect("mark state")
                        .as_ref()
                        .filter(|m| state.banks.mark_is_live(&m.bank))
                        .map(|m| (m.bank.serial(), *mark))
                })
                .max_by_key(|(serial, _)| *serial)
                .expect("complete nominated marks")
                .1;
            let tokens = mark
                .marks
                .lock()
                .expect("mark state")
                .take()
                .expect("live nominated mark");
            protocol(state.strings.accept_mark(tokens.strings));
            state.banks.accept(tokens.bank);
            #[cfg(test)]
            metric(Event::SharingMarkAccepted);
            state.roots.as_mut().expect("owned root").marks -= 1;
        }
        let root = state.roots.take().expect("owned root");
        let strings = state
            .strings
            .prepare_seal(root.strings)
            .unwrap_or_else(|_| panic!("preadmitted mark-free freeze"));
        protocol(state.strings.validate_seal(&strings));
        state.banks.freeze(id);
        // The frozen bank root and prepared HTML token remain paired.
        prepared.root = Some(id);
        prepared.strings = Some(strings);
        #[cfg(test)]
        metric(Event::SharingPairClosed);
        *self.phase.lock().expect("session phase") = Phase::Bypassed;
        state.reconcile();
        prepared
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if let (Some(owner), Phase::Active(id)) = (
            self.owner.upgrade(),
            *self.phase.get_mut().expect("session phase"),
        ) {
            lock(&owner).abort(id);
        }
    }
}
impl Checkpoint {
    pub(in crate::store) fn restore(self) {
        self.finish(true);
    }
    fn finish(&self, restore: bool) {
        let Some(tokens) = self.marks.lock().expect("mark state").take() else {
            return;
        };
        let (Some(owner), Some(id)) = (self.owner.upgrade(), self.root) else {
            return;
        };
        let mut state = lock(&owner);
        if !state.active(id) {
            return;
        }
        if restore {
            protocol(state.strings.restore_mark(tokens.strings));
            let removed = state.banks.restore(tokens.bank);
            #[cfg(test)]
            metric(Event::SharingMarkRestored);
            state.roots.as_mut().expect("owned root").marks -= removed;
        } else {
            protocol(state.strings.accept_mark(tokens.strings));
            if state.banks.accept(tokens.bank) {
                #[cfg(test)]
                metric(Event::SharingMarkAccepted);
                state.roots.as_mut().expect("owned root").marks -= 1;
            }
        }
        state.reconcile();
    }
}
impl Drop for Checkpoint {
    fn drop(&mut self) {
        self.finish(false);
    }
}
impl Prepared {
    pub(in crate::store) fn commit(mut self) {
        if let (Some(owner), Some(id), Some(strings)) =
            (self.owner.upgrade(), self.root.take(), self.strings.take())
        {
            let mut state = lock(&owner);
            protocol(state.strings.validate_seal(&strings));
            protocol(state.strings.seal(strings));
            state.banks.seal(id);
            #[cfg(test)]
            metric(Event::SharingPairCommitted);
            state.reconcile();
        }
    }
}
impl Drop for Prepared {
    fn drop(&mut self) {
        if let (Some(owner), Some(id), Some(strings)) =
            (self.owner.upgrade(), self.root.take(), self.strings.take())
        {
            let mut state = lock(&owner);
            protocol(state.strings.abort_seal(strings));
            state.banks.abort(id);
            #[cfg(test)]
            metric(Event::SharingPairAborted);
            state.reconcile();
        }
    }
}

#[cfg(test)]
fn allocation(bytes: usize) {
    if super::scale_metrics::ENABLED && bytes != 0 {
        super::scale_metrics::record(super::scale_metrics::Event::StructuralAllocation { bytes });
    }
}

//! Persistent indexed records. Slot identity and generation policy belong to
//! the consuming owner; this bank owns path copying and vacancy summaries.

pub(crate) mod owned;

use std::{
    any::Any,
    marker::PhantomData,
    mem::size_of,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

use super::{
    DomError,
    custody::{Debit, Held, Ledger, MaterializationId, SharedDebit, Usage},
};

#[cfg(test)]
use super::scale_metrics::{self, Domain, Event};

#[cfg(test)]
#[inline]
fn metric(event: Event) {
    if scale_metrics::ENABLED {
        scale_metrics::record(event);
    }
}

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
    identity: MaterializationId,
    reusable: bool,
    totals: Totals,
}

impl<T> Clone for Entry<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            identity: self.identity,
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
    identity: MaterializationId,
    /// Child-index shift; zero identifies a leaf page.
    shift: u32,
    contents: Contents<T>,
    summary: Summary,
    // The original hosted producer owns this physical allocation. DOM custody
    // remains exclusively in the parallel owner graph. Contents drop first.
    producer_origin: Option<MaterializationId>,
    _producer: Option<crate::hosted::Retained>,
}

/// Qualification-only layout allowance, independent of observed allocations.
#[cfg(test)]
pub(in crate::store) fn scale_full_page_bytes<T>() -> usize {
    assert_eq!(WIDTH, 16, "qualification radix premise");
    size_of::<Page<T>>()
        .checked_add(
            WIDTH
                .checked_mul(size_of::<Option<Entry<T>>>().max(size_of::<Option<Arc<Page<T>>>>()))
                .expect("typed qualification page backing fits"),
        )
        .expect("typed qualification page fits")
}

#[cfg(test)]
thread_local! {
    static SCALE_PAGE_VERIFICATION: std::cell::Cell<Option<usize>> = const {
        std::cell::Cell::new(None)
    };
}

/// Scoped observation of actual setup pages, including superseded candidates.
#[cfg(test)]
pub(in crate::store) struct ScalePageCapacityVerification;
#[cfg(test)]
impl ScalePageCapacityVerification {
    pub fn begin() -> Self {
        SCALE_PAGE_VERIFICATION.with(|state| {
            assert!(state.get().is_none(), "one setup observer");
            state.set(Some(0));
        });
        Self
    }
    pub fn finish(self) -> usize {
        SCALE_PAGE_VERIFICATION.with(|state| state.get().expect("active setup observer"))
    }
}
#[cfg(test)]
impl Drop for ScalePageCapacityVerification {
    fn drop(&mut self) {
        SCALE_PAGE_VERIFICATION.with(|state| state.set(None));
    }
}
#[cfg(test)]
fn scale_verify_page<T>(actual_own_bytes: usize) {
    SCALE_PAGE_VERIFICATION.with(|state| {
        if let Some(count) = state.get() {
            assert!(
                actual_own_bytes <= scale_full_page_bytes::<T>(),
                "actual setup page exceeds the typed full-page capacity premise"
            );
            state.set(Some(count.checked_add(1).expect("setup page count fits")));
        }
    });
}

#[cfg(test)]
impl<T> Drop for Page<T> {
    fn drop(&mut self) {
        // The observation occurs while this actual Page and Vec still exist.
        drop_observation::page(self.identity);
    }
}

#[cfg(test)]
mod drop_observation {
    use std::cell::RefCell;

    use super::MaterializationId;

    type PageHook = Box<dyn FnMut(MaterializationId)>;
    thread_local! {
        static PAGES: RefCell<Option<PageHook>> = const { RefCell::new(None) };
        static RELEASE: RefCell<Option<Box<dyn FnOnce()>>> = const { RefCell::new(None) };
        static PLANNED: RefCell<Option<Box<dyn FnOnce()>>> = const { RefCell::new(None) };
    }

    pub(super) struct Pages;

    pub(super) fn pages(hook: impl FnMut(MaterializationId) + 'static) -> Pages {
        PAGES.with(|slot| {
            assert!(
                slot.borrow().is_none(),
                "one page-drop observation per owner thread"
            );
            *slot.borrow_mut() = Some(Box::new(hook));
        });
        Pages
    }

    pub(super) fn page(identity: MaterializationId) {
        if let Some(mut hook) = PAGES.with(|slot| slot.borrow_mut().take()) {
            hook(identity);
            PAGES.with(|slot| *slot.borrow_mut() = Some(hook));
        }
    }

    impl Drop for Pages {
        fn drop(&mut self) {
            PAGES.with(|slot| *slot.borrow_mut() = None);
        }
    }

    pub(super) fn release_once(hook: impl FnOnce() + 'static) {
        RELEASE.with(|slot| {
            assert!(
                slot.borrow().is_none(),
                "one scheduled release stage per thread"
            );
            *slot.borrow_mut() = Some(Box::new(hook));
        });
    }

    pub(super) fn before_release() {
        if let Some(hook) = RELEASE.with(|slot| slot.borrow_mut().take()) {
            hook();
        }
    }

    pub(super) fn registration_once(hook: impl FnOnce() + 'static) {
        PLANNED.with(|slot| {
            assert!(
                slot.borrow().is_none(),
                "one scheduled registration stage per thread"
            );
            *slot.borrow_mut() = Some(Box::new(hook));
        });
    }

    pub(super) fn planned() {
        if let Some(hook) = PLANNED.with(|slot| slot.borrow_mut().take()) {
            hook();
        }
    }
}

type PageReplacement<T> = (Option<Arc<Page<T>>>, IndexCharge);

/// Identity travels with the actual Arc materialization through every alias.
/// It does not participate in semantic bank comparison.
pub(crate) struct Record<T> {
    value: Arc<T>,
    identity: MaterializationId,
}

/// Affine identity reservation for owners that must admit a new record before
/// constructing/allocating it. Failed attempts may burn this private identity.
pub(crate) struct ReservedRecord<T> {
    identity: MaterializationId,
    value: PhantomData<fn() -> T>,
}

impl<T> ReservedRecord<T> {
    pub fn materialize(self, value: T) -> Record<T> {
        let record = Record {
            value: Arc::new(value),
            identity: self.identity,
        };
        #[cfg(test)]
        metric(Event::RecordMaterialized {
            bytes: size_of::<T>(),
        });
        record
    }
}

impl<T> Record<T> {
    pub fn new(value: T) -> Result<Self, DomError> {
        Ok(Self::reserve()?.materialize(value))
    }

    pub fn reserve() -> Result<ReservedRecord<T>, DomError> {
        Ok(ReservedRecord {
            identity: MaterializationId::fresh()?,
            value: PhantomData,
        })
    }

    pub fn value(&self) -> &Arc<T> {
        &self.value
    }
}

impl<T> Clone for Record<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            identity: self.identity,
        }
    }
}

/// An admitted materialized record and its matching controlled occurrence.
/// The semantic handle drops before its custody handle. Cloning this pair
/// aliases that record; it does not manufacture another payload-field occurrence.
pub(crate) struct ControlledRecord<T> {
    record: Record<T>,
    custody: RecordRetention,
}

/// A real owner-only dependency allocation. The semantic record never carries
/// this authority; its controlled record owns the child frontier instead.
#[derive(Clone)]
pub(crate) struct Dependency {
    value: Arc<dyn Any + Send + Sync>,
    debit: SharedDebit,
}

impl Dependency {
    pub fn new<T: Any + Send + Sync>(
        value: T,
        ledger: &Ledger,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let value = Held::new(
            value,
            ledger.retain(Usage::new(size_of::<T>(), 0), &mut admit)?,
        );
        Self::from_retained(value, Usage::new(size_of::<T>(), 0), admit)
    }

    /// Transfer an already admitted dependency body and its actual dynamic
    /// backing. Data remains ahead of credit through every fallible step.
    pub fn from_retained<T: Any + Send + Sync>(
        mut incoming: Held<T>,
        usage: Usage,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        assert!(
            usage.bytes() >= size_of::<T>(),
            "dependency includes its body"
        );
        let extra = usage.checked_sub(incoming.debit().usage())?;
        incoming.debit_mut().grow(extra, &mut admit)?;
        let debit = incoming.debit_mut().share(&mut admit)?;
        let (value, empty) = incoming.into_parts();
        drop(empty);
        let value = Arc::new(value);
        #[cfg(test)]
        metric(Event::StructuralAllocation {
            bytes: size_of::<T>(),
        });
        Ok(Self { value, debit })
    }

    /// Complete an unpublished dependency before any custody alias escapes.
    pub(in crate::store) fn value_mut<T: Any + Send + Sync>(&mut self) -> &mut T {
        Arc::get_mut(&mut self.value)
            .expect("unpublished dependency has one owner")
            .downcast_mut::<T>()
            .expect("one owning dependency type")
    }

    fn get<T: Any + Send + Sync>(&self) -> Retained<T> {
        let value = self
            .value
            .clone()
            .downcast::<T>()
            .unwrap_or_else(|_| panic!("a materialization has one owning dependency type"));
        Retained {
            value,
            debit: self.debit.clone(),
        }
    }
}

/// A borrowed child frontier may escape its original record only together
/// with the debit of the actual dependency body it keeps alive.
pub(crate) struct Retained<T> {
    value: Arc<T>,
    debit: SharedDebit,
}
impl<T> Clone for Retained<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            debit: self.debit.clone(),
        }
    }
}
impl<T> std::ops::Deref for Retained<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

/// A record's shallow occurrence and, when present, its owned child frontier.
pub(crate) struct RecordDescription {
    usage: Usage,
    retained: Option<SharedDebit>,
    dependency: Option<Dependency>,
}
impl From<Usage> for RecordDescription {
    fn from(usage: Usage) -> Self {
        Self {
            usage,
            retained: None,
            dependency: None,
        }
    }
}
impl RecordDescription {
    pub fn with_retained(mut self, retained: SharedDebit) -> Self {
        assert_eq!(
            retained.usage(),
            self.usage
                .checked_add(SharedDebit::administration_usage())
                .expect("preadmitted record receipt")
        );
        self.retained = Some(retained);
        self
    }

    fn assert_domain(&self, ledger: &Ledger) {
        assert!(
            self.retained
                .as_ref()
                .is_none_or(|retained| retained.belongs_to(ledger)),
            "retained record uses the original controlled domain"
        );
        assert!(
            self.dependency
                .as_ref()
                .is_none_or(|dependency| dependency.debit.belongs_to(ledger)),
            "a record dependency uses its controlled domain"
        );
    }

    pub fn with_dependency(usage: Usage, dependency: Dependency) -> Self {
        Self {
            usage,
            retained: None,
            dependency: Some(dependency),
        }
    }
}

/// Complete provisional owner from entry through identity and metadata refusal.
/// Semantic fields must be destroyed before their nested dependency and debit.
struct PendingRecord<T> {
    value: T,
    description: RecordDescription,
    debit: Option<SharedDebit>,
}

impl<T> ControlledRecord<T> {
    /// Admission must cover the full record and its conservative field receipts
    /// before allocating/cloning materialized payload. The SharedDebit also
    /// includes its actual out-of-line metadata target.
    pub fn from_parts(record: Record<T>, debit: SharedDebit) -> Self {
        Self {
            record,
            custody: RecordRetention::Pending {
                debit,
                dependency: None,
            },
        }
    }

    pub fn value(&self) -> &Arc<T> {
        self.record.value()
    }

    /// Add an already admitted child dependency before registering this record.
    pub fn with_dependency(mut self, dependency: Dependency) -> Self {
        let RecordRetention::Pending {
            dependency: slot, ..
        } = &mut self.custody
        else {
            panic!("a prepared record receives its dependency before registration");
        };
        assert!(
            slot.is_none(),
            "one dependency capsule owns all nested fields"
        );
        *slot = Some(dependency);
        self
    }

    pub fn dependency<D: Any + Send + Sync>(&self) -> Option<Retained<D>> {
        match &self.custody {
            RecordRetention::Pending { dependency, .. } => dependency.as_ref(),
            RecordRetention::Registered(custody) => custody.dependency.as_ref(),
        }
        .map(Dependency::get)
    }

    /// Reserve identity and admit the materialized occurrence before its Arc
    /// allocation. Any preexisting backing in `value` already belongs to its
    /// constructing owner's admitted temporary custody.
    pub fn new(
        value: T,
        description: impl Into<RecordDescription>,
        registry: &CustodyRegistry,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        Self::new_with_identity(
            value,
            description,
            registry,
            &mut admit,
            MaterializationId::fresh,
        )
    }

    fn new_with_identity(
        value: T,
        description: impl Into<RecordDescription>,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        allocate_identity: impl FnOnce() -> Result<MaterializationId, DomError>,
    ) -> Result<Self, DomError> {
        let pending = PendingRecord {
            value,
            description: description.into(),
            debit: None,
        };
        let result = Self::materialize_pending(pending, registry, admit, allocate_identity);
        if result.is_err() {
            registry.reconcile_releases();
        }
        result
    }

    fn materialize_pending(
        mut pending: PendingRecord<T>,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        allocate_identity: impl FnOnce() -> Result<MaterializationId, DomError>,
    ) -> Result<Self, DomError> {
        pending.description.assert_domain(registry.ledger());
        let identity = allocate_identity()?;
        pending.debit = Some(match &pending.description.retained {
            Some(retained) => retained.clone(),
            None => registry
                .ledger
                .retain_shared(pending.description.usage, &mut *admit)?,
        });
        let custody = registry.register_record(
            identity,
            pending.debit.as_ref().unwrap().clone(),
            pending.description.dependency.clone(),
            admit,
        )?;
        let record = ReservedRecord {
            identity,
            value: PhantomData,
        }
        .materialize(pending.value);
        Ok(Self {
            record,
            custody: RecordRetention::Registered(custody),
        })
    }
}

impl<T> Clone for ControlledRecord<T> {
    fn clone(&self) -> Self {
        Self {
            record: self.record.clone(),
            custody: self.custody.clone(),
        }
    }
}

#[derive(Clone)]
enum RecordRetention {
    Pending {
        debit: SharedDebit,
        dependency: Option<Dependency>,
    },
    Registered(Arc<RecordCustody>),
}

struct RecordCustody {
    identity: MaterializationId,
    dependency: Option<Dependency>,
    attachment: Mutex<Option<Attachment>>,
    _payload: SharedDebit,
    release: ReleaseTicket,
    _administration: Debit,
}

/// Owner-only custody of a non-bank materialization carrying the same private
/// identity discipline (for example an original Source spent-work atom).
#[derive(Clone)]
pub(crate) struct ObjectCustody(Arc<RecordCustody>);

/// A cache target cannot outlive the original-object custody that owns its
/// Weak backing debit. Destroy its actual data before the final owner alias.
pub(crate) struct Attached<T> {
    value: Arc<T>,
    owner: ObjectCustody,
}

impl<T> Clone for Attached<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            owner: self.owner.clone(),
        }
    }
}

impl<T> std::ops::Deref for Attached<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> Attached<T> {
    #[cfg(test)]
    pub(in crate::store) fn inspect_control_graph(&self) -> PhysicalControlGraph {
        self.owner.inspect_control_graph()
    }

    #[cfg(test)]
    pub fn inspect_owner_payload(&self) -> (MaterializationId, Usage) {
        (self.owner.identity(), self.owner.inspect_payload_usage())
    }

    #[cfg(test)]
    pub fn shares_target(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.value, &other.value)
    }
}

/// The weak target is destroyed before the admitted body charge. This is an
/// owner-side cache; it never retains semantic records or grants execution.
struct Attachment {
    target: Weak<dyn Any + Send + Sync>,
    _debit: Debit,
}
struct PreparedAttachment<T> {
    value: Arc<T>,
    attachment: Attachment,
}

impl ObjectCustody {
    #[cfg(test)]
    pub(in crate::store) fn inspect_dependency<T: Any + Send + Sync>(&self) -> Option<Retained<T>> {
        self.0.dependency.as_ref().map(Dependency::get)
    }
    #[cfg(test)]
    pub(in crate::store) fn inspect_control_graph(&self) -> PhysicalControlGraph {
        let mut graph = PhysicalControlGraph::default();
        graph.visit_record(&self.0);
        graph
    }

    #[cfg(test)]
    pub fn inspect_payload_usage(&self) -> Usage {
        self.0
            ._payload
            .usage()
            .checked_sub(SharedDebit::administration_usage())
            .expect("shared payload includes its actual receipt administration")
    }

    #[cfg(test)]
    pub fn identity(&self) -> MaterializationId {
        self.0.identity
    }

    pub fn attachment<T: Any + Send + Sync>(&self) -> Option<Attached<T>> {
        let value = self
            .0
            .attachment
            .lock()
            .expect("owner attachment lock")
            .as_ref()
            .and_then(|attachment| attachment.target.upgrade());
        value.map(|value| Attached {
            value: value
                .downcast::<T>()
                .unwrap_or_else(|_| panic!("a materialization has one owning attachment type")),
            owner: self.clone(),
        })
    }

    /// All admission/allocation has completed before this short cache lock.
    /// A concurrent winner is reused; losing data is destroyed before its debit.
    pub fn attach<T: Any + Send + Sync>(&self, prepared: Held<Arc<T>>) -> Attached<T> {
        assert!(
            self.0._payload.shares_domain(prepared.debit()),
            "an attachment uses its original object's controlled domain"
        );
        let (value, debit) = prepared.into_parts();
        let erased: Arc<dyn Any + Send + Sync> = value.clone();
        let prepared = PreparedAttachment {
            value,
            attachment: Attachment {
                target: Arc::downgrade(&erased),
                _debit: debit,
            },
        };
        drop(erased);
        let mut slot = self.0.attachment.lock().expect("owner attachment lock");
        if let Some(existing) = slot
            .as_ref()
            .and_then(|attachment| attachment.target.upgrade())
        {
            drop(slot);
            return Attached {
                value: existing
                    .downcast::<T>()
                    .unwrap_or_else(|_| panic!("a materialization has one owning attachment type")),
                owner: self.clone(),
            };
        }
        let old = slot.replace(prepared.attachment);
        drop(slot);
        drop(old);
        Attached {
            value: prepared.value,
            owner: self.clone(),
        }
    }
}

enum CustodyEdges {
    Leaf([Option<Arc<RecordCustody>>; WIDTH]),
    Branch([Option<Arc<PageCustody>>; WIDTH]),
}

/// DOM tokens and child edges stay outside immutable semantic Pages.
/// Children release before this page's own structural/control debit.
struct PageCustody {
    identity: MaterializationId,
    edges: CustodyEdges,
    _payload: Option<SharedDebit>,
    release: ReleaseTicket,
    debit: Debit,
}

/// A controlled root's parallel lease graph, never stored in immutable State.
/// The owner must pair it with the exact Bank and destroy that semantic holder
/// first. Root cloning is O(1) and creates no recursive payload debit.
pub(crate) struct BankCustody<T> {
    root: Option<Arc<PageCustody>>,
    record: PhantomData<fn() -> T>,
}

impl<T> Default for BankCustody<T> {
    fn default() -> Self {
        Self {
            root: None,
            record: PhantomData,
        }
    }
}

impl<T> Clone for BankCustody<T> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            record: PhantomData,
        }
    }
}

#[cfg(test)]
impl<T> BankCustody<T> {
    pub fn inspect_materializations(&self) -> Vec<(MaterializationId, Usage)> {
        fn walk(page: &PageCustody, output: &mut Vec<(MaterializationId, Usage)>) {
            let own = match &page._payload {
                Some(payload) => payload
                    .usage()
                    .checked_sub(SharedDebit::administration_usage()),
                None => page.debit.usage().checked_sub(Usage::new(
                    size_of::<PageCustody>() + size_of::<Mutex<RegistryState>>(),
                    0,
                )),
            }
            .expect("actual page body debit excludes separate metadata");
            output.push((page.identity, own));
            match &page.edges {
                CustodyEdges::Branch(children) => {
                    for child in children.iter().flatten() {
                        walk(child, output);
                    }
                }
                CustodyEdges::Leaf(records) => {
                    for record in records.iter().flatten() {
                        output.push((
                            record.identity,
                            record
                                ._payload
                                .usage()
                                .checked_sub(SharedDebit::administration_usage())
                                .expect("actual record occurrence"),
                        ));
                    }
                }
            }
        }
        let mut output = Vec::new();
        if let Some(root) = &self.root {
            walk(root, &mut output);
        }
        output
    }
    pub fn inspect_dependencies<D: Any + Send + Sync>(&self) -> Vec<Retained<D>> {
        fn walk<D: Any + Send + Sync>(page: &PageCustody, output: &mut Vec<Retained<D>>) {
            match &page.edges {
                CustodyEdges::Branch(children) => {
                    for child in children.iter().flatten() {
                        walk(child, output);
                    }
                }
                CustodyEdges::Leaf(records) => {
                    for record in records.iter().flatten() {
                        if let Some(dependency) = &record.dependency {
                            output.push(dependency.get());
                        }
                    }
                }
            }
        }
        let mut output = Vec::new();
        if let Some(root) = &self.root {
            walk(root, &mut output);
        }
        output
    }

    pub(in crate::store) fn inspect_control_graph(&self) -> PhysicalControlGraph {
        let mut graph = PhysicalControlGraph::default();
        if let Some(root) = &self.root {
            graph.visit_page(root);
        }
        graph
    }
}

#[cfg(test)]
#[derive(Default)]
pub(in crate::store) struct PhysicalControlGraph {
    pages: std::collections::BTreeSet<usize>,
    records: std::collections::BTreeSet<usize>,
    payload_debits: std::collections::BTreeSet<usize>,
    releases: std::collections::BTreeSet<usize>,
}

#[cfg(test)]
impl PhysicalControlGraph {
    fn visit_page(&mut self, page: &Arc<PageCustody>) {
        if !self.pages.insert(Arc::as_ptr(page) as usize) {
            return;
        }
        self.visit_release(&page.release);
        if let Some(payload) = &page._payload {
            self.payload_debits.insert(payload.allocation_identity());
        }
        match &page.edges {
            CustodyEdges::Branch(children) => {
                for child in children.iter().flatten() {
                    self.visit_page(child);
                }
            }
            CustodyEdges::Leaf(records) => {
                for record in records.iter().flatten() {
                    self.visit_record(record);
                }
            }
        }
    }

    fn visit_record(&mut self, record: &Arc<RecordCustody>) {
        if !self.records.insert(Arc::as_ptr(record) as usize) {
            return;
        }
        self.payload_debits
            .insert(record._payload.allocation_identity());
        self.visit_release(&record.release);
    }

    fn visit_release(&mut self, release: &ReleaseTicket) {
        if let Some(node) = &release.node {
            self.releases.insert(&**node as *const ReleaseNode as usize);
        }
    }

    pub(in crate::store) fn merge(&mut self, other: Self) {
        self.pages.extend(other.pages);
        self.records.extend(other.records);
        self.payload_debits.extend(other.payload_debits);
        self.releases.extend(other.releases);
    }

    pub(in crate::store) fn usage(&self) -> Usage {
        Usage::new(
            self.pages.len() * size_of::<PageCustody>()
                + self.records.len() * size_of::<RecordCustody>()
                + (self.pages.len() + self.records.len()) * size_of::<Mutex<RegistryState>>()
                + self.payload_debits.len() * size_of::<Debit>()
                + self.releases.len() * size_of::<ReleaseNode>(),
            0,
        )
    }
}

/// Every provisional holder destroys semantic storage and borrowed child
/// handles before releasing their parallel custody and structural debit.
struct PendingPage<D> {
    semantic: D,
    edges: CustodyEdges,
    payload: Option<SharedDebit>,
    debit: Debit,
}

struct RegisteredPage<D> {
    semantic: D,
    custody: Arc<PageCustody>,
}

struct ControlledSlots<V, C> {
    values: Vec<Option<V>>,
    controls: [Option<C>; WIDTH],
    debit: Debit,
}

type ControlledPage<T> = RegisteredPage<Arc<Page<T>>>;
type ControlledReplacement<T> = Option<ControlledPage<T>>;

const ID_SHIFT: u32 = 60;

#[derive(Clone, Copy)]
enum RegistryKind {
    Page,
    Record,
}

enum WeakCustody {
    Page(Weak<PageCustody>),
    Record(Weak<RecordCustody>),
}

enum LiveCustody {
    Page(Arc<PageCustody>),
    Record(Arc<RecordCustody>),
}

struct RegistryEntry {
    nonce: u64,
    custody: WeakCustody,
    // Weak target bodies remain allocated after their value/destructor dies.
    // The Weak handle's inline size is in the registry leaf capacity already.
    _backing: Debit,
}

enum RegistryNode {
    Branch {
        children: Vec<Option<Box<RegistryNode>>>,
        _debit: Debit,
    },
    Leaf {
        entries: Vec<Option<RegistryEntry>>,
        _debit: Debit,
    },
}

struct ReleaseNode {
    kind: RegistryKind,
    identity: MaterializationId,
    nonce: u64,
    next: Option<Box<ReleaseNode>>,
    _debit: Debit,
}

impl Drop for ReleaseNode {
    fn drop(&mut self) {
        let mut next = self.next.take();
        while let Some(mut node) = next {
            next = node.next.take();
        }
    }
}

/// Preallocated before registry insertion. Last-custodian drop only links this
/// node into its original owner's queue; it allocates no storage and runs no
/// cancellation/limit callback. The weak owner target's fixed body is included
/// in the containing Page/Record custody debit from construction.
struct ReleaseTicket {
    owner: Weak<Mutex<RegistryState>>,
    node: Option<Box<ReleaseNode>>,
}

impl Drop for ReleaseTicket {
    fn drop(&mut self) {
        let Some(mut node) = self.node.take() else {
            return;
        };
        #[cfg(test)]
        drop_observation::before_release();
        if let Some(owner) = self.owner.upgrade() {
            let mut state = owner
                .lock()
                .expect("custody registry invariant has not panicked");
            node.next = state.released.take();
            state.released = Some(node);
        }
    }
}

struct RegistryState {
    pages: Option<Box<RegistryNode>>,
    records: Option<Box<RegistryNode>>,
    released: Option<Box<ReleaseNode>>,
    next_nonce: u64,
    _administration: Debit,
}

impl RegistryState {
    fn root(&self, kind: RegistryKind) -> &Option<Box<RegistryNode>> {
        match kind {
            RegistryKind::Page => &self.pages,
            RegistryKind::Record => &self.records,
        }
    }

    fn root_mut(&mut self, kind: RegistryKind) -> &mut Option<Box<RegistryNode>> {
        match kind {
            RegistryKind::Page => &mut self.pages,
            RegistryKind::Record => &mut self.records,
        }
    }

    fn reconcile_releases(&mut self) {
        while let Some(mut released) = self.released.take() {
            self.released = released.next.take();
            remove_registry_entry(
                self.root_mut(released.kind),
                released.identity.key(),
                ID_SHIFT,
                released.nonce,
            );
        }
    }

    /// A value can reach strong zero before its destructor queues its ticket.
    /// Retire that exact expired Weak now; its delayed nonce cannot affect a
    /// later custodian for the same still externally retained materialization.
    fn live(&mut self, kind: RegistryKind, identity: MaterializationId) -> Option<LiveCustody> {
        let entry = registry_entry(self.root(kind), identity.key(), ID_SHIFT)?;
        let live = match &entry.custody {
            WeakCustody::Page(value) => value.upgrade().map(LiveCustody::Page),
            WeakCustody::Record(value) => value.upgrade().map(LiveCustody::Record),
        };
        if live.is_none() {
            let nonce = entry.nonce;
            remove_registry_entry(self.root_mut(kind), identity.key(), ID_SHIFT, nonce);
        }
        live
    }

    fn plan(
        &mut self,
        kind: RegistryKind,
        identity: MaterializationId,
    ) -> Result<RegistrationPlan, DomError> {
        let nonce = self.next_nonce;
        self.next_nonce = nonce
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(RegistrationPlan {
            nonce,
            missing: missing_registry_shift(self.root(kind), identity.key(), ID_SHIFT),
        })
    }
}

/// Mutable accounting administration for one controlled domain. Keys are
/// immutable materialization IDs; neither keys nor weak sidecars retain any
/// semantic Page/record. A current captured pair bypasses lookup entirely.
#[derive(Clone)]
pub(crate) struct CustodyRegistry {
    state: Arc<Mutex<RegistryState>>,
    ledger: Ledger,
}

impl CustodyRegistry {
    pub const fn administration_usage() -> Usage {
        Usage::new(size_of::<Mutex<RegistryState>>(), 0)
    }

    pub fn new(
        ledger: &Ledger,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let administration = ledger.retain(Self::administration_usage(), admit)?;
        let state = Arc::new(Mutex::new(RegistryState {
            pages: None,
            records: None,
            released: None,
            next_nonce: 1,
            _administration: administration,
        }));
        #[cfg(test)]
        metric(Event::StructuralAllocation {
            bytes: size_of::<Mutex<RegistryState>>(),
        });
        Ok(Self {
            state,
            ledger: ledger.clone(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, RegistryState> {
        self.state
            .lock()
            .expect("custody registry invariant has not panicked")
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn total(&self) -> Usage {
        self.ledger.total()
    }

    /// Exact queued removals and empty-path pruning are allocation-free. This
    /// destroys weak target fields and releases their debit only when removed.
    pub fn reconcile_releases(&self) {
        self.lock().reconcile_releases();
    }

    /// Adopt an existing, borrowed materialization once in this domain. The
    /// issuer reserved its ID at original construction; adoption must not mint
    /// a new ID for an alias or put this token in externally held semantics.
    pub fn adopt_object(
        &self,
        identity: MaterializationId,
        usage: Usage,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<ObjectCustody, DomError> {
        self.adopt_object_with_children(identity, usage, |_| Ok(None), admit)
    }

    /// Construct a missing object's owned child frontier once. An active
    /// materialization aliases its original capsule without reading children.
    pub fn adopt_object_with_children(
        &self,
        identity: MaterializationId,
        usage: Usage,
        children: impl FnOnce(
            &mut dyn FnMut(Usage) -> Result<(), DomError>,
        ) -> Result<Option<Dependency>, DomError>,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<ObjectCustody, DomError> {
        #[cfg(test)]
        metric(Event::Entered(Domain::Accounting));
        if let Some(existing) = self.find_record(identity) {
            admit(self.total())?;
            return Ok(ObjectCustody(existing));
        }
        let result = (|| {
            let debit = self.ledger.retain_shared(usage, &mut admit)?;
            let dependency = children(&mut admit)?;
            self.register_record(identity, debit, dependency, &mut admit)
                .map(ObjectCustody)
        })();
        self.reconcile_releases();
        result
    }

    /// Register an already admitted occurrence without retaining its payload
    /// again. The caller keeps its complete semantic/token holder through all
    /// fallible preparation, then transfers that holder after success. An alias
    /// must carry the original identity and the original occurrence's receipt.
    pub fn adopt_object_retained(
        &self,
        identity: MaterializationId,
        retained: &SharedDebit,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<ObjectCustody, DomError> {
        self.adopt_object_retained_with_children(identity, retained, None, admit)
    }

    pub fn adopt_object_retained_with_children(
        &self,
        identity: MaterializationId,
        retained: &SharedDebit,
        dependency: Option<Dependency>,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<ObjectCustody, DomError> {
        assert!(
            retained.belongs_to(&self.ledger),
            "retained object admission uses its controlled domain"
        );
        #[cfg(test)]
        metric(Event::Entered(Domain::Accounting));
        let result = (|| {
            if let Some(existing) = self.find_record(identity) {
                admit(self.total())?;
                return Ok(ObjectCustody(existing));
            }
            self.register_record(identity, retained.clone(), dependency, &mut admit)
                .map(ObjectCustody)
        })();
        self.reconcile_releases();
        result
    }

    /// Actual registry allocations/weak occurrences, independently enumerated
    /// for tests. It deliberately never reads a Debit or cached Usage value.
    #[cfg(test)]
    pub(in crate::store) fn inspect_raw_allocations(
        &self,
        graph: &mut super::compact_tests::AllocationGraph,
    ) {
        use super::compact_tests::AllocationGraph;
        fn release(value: &ReleaseTicket, graph: &mut AllocationGraph) {
            if let Some(node) = &value.node {
                graph.allocation(
                    &**node as *const ReleaseNode as usize,
                    "control.release",
                    size_of::<ReleaseNode>(),
                );
            }
        }
        fn record(value: &Arc<RecordCustody>, graph: &mut AllocationGraph) {
            graph.target(value, "control.record");
            graph.allocation(
                value._payload.allocation_identity(),
                "control.debit",
                size_of::<Debit>(),
            );
            release(&value.release, graph);
            if let Some(dependency) = &value.dependency {
                graph.target(&dependency.value, "control.dependency");
                graph.allocation(
                    dependency.debit.allocation_identity(),
                    "control.debit",
                    size_of::<Debit>(),
                );
            }
            if let Some(attachment) = value.attachment.lock().unwrap().as_ref() {
                let target = attachment.target.upgrade().expect("this baseline observes live attached frontiers; dead erased targets need a separately qualified raw extent");
                graph.target(&target, "control.attachment");
            }
        }
        fn node(value: &RegistryNode, graph: &mut AllocationGraph, live: &mut Vec<LiveCustody>) {
            graph.allocation(
                value as *const RegistryNode as usize,
                "registry.node",
                size_of::<RegistryNode>(),
            );
            match value {
                RegistryNode::Branch { children, .. } => {
                    graph.buffer(children, "registry.children");
                    for child in children.iter().flatten() {
                        node(child, graph, live);
                    }
                }
                RegistryNode::Leaf { entries, .. } => {
                    graph.buffer(entries, "registry.entries");
                    for entry in entries.iter().flatten() {
                        match &entry.custody {
                            WeakCustody::Page(weak) => {
                                graph.allocation(
                                    weak.as_ptr() as usize,
                                    "control.page",
                                    size_of::<PageCustody>(),
                                );
                                if let Some(value) = weak.upgrade() {
                                    live.push(LiveCustody::Page(value));
                                }
                            }
                            WeakCustody::Record(weak) => {
                                graph.allocation(
                                    weak.as_ptr() as usize,
                                    "control.record",
                                    size_of::<RecordCustody>(),
                                );
                                if let Some(value) = weak.upgrade() {
                                    live.push(LiveCustody::Record(value));
                                }
                            }
                        }
                    }
                }
            }
        }
        graph.target(&self.state, "registry.state");
        self.ledger.inspect_raw_allocations(graph);
        let mut live = Vec::new();
        let state = self.lock();
        for root in [&state.pages, &state.records].into_iter().flatten() {
            node(root, graph, &mut live);
        }
        let mut released = state.released.as_deref();
        while let Some(value) = released {
            graph.allocation(
                value as *const ReleaseNode as usize,
                "control.release",
                size_of::<ReleaseNode>(),
            );
            released = value.next.as_deref();
        }
        // An upgraded last alias can release its preallocated ticket. Drop
        // these observations only after releasing the original registry lock.
        drop(state);
        for value in live {
            match value {
                LiveCustody::Page(value) => {
                    release(&value.release, graph);
                    if let Some(payload) = &value._payload {
                        graph.allocation(
                            payload.allocation_identity(),
                            "control.debit",
                            size_of::<Debit>(),
                        );
                    }
                }
                LiveCustody::Record(value) => record(&value, graph),
            }
        }
    }

    #[cfg(test)]
    pub(in crate::store) fn inspect_storage(&self) -> Usage {
        fn node_bytes(node: &RegistryNode) -> usize {
            size_of::<RegistryNode>()
                + match node {
                    RegistryNode::Branch { children, .. } => {
                        children.capacity() * size_of::<Option<Box<RegistryNode>>>()
                            + children
                                .iter()
                                .flatten()
                                .map(|node| node_bytes(node))
                                .sum::<usize>()
                    }
                    RegistryNode::Leaf { entries, .. } => {
                        entries.capacity() * size_of::<Option<RegistryEntry>>()
                            + entries
                                .iter()
                                .flatten()
                                .map(|entry| match &entry.custody {
                                    WeakCustody::Page(_) => size_of::<PageCustody>(),
                                    WeakCustody::Record(_) => size_of::<RecordCustody>(),
                                })
                                .sum::<usize>()
                    }
                }
        }
        let state = self.lock();
        let mut bytes = size_of::<Mutex<RegistryState>>()
            + state.pages.as_deref().map_or(0, node_bytes)
            + state.records.as_deref().map_or(0, node_bytes);
        let mut released = state.released.as_deref();
        while let Some(node) = released {
            bytes += size_of::<ReleaseNode>();
            released = node.next.as_deref();
        }
        Usage::new(bytes, 0)
    }

    fn find_page(&self, identity: MaterializationId) -> Option<Arc<PageCustody>> {
        let mut state = self.lock();
        state.reconcile_releases();
        match state.live(RegistryKind::Page, identity)? {
            LiveCustody::Page(custody) => Some(custody),
            LiveCustody::Record(_) => unreachable!("page registry contains only page custodians"),
        }
    }

    fn find_record(&self, identity: MaterializationId) -> Option<Arc<RecordCustody>> {
        let mut state = self.lock();
        state.reconcile_releases();
        match state.live(RegistryKind::Record, identity)? {
            LiveCustody::Record(custody) => Some(custody),
            LiveCustody::Page(_) => unreachable!("record registry contains only record custodians"),
        }
    }

    fn prepare_registration(
        &self,
        plan: RegistrationPlan,
        kind: RegistryKind,
        identity: MaterializationId,
        target_bytes: usize,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<PreparedRegistration, DomError> {
        let path = plan
            .missing
            .map(|shift| prepare_registry_path(identity.key(), shift, &self.ledger, admit))
            .transpose()?;
        let backing = self
            .ledger
            .retain(Usage::new(target_bytes, 0), &mut *admit)?;
        let release = self
            .ledger
            .retain(Usage::new(size_of::<ReleaseNode>(), 0), &mut *admit)?;
        let node = Box::new(ReleaseNode {
            kind,
            identity,
            nonce: plan.nonce,
            next: None,
            _debit: release,
        });
        #[cfg(test)]
        metric(Event::StructuralAllocation {
            bytes: size_of::<ReleaseNode>(),
        });
        Ok(PreparedRegistration {
            path,
            backing,
            node,
            plan,
        })
    }

    fn register_page<D>(
        &self,
        identity: MaterializationId,
        pending: PendingPage<D>,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<RegisteredPage<D>, DomError> {
        loop {
            let plan = {
                let mut state = self.lock();
                state.reconcile_releases();
                if let Some(LiveCustody::Page(custody)) = state.live(RegistryKind::Page, identity) {
                    drop(state);
                    return Ok(RegisteredPage {
                        semantic: pending.semantic,
                        custody,
                    });
                }
                state.plan(RegistryKind::Page, identity)?
            };
            #[cfg(test)]
            drop_observation::planned();
            // Every admission callback and fallible allocation runs unlocked.
            // Its temporary release node has no queue authority before attach.
            let prepared = self.prepare_registration(
                plan,
                RegistryKind::Page,
                identity,
                size_of::<PageCustody>(),
                admit,
            )?;
            let mut state = self.lock();
            state.reconcile_releases();
            if let Some(LiveCustody::Page(custody)) = state.live(RegistryKind::Page, identity) {
                drop(state);
                drop(prepared);
                return Ok(RegisteredPage {
                    semantic: pending.semantic,
                    custody,
                });
            }
            if missing_registry_shift(state.root(RegistryKind::Page), identity.key(), ID_SHIFT)
                != prepared.plan.missing
            {
                drop(state);
                drop(prepared);
                continue;
            }
            let custody = Arc::new(PageCustody {
                _payload: pending.payload,
                identity,
                edges: pending.edges,
                release: ReleaseTicket {
                    owner: Arc::downgrade(&self.state),
                    node: Some(prepared.node),
                },
                debit: pending.debit,
            });
            #[cfg(test)]
            metric(Event::StructuralAllocation {
                bytes: size_of::<PageCustody>(),
            });
            commit_registration(
                &mut state,
                RegistryKind::Page,
                identity,
                WeakCustody::Page(Arc::downgrade(&custody)),
                prepared.path,
                prepared.backing,
                prepared.plan.nonce,
            );
            drop(state);
            return Ok(RegisteredPage {
                semantic: pending.semantic,
                custody,
            });
        }
    }

    fn register_record(
        &self,
        identity: MaterializationId,
        debit: SharedDebit,
        dependency: Option<Dependency>,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Arc<RecordCustody>, DomError> {
        assert!(
            debit.belongs_to(&self.ledger),
            "record admission uses its controlled domain"
        );
        assert!(
            dependency
                .as_ref()
                .is_none_or(|dependency| dependency.debit.belongs_to(&self.ledger)),
            "a record dependency uses its controlled domain"
        );
        if let Some(existing) = self.find_record(identity) {
            return Ok(existing);
        }
        let administration = self.ledger.retain(
            Usage::new(
                size_of::<RecordCustody>() + size_of::<Mutex<RegistryState>>(),
                0,
            ),
            &mut *admit,
        )?;
        loop {
            let plan = {
                let mut state = self.lock();
                state.reconcile_releases();
                if let Some(LiveCustody::Record(existing)) =
                    state.live(RegistryKind::Record, identity)
                {
                    drop(state);
                    return Ok(existing);
                }
                state.plan(RegistryKind::Record, identity)?
            };
            #[cfg(test)]
            drop_observation::planned();
            let prepared = self.prepare_registration(
                plan,
                RegistryKind::Record,
                identity,
                size_of::<RecordCustody>(),
                admit,
            )?;
            let mut state = self.lock();
            state.reconcile_releases();
            if let Some(LiveCustody::Record(existing)) = state.live(RegistryKind::Record, identity)
            {
                drop(state);
                drop(prepared);
                return Ok(existing);
            }
            if missing_registry_shift(state.root(RegistryKind::Record), identity.key(), ID_SHIFT)
                != prepared.plan.missing
            {
                drop(state);
                drop(prepared);
                continue;
            }
            let custody = Arc::new(RecordCustody {
                identity,
                dependency,
                attachment: Mutex::new(None),
                _payload: debit,
                release: ReleaseTicket {
                    owner: Arc::downgrade(&self.state),
                    node: Some(prepared.node),
                },
                _administration: administration,
            });
            #[cfg(test)]
            metric(Event::StructuralAllocation {
                bytes: size_of::<RecordCustody>(),
            });
            commit_registration(
                &mut state,
                RegistryKind::Record,
                identity,
                WeakCustody::Record(Arc::downgrade(&custody)),
                prepared.path,
                prepared.backing,
                prepared.plan.nonce,
            );
            drop(state);
            return Ok(custody);
        }
    }
}

struct PreparedRegistration {
    path: Option<Box<RegistryNode>>,
    backing: Debit,
    node: Box<ReleaseNode>,
    plan: RegistrationPlan,
}

#[derive(Clone, Copy)]
struct RegistrationPlan {
    nonce: u64,
    missing: Option<u32>,
}

fn commit_registration(
    state: &mut RegistryState,
    kind: RegistryKind,
    identity: MaterializationId,
    custody: WeakCustody,
    path: Option<Box<RegistryNode>>,
    backing: Debit,
    nonce: u64,
) {
    let root = state.root_mut(kind);
    attach_registry_path(root, identity.key(), ID_SHIFT, path);
    let entry = registry_entry_mut(root, identity.key(), ID_SHIFT);
    assert!(
        entry.is_none(),
        "new custody identity has no active registry entry"
    );
    *entry = Some(RegistryEntry {
        nonce,
        custody,
        _backing: backing,
    });
}

fn registry_entry(
    root: &Option<Box<RegistryNode>>,
    key: u64,
    shift: u32,
) -> Option<&RegistryEntry> {
    #[cfg(test)]
    if root.is_some() {
        metric(Event::BankPageVisited);
    }
    match root.as_deref()? {
        RegistryNode::Leaf { entries, .. } => entries[(key & MASK as u64) as usize].as_ref(),
        RegistryNode::Branch { children, .. } => registry_entry(
            &children[((key >> shift) & MASK as u64) as usize],
            key,
            shift - BITS,
        ),
    }
}

fn registry_entry_mut(
    root: &mut Option<Box<RegistryNode>>,
    key: u64,
    shift: u32,
) -> &mut Option<RegistryEntry> {
    #[cfg(test)]
    metric(Event::BankPageVisited);
    match root
        .as_deref_mut()
        .expect("prepared registry path is attached")
    {
        RegistryNode::Leaf { entries, .. } => &mut entries[(key & MASK as u64) as usize],
        RegistryNode::Branch { children, .. } => registry_entry_mut(
            &mut children[((key >> shift) & MASK as u64) as usize],
            key,
            shift - BITS,
        ),
    }
}

fn missing_registry_shift(root: &Option<Box<RegistryNode>>, key: u64, shift: u32) -> Option<u32> {
    #[cfg(test)]
    if root.is_some() {
        metric(Event::BankPageVisited);
    }
    match root.as_deref() {
        None => Some(shift),
        Some(RegistryNode::Leaf { .. }) => None,
        Some(RegistryNode::Branch { children, .. }) => missing_registry_shift(
            &children[((key >> shift) & MASK as u64) as usize],
            key,
            shift - BITS,
        ),
    }
}

fn registry_slots<T>(
    ledger: &Ledger,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Held<Vec<Option<T>>>, DomError> {
    let requested = WIDTH
        .checked_mul(size_of::<Option<T>>())
        .ok_or(DomError::RepresentationExhausted)?;
    let mut debit = ledger.retain(
        Usage::new(checked_add(size_of::<RegistryNode>(), requested)?, 0),
        &mut *admit,
    )?;
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(WIDTH)
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    metric(Event::StructuralAllocation {
        bytes: slots.capacity() * size_of::<Option<T>>(),
    });
    if slots.capacity() > WIDTH {
        let excess = (slots.capacity() - WIDTH)
            .checked_mul(size_of::<Option<T>>())
            .ok_or(DomError::RepresentationExhausted)?;
        debit.grow(Usage::new(excess, 0), &mut *admit)?;
    }
    slots.resize_with(WIDTH, || None);
    Ok(Held::new(slots, debit))
}

fn prepare_registry_path(
    key: u64,
    shift: u32,
    ledger: &Ledger,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Box<RegistryNode>, DomError> {
    let (entries, debit) = registry_slots(ledger, admit)?.into_parts();
    let mut node = Box::new(RegistryNode::Leaf {
        entries,
        _debit: debit,
    });
    #[cfg(test)]
    metric(Event::StructuralAllocation {
        bytes: size_of::<RegistryNode>(),
    });
    for depth in (BITS..=shift).step_by(BITS as usize) {
        let (mut children, debit) = registry_slots(ledger, admit)?.into_parts();
        children[((key >> depth) & MASK as u64) as usize] = Some(node);
        node = Box::new(RegistryNode::Branch {
            children,
            _debit: debit,
        });
        #[cfg(test)]
        metric(Event::StructuralAllocation {
            bytes: size_of::<RegistryNode>(),
        });
    }
    Ok(node)
}

fn attach_registry_path(
    root: &mut Option<Box<RegistryNode>>,
    key: u64,
    shift: u32,
    path: Option<Box<RegistryNode>>,
) {
    #[cfg(test)]
    if root.is_some() {
        metric(Event::BankPageVisited);
    }
    if root.is_none() {
        *root = path;
        return;
    }
    if let RegistryNode::Branch { children, .. } =
        root.as_deref_mut().expect("present registry path")
    {
        attach_registry_path(
            &mut children[((key >> shift) & MASK as u64) as usize],
            key,
            shift - BITS,
            path,
        );
    } else {
        assert!(
            path.is_none(),
            "existing registry leaf requires no new path"
        );
    }
}

fn remove_registry_entry(root: &mut Option<Box<RegistryNode>>, key: u64, shift: u32, nonce: u64) {
    let Some(node) = root.as_deref_mut() else {
        return;
    };
    #[cfg(test)]
    metric(Event::BankPageVisited);
    let empty = match node {
        RegistryNode::Leaf { entries, .. } => {
            let entry = &mut entries[(key & MASK as u64) as usize];
            if entry.as_ref().is_some_and(|entry| entry.nonce == nonce) {
                *entry = None;
            }
            entries.iter().all(Option::is_none)
        }
        RegistryNode::Branch { children, .. } => {
            remove_registry_entry(
                &mut children[((key >> shift) & MASK as u64) as usize],
                key,
                shift - BITS,
                nonce,
            );
            children.iter().all(Option::is_none)
        }
    };
    if empty {
        *root = None;
    }
}

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

    pub fn origin_path_index_charge(
        &self,
        index: usize,
        origin: MaterializationId,
    ) -> Result<IndexCharge, DomError> {
        #[cfg(test)]
        metric(Event::Entered(Domain::Bank));
        let Some(mut page) = self.root.as_deref() else {
            return Ok(IndexCharge::default());
        };
        #[cfg(test)]
        metric(Event::BankPageVisited);
        if !fits(index, page.shift) {
            return Ok(IndexCharge::default());
        }
        let mut total = IndexCharge::default();
        loop {
            if page.producer_origin == Some(origin) {
                total = total.checked_add(IndexCharge {
                    bytes: page_bytes(page)?,
                    pages: 1,
                })?;
            }
            let Contents::Branch(children) = &page.contents else {
                return Ok(total);
            };
            let Some(child) = children[(index >> page.shift) & MASK].as_deref() else {
                return Ok(total);
            };
            page = child;
            #[cfg(test)]
            metric(Event::BankPageVisited);
        }
    }

    /// Compare only changed persistent paths; identical subtrees stop at Arc.
    pub fn same_records_by(&self, other: &Self, equal: &impl Fn(&T, &T) -> bool) -> bool {
        #[cfg(test)]
        metric(Event::Entered(Domain::Bank));
        fn same<T>(
            a: Option<&Arc<Page<T>>>,
            b: Option<&Arc<Page<T>>>,
            equal: &impl Fn(&T, &T) -> bool,
        ) -> bool {
            // Each present operand is inspected. An identical Arc prunes
            // before reading contents, but still counts that identity visit.
            #[cfg(test)]
            {
                if a.is_some() {
                    metric(Event::BankPageVisited);
                }
                if b.is_some_and(|b| !a.is_some_and(|a| Arc::ptr_eq(a, b))) {
                    metric(Event::BankPageVisited);
                }
            }
            match (a, b) {
                (None, None) => true,
                (Some(a), Some(b)) if Arc::ptr_eq(a, b) => true,
                (Some(a), Some(b)) => {
                    if a.summary.records != b.summary.records {
                        return false;
                    }
                    if a.shift > b.shift {
                        let Contents::Branch(children) = &a.contents else {
                            unreachable!()
                        };
                        return children.iter().skip(1).flatten().all(|page| {
                            #[cfg(test)]
                            metric(Event::BankPageVisited);
                            page.summary.records == 0
                        }) && same(children[0].as_ref(), Some(b), equal);
                    }
                    if b.shift > a.shift {
                        let Contents::Branch(children) = &b.contents else {
                            unreachable!()
                        };
                        return children.iter().skip(1).flatten().all(|page| {
                            #[cfg(test)]
                            metric(Event::BankPageVisited);
                            page.summary.records == 0
                        }) && same(Some(a), children[0].as_ref(), equal);
                    }
                    match (&a.contents, &b.contents) {
                        (Contents::Leaf(a), Contents::Leaf(b)) => {
                            a.iter().zip(b).all(|(a, b)| match (a, b) {
                                (None, None) => true,
                                (Some(a), Some(b)) => {
                                    #[cfg(test)]
                                    {
                                        metric(Event::CanonicalRecordRead);
                                        metric(Event::CanonicalRecordRead);
                                    }
                                    Arc::ptr_eq(&a.value, &b.value) || equal(&a.value, &b.value)
                                }
                                _ => false,
                            })
                        }
                        (Contents::Branch(a), Contents::Branch(b)) => a
                            .iter()
                            .zip(b)
                            .all(|(a, b)| same(a.as_ref(), b.as_ref(), equal)),
                        _ => unreachable!(),
                    }
                }
                (Some(page), None) | (None, Some(page)) => page.summary.records == 0,
            }
        }
        same(self.root.as_ref(), other.root.as_ref(), equal)
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
    pub fn path_index_charge_not_shared_with(&self, base: &Self, index: usize) -> IndexCharge {
        #[cfg(test)]
        metric(Event::Entered(Domain::Bank));
        let Some(root) = &self.root else {
            return IndexCharge::default();
        };
        if !fits(index, root.shift) {
            #[cfg(test)]
            metric(Event::BankPageVisited);
            return IndexCharge::default();
        }
        unshared_index(root, base.root.as_ref(), Some(index))
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

    #[cfg(test)]
    pub fn record(&self, index: usize) -> Option<Record<T>> {
        self.entry(index).map(|entry| Record {
            value: entry.value.clone(),
            identity: entry.identity,
        })
    }

    #[cfg(test)]
    pub fn totals_at(&self, index: usize) -> Option<Totals> {
        self.entry(index).map(|entry| entry.totals)
    }

    fn entry(&self, index: usize) -> Option<&Entry<T>> {
        #[cfg(test)]
        {
            metric(Event::Entered(Domain::Bank));
            metric(Event::BankLookup);
        }
        if index >= self.extent {
            return None;
        }
        let mut page = self.root.as_deref()?;
        if !fits(index, page.shift) {
            return None;
        }
        loop {
            #[cfg(test)]
            metric(Event::BankPageVisited);
            match &page.contents {
                Contents::Leaf(entries) => {
                    let entry = entries[index & MASK].as_ref();
                    #[cfg(test)]
                    if entry.is_some() {
                        metric(Event::CanonicalRecordRead);
                    }
                    return entry;
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
        value: T,
        reusable: bool,
        totals: Totals,
    ) -> Result<IndexCharge, DomError> {
        self.set_admitted(index, Record::new(value)?, reusable, totals, |_| Ok(()))
    }

    /// Admit page values and backing before allocation, then any actual reserve
    /// excess immediately after allocation, matching the owner's Vec policy.
    /// Every returned allocation remains in the resulting root; root growth
    /// includes retained leading-zero wrappers outside the edited path.
    pub fn set_admitted(
        &mut self,
        index: usize,
        record: Record<T>,
        reusable: bool,
        totals: Totals,
        mut admit: impl FnMut(IndexCharge) -> Result<(), DomError>,
    ) -> Result<IndexCharge, DomError> {
        self.set_with(index, record, reusable, totals, None, |charge| {
            admit(charge)?;
            Ok::<_, DomError>(None)
        })
    }

    /// Retain each newly allocated page in its original hosted issuer. The
    /// callback supplies the page/header receipt before allocation and any
    /// allocator-exposed capacity excess before retained fields are copied.
    pub fn set_hosted_admitted(
        &mut self,
        index: usize,
        record: Record<T>,
        reusable: bool,
        totals: Totals,
        origin: MaterializationId,
        mut admit: impl FnMut(
            IndexCharge,
        ) -> Result<crate::hosted::Retained, crate::hosted::DomHostedError>,
    ) -> Result<IndexCharge, crate::hosted::DomHostedError> {
        self.set_with(index, record, reusable, totals, Some(origin), |charge| {
            admit(charge).map(Some)
        })
    }

    fn set_with<E: From<DomError>>(
        &mut self,
        index: usize,
        record: Record<T>,
        reusable: bool,
        totals: Totals,
        origin: Option<MaterializationId>,
        mut admit: impl FnMut(IndexCharge) -> Result<Option<crate::hosted::Retained>, E>,
    ) -> Result<IndexCharge, E> {
        let extent = checked_add(index, 1)?.max(self.extent);
        let mut root = self.root.as_ref().map(Arc::clone);
        let mut shift = root.as_ref().map_or(0, |page| page.shift);
        let mut growth = IndexCharge::default();
        while !fits(index, shift) {
            shift += BITS;
        }
        while root.as_ref().is_some_and(|page| page.shift + BITS < shift) {
            if let Some(previous) = root.take() {
                let identity = MaterializationId::fresh()?;
                let next_shift = previous.shift + BITS;
                let mut children = empty_slots_with(size_of::<Page<T>>(), &mut admit)?;
                children[0] = Some(previous);
                let (page, charge) = make_page_with_producer(
                    identity,
                    next_shift,
                    Contents::Branch(children.slots),
                    origin,
                    children.producer,
                )?;
                root = Some(page);
                growth = growth.checked_add(charge)?;
            }
        }
        let entry = Entry {
            value: record.value,
            identity: record.identity,
            reusable,
            totals,
        };
        let (root, charge) =
            replace_with(root.as_ref(), shift, index, Some(entry), origin, &mut admit)?;
        let charge = growth.checked_add(charge)?;
        self.root = root;
        self.extent = extent;
        Ok(charge)
    }

    /// Compute a replacement from the borrowed current record without cloning
    /// unrelated records. A callback refusal preserves the original bank.
    #[cfg(test)]
    pub fn update(
        &mut self,
        index: usize,
        change: impl FnOnce(Option<&T>) -> Result<(Record<T>, bool, Totals), DomError>,
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
        #[cfg(test)]
        metric(Event::Entered(Domain::Bank));
        if self.is_empty() {
            return None;
        }
        let Some(root) = &self.root else {
            return Some(0);
        };
        #[cfg(test)]
        metric(Event::VacancyPageVisited);
        let candidate = root
            .summary
            .vacant
            .or_else(|| 1usize.checked_shl(root.shift + BITS))?;
        (candidate < self.extent).then_some(candidate)
    }

    /// Present records in exact ascending slot order, without visiting holes.
    pub fn iter(&self) -> Iter<'_, T> {
        #[cfg(test)]
        metric(Event::Entered(Domain::Bank));
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

    fn assert_custody(&self, custody: &BankCustody<T>, registry: &CustodyRegistry) {
        match (&self.root, &custody.root) {
            (None, None) => {}
            (Some(page), Some(control)) => {
                assert_eq!(
                    page.identity, control.identity,
                    "semantic and custody roots are paired"
                );
                assert!(
                    control.debit.belongs_to(registry.ledger()),
                    "paired roots share their admission domain"
                );
                assert!(
                    Weak::ptr_eq(&control.release.owner, &Arc::downgrade(&registry.state)),
                    "paired roots share their custody registry"
                );
            }
            _ => panic!("semantic and custody roots are paired"),
        }
    }

    /// Alias an already controlled record, preserving its materialization ID.
    /// Any retained output-holder allocation is admitted by its owning caller.
    pub fn controlled_record(
        &self,
        custody: &BankCustody<T>,
        index: usize,
    ) -> Option<ControlledRecord<T>> {
        let entry = self.entry(index)?;
        let mut page = self.root.as_deref()?;
        let mut control = custody.root.as_deref()?;
        loop {
            assert_eq!(
                page.identity, control.identity,
                "record lookup follows matching custody paths"
            );
            match (&page.contents, &control.edges) {
                (Contents::Leaf(_), CustodyEdges::Leaf(records)) => {
                    let retained = records[index & MASK]
                        .as_ref()
                        .expect("present record has custody");
                    assert_eq!(
                        entry.identity, retained.identity,
                        "record lookup preserves materialization identity"
                    );
                    return Some(ControlledRecord {
                        record: Record {
                            value: entry.value.clone(),
                            identity: entry.identity,
                        },
                        custody: RecordRetention::Registered(retained.clone()),
                    });
                }
                (Contents::Branch(children), CustodyEdges::Branch(controls)) => {
                    let digit = (index >> page.shift) & MASK;
                    page = children[digit]
                        .as_deref()
                        .expect("record has semantic index path");
                    control = controls[digit]
                        .as_deref()
                        .expect("record has custody index path");
                }
                _ => unreachable!("semantic and custody page kinds match"),
            }
        }
    }

    /// Construct a complete new semantic/custody path pair. The original pair
    /// remains unchanged on failure and retains all required overlap. The
    /// returned tuple destroys its semantic root before its custody root.
    #[expect(
        clippy::too_many_arguments,
        reason = "The paired bank, affine record, slot metadata and admission context enter one atomic path construction."
    )]
    pub fn set_controlled(
        &self,
        custody: &BankCustody<T>,
        index: usize,
        record: ControlledRecord<T>,
        reusable: bool,
        totals: Totals,
        registry: &CustodyRegistry,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(Self, BankCustody<T>), DomError> {
        self.assert_custody(custody, registry);
        let result = (|| {
            let extent = checked_add(index, 1)?.max(self.extent);
            let retained = match &record.custody {
                RecordRetention::Pending { debit, dependency } => registry.register_record(
                    record.record.identity,
                    debit.clone(),
                    dependency.clone(),
                    &mut admit,
                )?,
                RecordRetention::Registered(retained) => {
                    assert!(
                        Weak::ptr_eq(&retained.release.owner, &Arc::downgrade(&registry.state)),
                        "record belongs to this controlled registry"
                    );
                    retained.clone()
                }
            };
            let mut root = self
                .root
                .as_ref()
                .zip(custody.root.as_ref())
                .map(|(page, control)| RegisteredPage {
                    semantic: page.clone(),
                    custody: control.clone(),
                });
            let mut shift = root.as_ref().map_or(0, |pair| pair.semantic.shift);
            while !fits(index, shift) {
                shift += BITS;
            }
            while root
                .as_ref()
                .is_some_and(|pair| pair.semantic.shift + BITS < shift)
            {
                let identity = MaterializationId::fresh()?;
                let mut slots =
                    controlled_slots::<T, Arc<Page<T>>, Arc<PageCustody>>(registry, &mut admit)?;
                let previous = root.take().expect("present growth basis");
                let next_shift = previous.semantic.shift + BITS;
                slots.values[0] = Some(previous.semantic);
                slots.controls[0] = Some(previous.custody);
                root = Some(make_controlled_page(
                    identity,
                    next_shift,
                    PendingPage {
                        payload: None,
                        semantic: Contents::Branch(slots.values),
                        edges: CustodyEdges::Branch(slots.controls),
                        debit: slots.debit,
                    },
                    registry,
                    &mut admit,
                )?);
            }
            let entry = Entry {
                value: record.record.value.clone(),
                identity: record.record.identity,
                reusable,
                totals,
            };
            let replacement = replace_controlled(
                root.as_ref().map(|pair| (&pair.semantic, &pair.custody)),
                shift,
                index,
                Some((entry, retained)),
                registry,
                &mut admit,
            )?;
            let (root, control) = match replacement {
                Some(pair) => (Some(pair.semantic), Some(pair.custody)),
                None => (None, None),
            };
            Ok((
                Self { root, extent },
                BankCustody {
                    root: control,
                    record: PhantomData,
                },
            ))
        })();
        drop(record);
        registry.reconcile_releases();
        result
    }

    /// Remove one entry while retaining historical extent and returning an
    /// admitted alias of the removed record. All mutation remains transactional.
    #[expect(
        clippy::type_complexity,
        reason = "The returned tuple keeps semantic storage before matching custody and its removed affine alias in destruction order."
    )]
    pub fn remove_controlled(
        &self,
        custody: &BankCustody<T>,
        index: usize,
        registry: &CustodyRegistry,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(Self, BankCustody<T>, Option<ControlledRecord<T>>), DomError> {
        self.assert_custody(custody, registry);
        let result = (|| {
            let Some(removed) = self.controlled_record(custody, index) else {
                return Ok((self.clone(), custody.clone(), None));
            };
            let previous = self.root.as_ref().expect("removed record has a root");
            let control = custody.root.as_ref().expect("removed record has custody");
            let replacement = replace_controlled(
                Some((previous, control)),
                previous.shift,
                index,
                None,
                registry,
                &mut admit,
            )?;
            let (root, control) = match replacement {
                Some(pair) => (Some(pair.semantic), Some(pair.custody)),
                None => (None, None),
            };
            Ok((
                Self {
                    root,
                    extent: self.extent,
                },
                BankCustody {
                    root: control,
                    record: PhantomData,
                },
                Some(removed),
            ))
        })();
        registry.reconcile_releases();
        result
    }

    /// Borrow a historical input throughout reconciliation, pruning every
    /// already controlled materialization. Only after this succeeds may its
    /// caller retain the semantic root alongside this returned custody root.
    /// Reuse that admitted capture across all frames/continuations/savepoints.
    pub fn adopt_controlled<D: Into<RecordDescription>>(
        &self,
        registry: &CustodyRegistry,
        describe: impl FnMut(&T) -> Result<D, DomError>,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<BankCustody<T>, DomError> {
        self.adopt_controlled_from(registry, describe, |_, _, _| Ok(None), admit)
    }

    /// Adopt an exact new frontier from a complete caller-held receipt pool.
    /// The caller retains an alias of every supplied payload until its semantic
    /// candidate and returned custody root transfer together, including refusal.
    pub fn adopt_controlled_from<D: Into<RecordDescription>>(
        &self,
        registry: &CustodyRegistry,
        mut describe: impl FnMut(&T) -> Result<D, DomError>,
        mut retained_page: impl FnMut(
            MaterializationId,
            Option<MaterializationId>,
            Usage,
        ) -> Result<Option<SharedDebit>, DomError>,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<BankCustody<T>, DomError> {
        #[cfg(test)]
        metric(Event::Entered(Domain::Accounting));
        let result = (|| {
            admit(registry.total())?;
            let root = self
                .root
                .as_ref()
                .map(|page| {
                    adopt_page_from(
                        page,
                        registry,
                        &mut describe,
                        &mut retained_page,
                        &mut admit,
                    )
                })
                .transpose()?;
            Ok(BankCustody {
                root,
                record: PhantomData,
            })
        })();
        registry.reconcile_releases();
        result
    }

    /// Independent test oracle: inspect actual allocations/record Arc identity,
    /// capacities and caller-described payloads, without cached Summary/Totals
    /// or any custody token/ledger values.
    #[cfg(test)]
    pub fn inspect_graph(
        &self,
        mut describe: impl FnMut(&T) -> Result<Totals, DomError>,
    ) -> Result<PhysicalGraph, DomError> {
        let mut graph = PhysicalGraph::default();
        if let Some(root) = &self.root {
            inspect_page(root, &mut graph, &mut describe)?;
        }
        graph.checked_index_charge()?;
        graph.checked_record_totals()?;
        Ok(graph)
    }

    #[cfg(test)]
    pub(in crate::store) fn inspect_raw_allocations(
        &self,
        graph: &mut super::compact_tests::AllocationGraph,
    ) {
        fn visit<T>(page: &Arc<Page<T>>, graph: &mut super::compact_tests::AllocationGraph) {
            graph.target(page, "bank.page");
            match &page.contents {
                Contents::Leaf(entries) => {
                    graph.buffer(entries, "bank.leaf");
                    for entry in entries.iter().flatten() {
                        graph.target(&entry.value, "bank.record");
                    }
                }
                Contents::Branch(children) => {
                    graph.buffer(children, "bank.branch");
                    for child in children.iter().flatten() {
                        visit(child, graph);
                    }
                }
            }
        }
        if let Some(root) = &self.root {
            visit(root, graph);
        }
    }

    #[cfg(test)]
    pub fn inspect_origin_pages(
        &self,
    ) -> Result<Vec<(MaterializationId, Option<MaterializationId>, Usage)>, DomError> {
        fn visit<T>(
            page: &Page<T>,
            output: &mut Vec<(MaterializationId, Option<MaterializationId>, Usage)>,
        ) -> Result<(), DomError> {
            output.push((
                page.identity,
                page.producer_origin,
                Usage::new(page_bytes(page)?, 0),
            ));
            if let Contents::Branch(children) = &page.contents {
                for child in children.iter().flatten() {
                    visit(child, output)?;
                }
            }
            Ok(())
        }
        let mut output = Vec::new();
        if let Some(root) = self.root.as_deref() {
            visit(root, &mut output)?;
        }
        Ok(output)
    }
}

/// Test-only physical graph union. Address keys are short-lived observations of
/// strongly retained objects, never production identity or ownership policy.
#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct PhysicalGraph {
    pages: std::collections::BTreeMap<usize, (MaterializationId, IndexCharge)>,
    records: std::collections::BTreeMap<usize, (MaterializationId, Totals)>,
}

#[cfg(test)]
impl PhysicalGraph {
    fn checked_index_charge(&self) -> Result<IndexCharge, DomError> {
        self.pages
            .values()
            .try_fold(IndexCharge::default(), |sum, (_, own)| {
                sum.checked_add(*own)
            })
    }

    fn checked_record_totals(&self) -> Result<Totals, DomError> {
        self.records
            .values()
            .try_fold(Totals::default(), |sum, (_, own)| sum.checked_add(*own))
    }

    pub fn index_charge(&self) -> IndexCharge {
        self.checked_index_charge()
            .expect("physical graph charges were checked during collection/merge")
    }

    pub fn materializations(&self) -> impl Iterator<Item = (MaterializationId, Usage)> + '_ {
        self.pages
            .values()
            .map(|(id, own)| (*id, Usage::new(own.bytes, 0)))
            .chain(
                self.records
                    .values()
                    .map(|(id, own)| (*id, Usage::new(own.bytes, own.units))),
            )
    }

    pub fn record_totals(&self) -> Totals {
        self.checked_record_totals()
            .expect("physical graph receipts were checked during collection/merge")
    }

    pub fn pages(&self) -> impl Iterator<Item = (usize, u64, IndexCharge)> + '_ {
        self.pages
            .iter()
            .map(|(address, (identity, charge))| (*address, identity.key(), *charge))
    }

    pub fn records(&self) -> impl Iterator<Item = (usize, u64, Totals)> + '_ {
        self.records
            .iter()
            .map(|(address, (identity, charge))| (*address, identity.key(), *charge))
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), DomError> {
        let mut index = self.checked_index_charge()?;
        let mut records = self.checked_record_totals()?;
        for (address, (identity, charge)) in &other.pages {
            if let Some(existing) = self.pages.get(address) {
                assert_eq!(
                    existing,
                    &(*identity, *charge),
                    "one live Page allocation has one identity/physical extent"
                );
            } else {
                index = index.checked_add(*charge)?;
            }
        }
        for (address, (identity, charge)) in &other.records {
            if let Some(existing) = self.records.get(address) {
                assert_eq!(
                    existing,
                    &(*identity, *charge),
                    "one live record allocation has one identity/receipt"
                );
            } else {
                records = records.checked_add(*charge)?;
            }
        }
        self.pages
            .extend(other.pages.iter().map(|(key, value)| (*key, *value)));
        self.records
            .extend(other.records.iter().map(|(key, value)| (*key, *value)));
        let _checked_totals = (index, records);
        Ok(())
    }
}

#[cfg(test)]
fn inspect_page<T>(
    page: &Arc<Page<T>>,
    graph: &mut PhysicalGraph,
    describe: &mut impl FnMut(&T) -> Result<Totals, DomError>,
) -> Result<(), DomError> {
    let address = Arc::as_ptr(page) as usize;
    if graph.pages.contains_key(&address) {
        return Ok(());
    }
    let backing = match &page.contents {
        Contents::Leaf(entries) => entries
            .capacity()
            .checked_mul(size_of::<Option<Entry<T>>>()),
        Contents::Branch(children) => children
            .capacity()
            .checked_mul(size_of::<Option<Arc<Page<T>>>>()),
    }
    .ok_or(DomError::RepresentationExhausted)?;
    graph.pages.insert(
        address,
        (
            page.identity,
            IndexCharge {
                bytes: checked_add(size_of::<Page<T>>(), backing)?,
                pages: 1,
            },
        ),
    );
    match &page.contents {
        Contents::Leaf(entries) => {
            for entry in entries.iter().flatten() {
                let address = Arc::as_ptr(&entry.value) as usize;
                if let Some((identity, _)) = graph.records.get(&address) {
                    assert_eq!(
                        *identity, entry.identity,
                        "record aliases retain their materialization ID"
                    );
                } else {
                    graph
                        .records
                        .insert(address, (entry.identity, describe(&entry.value)?));
                }
            }
        }
        Contents::Branch(children) => {
            for child in children.iter().flatten() {
                inspect_page(child, graph, describe)?;
            }
        }
    }
    Ok(())
}

fn controlled_slots<T, V, C>(
    registry: &CustodyRegistry,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<ControlledSlots<V, C>, DomError> {
    let backing = WIDTH
        .checked_mul(size_of::<Option<V>>())
        .ok_or(DomError::RepresentationExhausted)?;
    let own = checked_add(size_of::<Page<T>>(), backing)?;
    let control = checked_add(size_of::<PageCustody>(), size_of::<Mutex<RegistryState>>())?;
    let mut debit = registry
        .ledger
        .retain(Usage::new(checked_add(own, control)?, 0), &mut *admit)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(WIDTH)
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    metric(Event::StructuralAllocation {
        bytes: values.capacity() * size_of::<Option<V>>(),
    });
    if values.capacity() > WIDTH {
        let excess = (values.capacity() - WIDTH)
            .checked_mul(size_of::<Option<V>>())
            .ok_or(DomError::RepresentationExhausted)?;
        debit.grow(Usage::new(excess, 0), &mut *admit)?;
    }
    values.resize_with(WIDTH, || None);
    Ok(ControlledSlots {
        values,
        controls: std::array::from_fn(|_| None),
        debit,
    })
}

fn make_controlled_page<T>(
    identity: MaterializationId,
    shift: u32,
    pending: PendingPage<Contents<T>>,
    registry: &CustodyRegistry,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<ControlledPage<T>, DomError> {
    let PendingPage {
        payload,
        semantic: contents,
        edges,
        debit,
    } = pending;
    let (page, _) = make_page(identity, shift, contents)?;
    registry.register_page(
        identity,
        PendingPage {
            payload,
            semantic: page,
            edges,
            debit,
        },
        admit,
    )
}

fn replace_controlled<T>(
    previous: Option<(&Arc<Page<T>>, &Arc<PageCustody>)>,
    shift: u32,
    index: usize,
    replacement: Option<(Entry<T>, Arc<RecordCustody>)>,
    registry: &CustodyRegistry,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<ControlledReplacement<T>, DomError> {
    #[cfg(test)]
    {
        metric(Event::Entered(Domain::Bank));
        if previous.is_some() {
            metric(Event::BankPageVisited);
        }
    }
    if replacement.is_none() && previous.is_some_and(|(page, _)| page.summary.records == 1) {
        return Ok(None);
    }
    let identity = MaterializationId::fresh()?;
    let pending = if shift == 0 {
        let mut slots = controlled_slots::<T, Entry<T>, Arc<RecordCustody>>(registry, admit)?;
        if let Some((old, custody)) = previous {
            assert_eq!(
                old.identity, custody.identity,
                "path copying uses paired materializations"
            );
            let (Contents::Leaf(old), CustodyEdges::Leaf(old_controls)) =
                (&old.contents, &custody.edges)
            else {
                unreachable!("zero shift has leaf pair")
            };
            slots.values.clone_from_slice(old);
            slots.controls.clone_from_slice(old_controls);
        }
        let digit = index & MASK;
        match replacement {
            Some((entry, custody)) => {
                slots.values[digit] = Some(entry);
                slots.controls[digit] = Some(custody);
            }
            None => {
                slots.values[digit] = None;
                slots.controls[digit] = None;
            }
        }
        PendingPage {
            payload: None,
            semantic: Contents::Leaf(slots.values),
            edges: CustodyEdges::Leaf(slots.controls),
            debit: slots.debit,
        }
    } else {
        let mut slots = controlled_slots::<T, Arc<Page<T>>, Arc<PageCustody>>(registry, admit)?;
        if let Some((old, custody)) = previous {
            assert_eq!(
                old.identity, custody.identity,
                "path copying uses paired materializations"
            );
            if old.shift == shift {
                let (Contents::Branch(old), CustodyEdges::Branch(old_controls)) =
                    (&old.contents, &custody.edges)
                else {
                    unreachable!("branch pair")
                };
                slots.values.clone_from_slice(old);
                slots.controls.clone_from_slice(old_controls);
            } else {
                debug_assert_eq!(old.shift + BITS, shift);
                slots.values[0] = Some(old.clone());
                slots.controls[0] = Some(custody.clone());
            }
        }
        let digit = (index >> shift) & MASK;
        let updated = replace_controlled(
            slots.values[digit]
                .as_ref()
                .zip(slots.controls[digit].as_ref()),
            shift - BITS,
            index,
            replacement,
            registry,
            admit,
        )?;
        match updated {
            Some(pair) => {
                slots.values[digit] = Some(pair.semantic);
                slots.controls[digit] = Some(pair.custody);
            }
            None => {
                slots.values[digit] = None;
                slots.controls[digit] = None;
            }
        }
        PendingPage {
            payload: None,
            semantic: Contents::Branch(slots.values),
            edges: CustodyEdges::Branch(slots.controls),
            debit: slots.debit,
        }
    };
    make_controlled_page(identity, shift, pending, registry, admit).map(Some)
}

fn page_bytes<T>(page: &Page<T>) -> Result<usize, DomError> {
    let backing = match &page.contents {
        Contents::Leaf(entries) => entries
            .capacity()
            .checked_mul(size_of::<Option<Entry<T>>>()),
        Contents::Branch(children) => children
            .capacity()
            .checked_mul(size_of::<Option<Arc<Page<T>>>>()),
    }
    .ok_or(DomError::RepresentationExhausted)?;
    checked_add(size_of::<Page<T>>(), backing)
}

fn adopt_page_from<T, D: Into<RecordDescription>>(
    page: &Arc<Page<T>>,
    registry: &CustodyRegistry,
    describe: &mut impl FnMut(&T) -> Result<D, DomError>,
    retained_page: &mut impl FnMut(
        MaterializationId,
        Option<MaterializationId>,
        Usage,
    ) -> Result<Option<SharedDebit>, DomError>,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Arc<PageCustody>, DomError> {
    #[cfg(test)]
    {
        metric(Event::Entered(Domain::Accounting));
        metric(Event::BankPageVisited);
    }
    if let Some(control) = registry.find_page(page.identity) {
        return Ok(control);
    }
    let control_bytes = checked_add(size_of::<PageCustody>(), size_of::<Mutex<RegistryState>>())?;
    let own_usage = Usage::new(page_bytes(page)?, 0);
    let payload = retained_page(page.identity, page.producer_origin, own_usage)?;
    if let Some(retained) = &payload {
        assert!(
            retained.belongs_to(registry.ledger()),
            "moved page uses original controlled domain"
        );
        assert_eq!(
            retained.usage(),
            own_usage.checked_add(SharedDebit::administration_usage())?
        );
    }
    let debit = registry.ledger.retain(
        Usage::new(control_bytes, 0).checked_add(if payload.is_none() {
            own_usage
        } else {
            Usage::ZERO
        })?,
        &mut *admit,
    )?;
    let edges = match &page.contents {
        Contents::Leaf(entries) => {
            let mut controls = std::array::from_fn(|_| None);
            for (index, entry) in entries.iter().enumerate() {
                let Some(entry) = entry else { continue };
                controls[index] = Some(match registry.find_record(entry.identity) {
                    Some(existing) => existing,
                    None => {
                        #[cfg(test)]
                        metric(Event::AccountingRecordVisited);
                        let description = describe(&entry.value)?.into();
                        description.assert_domain(registry.ledger());
                        let debit = match description.retained {
                            Some(retained) => retained,
                            None => registry
                                .ledger
                                .retain_shared(description.usage, &mut *admit)?,
                        };
                        registry.register_record(
                            entry.identity,
                            debit,
                            description.dependency,
                            admit,
                        )?
                    }
                });
            }
            CustodyEdges::Leaf(controls)
        }
        Contents::Branch(children) => {
            let mut controls = std::array::from_fn(|_| None);
            for (index, child) in children.iter().enumerate() {
                if let Some(child) = child {
                    controls[index] = Some(adopt_page_from(
                        child,
                        registry,
                        describe,
                        retained_page,
                        admit,
                    )?);
                }
            }
            CustodyEdges::Branch(controls)
        }
    };
    registry
        .register_page(
            page.identity,
            PendingPage {
                payload,
                semantic: (),
                edges,
                debit,
            },
            admit,
        )
        .map(|registered| registered.custody)
}

fn fits(index: usize, shift: u32) -> bool {
    index >> shift < WIDTH
}

fn unshared_index<T>(
    current: &Arc<Page<T>>,
    mut base: Option<&Arc<Page<T>>>,
    path: Option<usize>,
) -> IndexCharge {
    #[cfg(test)]
    metric(Event::BankPageVisited);
    // A smaller current root can only overlap the leading-zero subtree of a
    // larger basis. Root growth follows the same alignment in reverse below.
    while let Some(previous) = base {
        #[cfg(test)]
        if !Arc::ptr_eq(current, previous) {
            metric(Event::BankPageVisited);
        }
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

/// Data precedes its original producer receipt on every early return.
struct PageSlots<T> {
    slots: Vec<Option<T>>,
    producer: Option<crate::hosted::Retained>,
}
impl<T> std::ops::Deref for PageSlots<T> {
    type Target = Vec<Option<T>>;
    fn deref(&self) -> &Self::Target {
        &self.slots
    }
}
impl<T> std::ops::DerefMut for PageSlots<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slots
    }
}
fn empty_slots_with<T, E: From<DomError>>(
    page_bytes: usize,
    admit: &mut impl FnMut(IndexCharge) -> Result<Option<crate::hosted::Retained>, E>,
) -> Result<PageSlots<T>, E> {
    let requested = WIDTH
        .checked_mul(size_of::<Option<T>>())
        .ok_or(DomError::RepresentationExhausted)?;
    let producer = admit(IndexCharge {
        bytes: checked_add(page_bytes, requested)?,
        pages: 1,
    })?;
    let mut candidate = PageSlots {
        slots: Vec::new(),
        producer,
    };
    candidate
        .slots
        .try_reserve_exact(WIDTH)
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    metric(Event::StructuralAllocation {
        bytes: candidate.slots.capacity() * size_of::<Option<T>>(),
    });
    if candidate.slots.capacity() > WIDTH {
        let extra = (candidate.slots.capacity() - WIDTH)
            .checked_mul(size_of::<Option<T>>())
            .ok_or(DomError::RepresentationExhausted)?;
        let receipt = admit(IndexCharge {
            bytes: extra,
            pages: 0,
        })?;
        match (&mut candidate.producer, receipt) {
            (Some(original), Some(extra)) => original
                .append(extra)
                .expect("one page callback retains one original hosted issuer"),
            (None, None) => {}
            _ => unreachable!("the typed page frontend fixes its producer domain"),
        }
    }
    candidate.slots.resize_with(WIDTH, || None);
    Ok(candidate)
}

/// Recursion is bounded by machine index width / BITS, never by record count.
fn replace<T>(
    previous: Option<&Arc<Page<T>>>,
    shift: u32,
    index: usize,
    replacement: Option<Entry<T>>,
    admit: &mut impl FnMut(IndexCharge) -> Result<(), DomError>,
) -> Result<PageReplacement<T>, DomError> {
    replace_with(previous, shift, index, replacement, None, &mut |charge| {
        admit(charge)?;
        Ok::<_, DomError>(None)
    })
}

fn replace_with<T, E: From<DomError>>(
    previous: Option<&Arc<Page<T>>>,
    shift: u32,
    index: usize,
    replacement: Option<Entry<T>>,
    origin: Option<MaterializationId>,
    admit: &mut impl FnMut(IndexCharge) -> Result<Option<crate::hosted::Retained>, E>,
) -> Result<PageReplacement<T>, E> {
    #[cfg(test)]
    {
        metric(Event::Entered(Domain::Bank));
        if previous.is_some() {
            metric(Event::BankPageVisited);
        }
    }
    if replacement.is_none() && previous.is_some_and(|page| page.summary.records == 1) {
        return Ok((None, IndexCharge::default()));
    }
    let identity = MaterializationId::fresh()?;
    let (contents, producer, descendant_charge) = if shift == 0 {
        let mut entries = empty_slots_with(size_of::<Page<T>>(), admit)?;
        if let Some(previous) = previous {
            let Contents::Leaf(old) = &previous.contents else {
                unreachable!("zero shift identifies a leaf");
            };
            entries.clone_from_slice(old);
        }
        entries[index & MASK] = replacement;
        (
            Contents::Leaf(entries.slots),
            entries.producer,
            IndexCharge::default(),
        )
    } else {
        let mut children = empty_slots_with(size_of::<Page<T>>(), admit)?;
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
        let (child, charge) = replace_with(
            children[digit].as_ref(),
            shift - BITS,
            index,
            replacement,
            origin,
            admit,
        )?;
        children[digit] = child;
        (Contents::Branch(children.slots), children.producer, charge)
    };
    let (page, own) = make_page_with_producer(identity, shift, contents, origin, producer)?;
    let charge = own.checked_add(descendant_charge)?;
    Ok((Some(page), charge))
}

fn make_page<T>(
    identity: MaterializationId,
    shift: u32,
    contents: Contents<T>,
) -> Result<(Arc<Page<T>>, IndexCharge), DomError> {
    make_page_with_producer(identity, shift, contents, None, None)
}

fn make_page_with_producer<T>(
    identity: MaterializationId,
    shift: u32,
    contents: Contents<T>,
    producer_origin: Option<MaterializationId>,
    producer: Option<crate::hosted::Retained>,
) -> Result<(Arc<Page<T>>, IndexCharge), DomError> {
    let mut candidate = Page {
        identity,
        shift,
        contents,
        summary: Summary::default(),
        producer_origin,
        _producer: producer,
    };
    let own = IndexCharge {
        bytes: checked_add(
            size_of::<Page<T>>(),
            match &candidate.contents {
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
    #[cfg(test)]
    if super::scale_metrics::ENABLED {
        scale_verify_page::<T>(own.bytes);
    }
    match &candidate.contents {
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
    candidate.summary = summary;
    let page = Arc::new(candidate);
    #[cfg(test)]
    metric(Event::StructuralAllocation {
        bytes: size_of::<Page<T>>(),
    });
    Ok((page, own))
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
            #[cfg(test)]
            if frame.next == 0 {
                metric(Event::BankPageVisited);
            }
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
                        #[cfg(test)]
                        metric(Event::CanonicalRecordRead);
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
            value,
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
                    Record::new(30)?,
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
            bank.set(usize::MAX, 1, false, Totals::default()),
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
            1,
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
                2,
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
        let refused = bank.set_admitted(
            1,
            Record::new(30).unwrap(),
            false,
            Totals::default(),
            |charge| {
                pages += charge.pages;
                if pages == 2 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
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
    fn record_comparison_prunes_shared_paths_and_preserves_sparse_semantics() {
        let mut basis = Bank::default();
        put(&mut basis, 0, 11, false);
        put(&mut basis, 15, 19, false);
        let mut same = basis.clone();
        put(&mut same, 0, 11, false);
        assert!(basis.same_records_by(&same, &u64::eq));
        put(&mut same, 4096, 23, false);
        assert!(!basis.same_records_by(&same, &u64::eq));
        same.remove(4096).unwrap();
        assert!(basis.same_records_by(&same, &u64::eq));
        assert!(same.same_records_by(&basis, &u64::eq));
        put(&mut same, 15, 29, false);
        assert!(!basis.same_records_by(&same, &u64::eq));
        let mut different_slot = Bank::default();
        put(&mut different_slot, 1, 11, false);
        put(&mut different_slot, 15, 19, false);
        assert!(!basis.same_records_by(&different_slot, &u64::eq));
    }

    #[test]
    fn record_comparison_visits_only_changed_paths_among_unrelated_records() {
        let mut basis = Bank::default();
        for index in 0..4096 {
            put(&mut basis, index, index as u64, false);
        }
        let mut next = basis.clone();
        put(&mut next, 17, 17, false);
        assert!(basis.same_records_by(&next, &u64::eq));
        if !scale_metrics::ENABLED {
            return;
        }
        let observation = scale_metrics::Measurement::begin().unwrap();
        assert!(basis.same_records_by(&next, &u64::eq));
        let counts = observation
            .finish(scale_metrics::Hooks::one(Domain::Bank))
            .unwrap();
        // Three radix levels: each changed branch compares at most WIDTH
        // handles, and only one fixed-size leaf inspects actual records.
        assert!(counts.bank_page_visits > 0);
        assert!(counts.bank_page_visits <= 3 * (WIDTH + 1));
        assert_eq!(counts.canonical_record_reads, 2 * WIDTH);
        let observation = scale_metrics::Measurement::begin().unwrap();
        assert!(basis.same_records_by(&basis.clone(), &u64::eq));
        let counts = observation
            .finish(scale_metrics::Hooks::one(Domain::Bank))
            .unwrap();
        assert_eq!(counts.bank_page_visits, 1);
        assert_eq!(counts.canonical_record_reads, 0);
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
                Observed(Arc::clone(&drops)),
                false,
                Totals::default(),
            )
            .unwrap();
        }
        let captured = bank.clone();
        bank.set(0, Observed(Arc::clone(&drops)), false, Totals::default())
            .unwrap();
        assert_eq!(drops.load(Ordering::Relaxed), 0);
        drop(captured);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        assert_eq!(bank.record_count(), 3);
        drop(bank);
        assert_eq!(drops.load(Ordering::Relaxed), 4);
    }
}

#[cfg(test)]
mod controlled_tests {
    use super::*;
    use crate::DomResource;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Value {
        text: Arc<Vec<u16>>,
        marker: u8,
    }

    type Pair = (Bank<Value>, BankCustody<Value>);

    fn value(marker: u8) -> Value {
        let mut text = Vec::with_capacity(23);
        text.push(u16::from(marker));
        Value {
            text: Arc::new(text),
            marker,
        }
    }

    fn supplied_usage(value: &Value) -> Usage {
        Usage::new(
            size_of::<Value>() + size_of::<Vec<u16>>() + value.text.capacity() * size_of::<u16>(),
            value.text.capacity(),
        )
    }

    // Independent description of actual retained fields; never reads Entry
    // totals, Page summaries, a token receipt or the admitted ledger total.
    fn describe(value: &Value) -> Result<Totals, DomError> {
        let field = &value.text;
        Ok(Totals {
            bytes: size_of::<Value>() + size_of::<Vec<u16>>() + field.capacity() * 2,
            units: field.capacity(),
            occurrences: 1,
        })
    }

    fn unlimited(_: Usage) -> Result<(), DomError> {
        Ok(())
    }

    fn new_registry() -> (Ledger, CustodyRegistry) {
        let ledger = Ledger::default();
        let registry = CustodyRegistry::new(&ledger, unlimited).unwrap();
        (ledger, registry)
    }

    const STAGE_DEADLINE: std::time::Duration = std::time::Duration::from_secs(5);

    struct ObservedWorker<T> {
        thread: std::thread::JoinHandle<()>,
        completed: std::sync::mpsc::Receiver<std::thread::Result<T>>,
    }

    fn observed_worker<T: Send + 'static>(
        work: impl FnOnce() -> T + Send + 'static,
    ) -> ObservedWorker<T> {
        let (completed, observed) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
            // Work and all of its owner-held locals have completed before the
            // observation. Only this nonblocking send and sender drop remain.
            let _ = completed.send(result);
        });
        ObservedWorker {
            thread,
            completed: observed,
        }
    }

    fn finish_worker<T>(worker: ObservedWorker<T>, last_stage: &str) -> T {
        let result = worker
            .completed
            .recv_timeout(STAGE_DEADLINE)
            .unwrap_or_else(|error| {
                panic!("worker completion deadline; last observed stage: {last_stage}; {error}")
            });
        // Join only after observing completion of the actual tested work.
        worker
            .thread
            .join()
            .expect("worker runtime completed after observation");
        match result {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    fn await_stage<T>(receiver: &std::sync::mpsc::Receiver<T>, last_stage: &str) -> T {
        receiver
            .recv_timeout(STAGE_DEADLINE)
            .unwrap_or_else(|error| {
                panic!("stage deadline; last observed stage: {last_stage}; {error}")
            })
    }

    fn put(pair: &mut Pair, index: usize, value: Value, registry: &CustodyRegistry) {
        let receipt = supplied_usage(&value);
        let record = ControlledRecord::new(value, receipt, registry, unlimited).unwrap();
        let totals = Totals {
            bytes: receipt.bytes(),
            units: receipt.units(),
            occurrences: 1,
        };
        let next = pair
            .0
            .set_controlled(&pair.1, index, record, false, totals, registry, unlimited)
            .unwrap();
        *pair = next;
        registry.reconcile_releases();
    }

    fn oracle(registry: &CustodyRegistry, roots: &[(&Bank<Value>, &BankCustody<Value>)]) -> Usage {
        oracle_with_records(registry, roots, &[])
    }

    fn oracle_with_records(
        registry: &CustodyRegistry,
        roots: &[(&Bank<Value>, &BankCustody<Value>)],
        held_records: &[&ControlledRecord<Value>],
    ) -> Usage {
        let mut semantic = PhysicalGraph::default();
        let mut control = PhysicalControlGraph::default();
        for (bank, custody) in roots {
            semantic
                .merge(&bank.inspect_graph(describe).unwrap())
                .unwrap();
            control.merge(custody.inspect_control_graph());
        }
        for held in held_records {
            semantic.records.insert(
                Arc::as_ptr(held.value()) as usize,
                (held.record.identity, describe(held.value()).unwrap()),
            );
            match &held.custody {
                RecordRetention::Registered(record) => control.visit_record(record),
                RecordRetention::Pending { debit, .. } => {
                    control.payload_debits.insert(debit.allocation_identity());
                }
            }
        }
        let records = semantic.record_totals();
        Usage::new(semantic.index_charge().bytes + records.bytes, records.units)
            .checked_add(control.usage())
            .unwrap()
            .checked_add(registry.inspect_storage())
            .unwrap()
    }

    fn assert_union(registry: &CustodyRegistry, roots: &[(&Bank<Value>, &BankCustody<Value>)]) {
        assert_eq!(registry.total(), oracle(registry, roots));
    }

    #[test]
    fn paired_roots_leaf_copies_and_nested_marks_charge_actual_object_union() {
        let (ledger, registry) = new_registry();
        let mut base = Pair::default();
        for index in 0..33 {
            put(&mut base, index, value(index as u8), &registry);
        }
        let snapshot = base.0.clone();
        let outer_mark = (base.0.clone(), base.1.clone());
        let before = ledger.total();
        assert_union(
            &registry,
            &[(&base.0, &base.1), (&outer_mark.0, &outer_mark.1)],
        );
        assert_eq!(ledger.total(), before);

        let receipt = supplied_usage(&value(201));
        let record = ControlledRecord::new(value(201), receipt, &registry, unlimited).unwrap();
        let candidate = base
            .0
            .set_controlled(
                &base.1,
                1,
                record,
                false,
                Totals {
                    bytes: receipt.bytes(),
                    units: receipt.units(),
                    occurrences: 1,
                },
                &registry,
                unlimited,
            )
            .unwrap();
        assert_union(
            &registry,
            &[
                (&base.0, &base.1),
                (&outer_mark.0, &outer_mark.1),
                (&candidate.0, &candidate.1),
            ],
        );
        let mut graph = base.0.inspect_graph(describe).unwrap();
        graph
            .merge(&candidate.0.inspect_graph(describe).unwrap())
            .unwrap();
        assert_eq!(graph.record_totals().occurrences, 34);
        assert!(
            graph.index_charge().pages
                < base.0.index_charge().pages + candidate.0.index_charge().pages
        );
        assert_eq!(snapshot[1].marker, 1);
        assert_eq!(candidate.0[1].marker, 201);
        let receipt = supplied_usage(&value(202));
        let record = ControlledRecord::new(value(202), receipt, &registry, unlimited).unwrap();
        let inner = candidate
            .0
            .set_controlled(
                &candidate.1,
                17,
                record,
                false,
                Totals {
                    bytes: receipt.bytes(),
                    units: receipt.units(),
                    occurrences: 1,
                },
                &registry,
                unlimited,
            )
            .unwrap();
        assert_union(
            &registry,
            &[
                (&base.0, &base.1),
                (&outer_mark.0, &outer_mark.1),
                (&candidate.0, &candidate.1),
                (&inner.0, &inner.1),
            ],
        );
        drop(inner);
        registry.reconcile_releases();
        assert_union(
            &registry,
            &[
                (&base.0, &base.1),
                (&outer_mark.0, &outer_mark.1),
                (&candidate.0, &candidate.1),
            ],
        );
        drop(outer_mark);
        registry.reconcile_releases();
        assert_union(
            &registry,
            &[(&base.0, &base.1), (&candidate.0, &candidate.1)],
        );
        drop(base);
        registry.reconcile_releases();
        assert_union(&registry, &[(&candidate.0, &candidate.1)]);
        // The external snapshot still holds old storage, but no controlled debit.
        assert_eq!(snapshot[1].marker, 1);
        drop(candidate);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        drop(snapshot);
        drop(registry);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn separate_records_sharing_field_backing_keep_conservative_occurrences() {
        let (_, registry) = new_registry();
        let shared = value(9).text;
        let mut pair = Pair::default();
        for (index, marker) in [(0, 1), (1, 2)] {
            put(
                &mut pair,
                index,
                Value {
                    text: shared.clone(),
                    marker,
                },
                &registry,
            );
        }
        assert!(Arc::ptr_eq(&pair.0[0].text, &pair.0[1].text));
        let graph = pair.0.inspect_graph(describe).unwrap();
        assert_eq!(graph.record_totals().occurrences, 2);
        assert_eq!(graph.record_totals().units, shared.capacity() * 2);
        assert_union(&registry, &[(&pair.0, &pair.1)]);
        let old = pair.0.controlled_record(&pair.1, 0).unwrap();
        let totals = pair.0.totals_at(0).unwrap();
        let (next, custody) = pair
            .0
            .set_controlled(&pair.1, 17, old, false, totals, &registry, unlimited)
            .unwrap();
        let mut union = pair.0.inspect_graph(describe).unwrap();
        union.merge(&next.inspect_graph(describe).unwrap()).unwrap();
        assert_eq!(union.record_totals().occurrences, 2);
        assert_eq!(
            next.record(0).unwrap().identity,
            next.record(17).unwrap().identity
        );
        assert_union(&registry, &[(&pair.0, &pair.1), (&next, &custody)]);
    }

    #[test]
    fn historical_adoption_prunes_controlled_subtrees_and_reuses_one_capture() {
        let (_, registry) = new_registry();
        let mut current = Pair::default();
        for index in 0..257 {
            put(&mut current, index, value(index as u8), &registry);
        }
        let mut historical = current.0.clone();
        historical
            .set(17, value(222), false, Totals::default())
            .unwrap();
        let described = AtomicUsize::new(0);
        let capture = historical
            .adopt_controlled(
                &registry,
                |value| {
                    described.fetch_add(1, Ordering::Relaxed);
                    let receipt = describe(value)?;
                    Ok(Usage::new(receipt.bytes, receipt.units))
                },
                unlimited,
            )
            .unwrap();
        assert_eq!(described.load(Ordering::Relaxed), 1);
        assert_union(
            &registry,
            &[(&current.0, &current.1), (&historical, &capture)],
        );
        let retained = registry.total();
        let same_capture = historical
            .adopt_controlled(
                &registry,
                |_| -> Result<Usage, DomError> { panic!("shared input is not re-enumerated") },
                unlimited,
            )
            .unwrap();
        assert_eq!(registry.total(), retained);
        drop(capture);
        registry.reconcile_releases();
        assert_union(
            &registry,
            &[(&current.0, &current.1), (&historical, &same_capture)],
        );
        drop(same_capture);
        registry.reconcile_releases();
        assert_union(&registry, &[(&current.0, &current.1)]);
        assert_eq!(historical[17].marker, 222);
    }

    #[test]
    fn independently_enumerated_capture_peak_admits_exact_and_rolls_back_one_short() {
        let (ledger, registry) = new_registry();
        let mut base = Pair::default();
        for index in [0, 1, 16, 256] {
            put(&mut base, index, value(index as u8), &registry);
        }
        let mut historical = base.0.clone();
        historical
            .set(1, value(45), false, Totals::default())
            .unwrap();
        let capture = historical
            .adopt_controlled(
                &registry,
                |value| {
                    let actual = describe(value)?;
                    Ok(Usage::new(actual.bytes, actual.units))
                },
                unlimited,
            )
            .unwrap();
        let exact = oracle(&registry, &[(&base.0, &base.1), (&historical, &capture)]);
        assert_eq!(ledger.total(), exact);
        drop(capture);
        registry.reconcile_releases();
        let baseline = oracle(&registry, &[(&base.0, &base.1)]);
        let work = AtomicUsize::new(0);
        let rejected = historical.adopt_controlled(
            &registry,
            |value| {
                let actual = describe(value)?;
                Ok(Usage::new(actual.bytes, actual.units))
            },
            |next| {
                work.fetch_add(1, Ordering::Relaxed);
                if next.bytes() > exact.bytes() - 1 {
                    Err(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: exact.bytes() - 1,
                    })
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(
            rejected,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        assert!(work.load(Ordering::Relaxed) > 0);
        assert_eq!(ledger.total(), baseline);
        let rejected = historical.adopt_controlled(
            &registry,
            |value| {
                let actual = describe(value)?;
                Ok(Usage::new(actual.bytes, actual.units))
            },
            |next| {
                work.fetch_add(1, Ordering::Relaxed);
                if next.units() > exact.units() - 1 {
                    Err(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedUnits,
                        limit: exact.units() - 1,
                    })
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(
            rejected,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedUnits,
                ..
            })
        ));
        assert_eq!(ledger.total(), baseline);
        let capture = historical
            .adopt_controlled(
                &registry,
                |value| {
                    let actual = describe(value)?;
                    Ok(Usage::new(actual.bytes, actual.units))
                },
                |next| {
                    assert!(next.bytes() <= exact.bytes());
                    assert!(next.units() <= exact.units());
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(ledger.total(), exact);
        assert_union(&registry, &[(&base.0, &base.1), (&historical, &capture)]);
    }

    #[test]
    fn refused_path_and_capture_leave_original_pair_and_true_retained_amount() {
        let (ledger, registry) = new_registry();
        let mut pair = Pair::default();
        for index in [0, 16, 4096] {
            put(&mut pair, index, value(index as u8), &registry);
        }
        let old_root = pair.0.clone();
        let receipt = supplied_usage(&value(12));
        let record = ControlledRecord::new(value(12), receipt, &registry, unlimited).unwrap();
        let calls = AtomicUsize::new(0);
        let result = pair.0.set_controlled(
            &pair.1,
            17,
            record,
            false,
            Totals {
                bytes: receipt.bytes(),
                units: receipt.units(),
                occurrences: 1,
            },
            &registry,
            |_| {
                if calls.fetch_add(1, Ordering::Relaxed) == 2 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(result, Err(DomError::Cancelled)));
        assert!(pair.0.shares_root(&old_root));
        assert_union(&registry, &[(&pair.0, &pair.1)]);
        let baseline = ledger.total();
        let mut historical = pair.0.clone();
        historical
            .set(18, value(19), false, Totals::default())
            .unwrap();
        let mut calls = 0;
        let result = historical.adopt_controlled(
            &registry,
            |value| {
                let actual = describe(value)?;
                Ok(Usage::new(actual.bytes, actual.units))
            },
            |_| {
                calls += 1;
                if calls == 4 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(result, Err(DomError::Cancelled)));
        assert_eq!(ledger.total(), baseline);
        assert!(pair.0.shares_root(&old_root));
    }

    #[test]
    fn stable_ids_survive_external_expiry_reentry_and_cleanup_stays_bounded() {
        let (ledger, registry) = new_registry();
        let mut raw = Bank::default();
        raw.set(0, value(1), false, Totals::default()).unwrap();
        raw.set(256, value(2), false, Totals::default()).unwrap();
        let ids: Vec<_> = raw
            .inspect_graph(describe)
            .unwrap()
            .records()
            .map(|(_, id, _)| id)
            .collect();
        let baseline = ledger.total();
        for _ in 0..40 {
            let capture = raw
                .adopt_controlled(
                    &registry,
                    |value| {
                        let actual = describe(value)?;
                        Ok(Usage::new(actual.bytes, actual.units))
                    },
                    unlimited,
                )
                .unwrap();
            assert_union(&registry, &[(&raw, &capture)]);
            assert_eq!(
                raw.inspect_graph(describe)
                    .unwrap()
                    .records()
                    .map(|(_, id, _)| id)
                    .collect::<Vec<_>>(),
                ids
            );
            drop(capture);
            registry.reconcile_releases();
            assert_eq!(ledger.total(), baseline);
            assert!(registry.lock().pages.is_none());
            assert!(registry.lock().records.is_none());
        }
        let first = Record::new(1u64).unwrap();
        let alias = first.clone();
        let equal = Record::new(1u64).unwrap();
        let other_type = Record::new(vec![1u16]).unwrap();
        assert_eq!(first.identity, alias.identity);
        assert_ne!(first.identity, equal.identity);
        assert_ne!(first.identity, other_type.identity);
        let (_, other_registry) = new_registry();
        let other = ControlledRecord::new(
            1u64,
            Usage::new(size_of::<u64>(), 0),
            &other_registry,
            unlimited,
        )
        .unwrap();
        assert_ne!(first.identity, other.record.identity);
    }

    #[test]
    fn nonbank_atoms_share_identity_and_release_weak_backing_after_owner_loss() {
        let (ledger, registry) = new_registry();
        let identity = MaterializationId::fresh().unwrap();
        let first = registry
            .adopt_object(identity, Usage::new(8, 0), unlimited)
            .unwrap();
        let total = ledger.total();
        let second = registry
            .adopt_object(identity, Usage::new(8, 0), unlimited)
            .unwrap();
        assert_eq!(first.identity(), second.identity());
        assert_eq!(ledger.total(), total);
        drop(first);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), total);
        let mut graph = PhysicalControlGraph::default();
        graph.visit_record(&second.0);
        let expected_after_owner_loss = Usage::new(8, 0).checked_add(graph.usage()).unwrap();
        drop(registry);
        assert_eq!(ledger.total(), expected_after_owner_loss);
        drop(second);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn retained_object_registration_transfers_the_original_extent_and_preserves_caller_on_refusal()
    {
        struct Body {
            ledger: Ledger,
            seen: Arc<Mutex<Vec<Usage>>>,
        }
        impl Drop for Body {
            fn drop(&mut self) {
                self.seen.lock().unwrap().push(self.ledger.total());
            }
        }
        for refuse_at in [Some(1), Some(2), None] {
            let (ledger, registry) = new_registry();
            let identity = MaterializationId::fresh().unwrap();
            let seen = Arc::new(Mutex::new(Vec::new()));
            let payload = Usage::new(size_of::<Body>(), 0);
            let debit = ledger.retain_shared(payload, unlimited).unwrap();
            let prepared = Retained {
                value: Arc::new(Body {
                    ledger: ledger.clone(),
                    seen: seen.clone(),
                }),
                debit,
            };
            let before = ledger.total();
            let mut calls = 0;
            let result = registry.adopt_object_retained(identity, &prepared.debit, |_| {
                calls += 1;
                if Some(calls) == refuse_at {
                    Err(DomError::AllocationFailed)
                } else {
                    Ok(())
                }
            });
            if let Some(refuse_at) = refuse_at {
                assert!(matches!(result, Err(DomError::AllocationFailed)));
                assert_eq!(calls, refuse_at);
                assert_eq!(ledger.total(), before);
                assert!(
                    seen.lock().unwrap().is_empty(),
                    "caller still owns the actual body"
                );
                drop(prepared);
                assert_eq!(*seen.lock().unwrap(), [before]);
            } else {
                let owner = result.unwrap();
                assert_eq!(
                    owner.0._payload.allocation_identity(),
                    prepared.debit.allocation_identity(),
                    "registration aliases the original SharedDebit target"
                );
                let mut controls = PhysicalControlGraph::default();
                controls.visit_record(&owner.0);
                let expected = payload
                    .checked_add(controls.usage())
                    .unwrap()
                    .checked_add(registry.inspect_storage())
                    .unwrap();
                assert_eq!(ledger.total(), expected);
                let mut alias_calls = 0;
                let alias = registry
                    .adopt_object_retained(identity, &prepared.debit, |total| {
                        alias_calls += 1;
                        assert_eq!(total, expected);
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(alias_calls, 1);
                assert!(Arc::ptr_eq(&owner.0, &alias.0));
                assert_eq!(ledger.total(), expected);
                drop(prepared);
                assert_eq!(*seen.lock().unwrap(), [expected]);
                drop(owner);
                assert_eq!(ledger.total(), expected);
                drop(alias);
            }
            registry.reconcile_releases();
            assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        }

        let (ledger, registry) = new_registry();
        let foreign = Ledger::default();
        let retained = foreign.retain_shared(Usage::new(8, 0), unlimited).unwrap();
        let before = registry.inspect_storage();
        let nonce = registry.lock().next_nonce;
        let calls = AtomicUsize::new(0);
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            registry.adopt_object_retained(MaterializationId::fresh().unwrap(), &retained, |_| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
        }));
        assert!(refused.is_err());
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        assert_eq!(registry.lock().next_nonce, nonce);
        assert_eq!(registry.inspect_storage(), before);
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert_eq!(foreign.total(), Usage::new(8 + size_of::<Debit>(), 0));
        drop(retained);
        assert_eq!(foreign.total(), Usage::ZERO);
    }

    #[test]
    fn physical_oracle_ignores_poisoned_cached_totals_and_preserves_leaf_alias_ids() {
        let record = Record::new(value(3)).unwrap();
        let mut bank = Bank::default();
        for index in [0, 16] {
            bank.set_admitted(
                index,
                record.clone(),
                false,
                Totals {
                    bytes: 987654,
                    units: 654321,
                    occurrences: 99,
                },
                |_| Ok(()),
            )
            .unwrap();
        }
        let graph = bank.inspect_graph(describe).unwrap();
        assert_eq!(graph.record_totals().occurrences, 1);
        assert_eq!(graph.record_totals().units, bank[0].text.capacity());
        assert_ne!(graph.record_totals(), bank.totals());
        assert_eq!(graph.index_charge(), bank.index_charge());
        assert_eq!(
            bank.record(0).unwrap().identity,
            bank.record(16).unwrap().identity
        );
        let page_ids: std::collections::BTreeSet<_> = graph.pages().map(|(_, id, _)| id).collect();
        assert!(!page_ids.contains(&record.identity.key()));
        assert_eq!(bank.values().count(), 2);
    }

    #[test]
    fn controlled_removal_transfers_record_custody_to_output_until_its_drop() {
        let (ledger, registry) = new_registry();
        let external = Record::new(value(7)).unwrap();
        let usage = supplied_usage(external.value());
        let admitted = registry.ledger().retain_shared(usage, unlimited).unwrap();
        let retained = ControlledRecord::from_parts(external.clone(), admitted);
        let empty = Pair::default();
        let original = empty
            .0
            .set_controlled(
                &empty.1,
                256,
                retained,
                false,
                Totals {
                    bytes: usage.bytes(),
                    units: usage.units(),
                    occurrences: 1,
                },
                &registry,
                unlimited,
            )
            .unwrap();
        let (next, next_custody, removed) = original
            .0
            .remove_controlled(&original.1, 256, &registry, unlimited)
            .unwrap();
        let removed = removed.unwrap();
        assert_eq!(next.len(), 257);
        assert_eq!(next.record_count(), 0);
        assert_eq!(removed.record.identity, external.identity);
        assert_union(
            &registry,
            &[(&original.0, &original.1), (&next, &next_custody)],
        );
        drop(original);
        registry.reconcile_releases();
        assert_eq!(
            registry.total(),
            oracle_with_records(&registry, &[(&next, &next_custody)], &[&removed])
        );
        assert_eq!(removed.value().marker, 7);
        drop(removed);
        registry.reconcile_releases();
        assert_union(&registry, &[(&next, &next_custody)]);
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert_eq!(external.value().marker, 7);
    }

    #[test]
    fn stale_queued_nonce_cannot_remove_later_custody_for_the_same_materialization() {
        let (ledger, registry) = new_registry();
        let identity = MaterializationId::fresh().unwrap();
        let old = registry
            .adopt_object(identity, Usage::new(8, 0), unlimited)
            .unwrap();
        let old_nonce = {
            let state = registry.lock();
            registry_entry(state.root(RegistryKind::Record), identity.key(), ID_SHIFT)
                .unwrap()
                .nonce
        };
        drop(old);
        registry.reconcile_releases();
        let current = registry
            .adopt_object(identity, Usage::new(8, 0), unlimited)
            .unwrap();
        let baseline = ledger.total();
        let release = ledger
            .retain(Usage::new(size_of::<ReleaseNode>(), 0), unlimited)
            .unwrap();
        {
            let mut state = registry.lock();
            let next = state.released.take();
            state.released = Some(Box::new(ReleaseNode {
                kind: RegistryKind::Record,
                identity,
                nonce: old_nonce,
                next,
                _debit: release,
            }));
        }
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
        let alias = registry
            .adopt_object(identity, Usage::new(8, 0), unlimited)
            .unwrap();
        assert_eq!(ledger.total(), baseline);
        assert_eq!(alias.identity(), current.identity());
        drop(current);
        drop(alias);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
    }

    #[test]
    fn provisional_slots_destroy_actual_values_before_their_structural_debit() {
        struct Observed {
            ledger: Ledger,
            seen: Arc<Mutex<Option<Usage>>>,
        }
        impl Drop for Observed {
            fn drop(&mut self) {
                *self.seen.lock().unwrap() = Some(self.ledger.total());
            }
        }
        let (ledger, registry) = new_registry();
        let seen = Arc::new(Mutex::new(None));
        let mut slots = controlled_slots::<Value, Observed, ()>(&registry, &mut unlimited).unwrap();
        slots.values[0] = Some(Observed {
            ledger: ledger.clone(),
            seen: seen.clone(),
        });
        let before = ledger.total();
        drop(slots);
        assert_eq!(*seen.lock().unwrap(), Some(before));
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
    }

    #[test]
    fn refused_page_registration_keeps_semantic_page_and_backing_admitted_through_their_drop() {
        let (ledger, registry) = new_registry();
        let record = value(9);
        let usage = supplied_usage(&record);
        let record = ControlledRecord::new(record, usage, &registry, unlimited).unwrap();
        let before = ledger.total();
        let shallow = size_of::<Page<Value>>()
            + WIDTH * size_of::<Option<Entry<Value>>>()
            + size_of::<PageCustody>()
            + size_of::<Mutex<RegistryState>>();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let output = seen.clone();
        let observed_ledger = ledger.clone();
        let watch =
            drop_observation::pages(move |_| output.lock().unwrap().push(observed_ledger.total()));
        let mut attempts = 0;
        let empty = Pair::default();
        let result = empty.0.set_controlled(
            &empty.1,
            0,
            record,
            false,
            Totals {
                bytes: usage.bytes(),
                units: usage.units(),
                occurrences: 1,
            },
            &registry,
            |_| {
                attempts += 1;
                if attempts == 2 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(result, Err(DomError::Cancelled)));
        drop(watch);
        assert_eq!(
            *seen.lock().unwrap(),
            [before.checked_add(Usage::new(shallow, 0)).unwrap()]
        );
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
    }

    #[test]
    fn refused_later_root_growth_destroys_each_registered_page_before_its_custody() {
        let (_, registry) = new_registry();
        let mut base = Pair::default();
        put(&mut base, 0, value(1), &registry);
        let incoming = value(2);
        let usage = supplied_usage(&incoming);
        let incoming = ControlledRecord::new(incoming, usage, &registry, unlimited).unwrap();
        let seen = Arc::new(Mutex::new(0));
        let observed = seen.clone();
        let observer_registry = registry.clone();
        let watch = drop_observation::pages(move |identity| {
            let state = observer_registry.lock();
            if let Some(entry) =
                registry_entry(state.root(RegistryKind::Page), identity.key(), ID_SHIFT)
            {
                let WeakCustody::Page(custody) = &entry.custody else {
                    unreachable!()
                };
                assert!(
                    custody.upgrade().is_some(),
                    "semantic Page destructor precedes last controlled custodian"
                );
                *observed.lock().unwrap() += 1;
            }
        });
        let mut attempts = 0;
        let result = base.0.set_controlled(
            &base.1,
            1 << 24,
            incoming,
            false,
            Totals {
                bytes: usage.bytes(),
                units: usage.units(),
                occurrences: 1,
            },
            &registry,
            |_| {
                attempts += 1;
                if attempts == 8 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(result, Err(DomError::Cancelled)));
        drop(watch);
        assert!(
            *seen.lock().unwrap() > 0,
            "refusal follows an actual intermediate page registration"
        );
        assert_union(&registry, &[(&base.0, &base.1)]);
    }

    #[test]
    fn registry_admission_may_drop_a_same_registry_controlled_record() {
        finish_worker(
            observed_worker(|| {
                let (ledger, registry) = new_registry();
                let original = value(1);
                let original_usage = supplied_usage(&original);
                let mut original = Some(
                    ControlledRecord::new(original, original_usage, &registry, unlimited).unwrap(),
                );
                let next = value(2);
                let usage = supplied_usage(&next);
                let mut attempts = 0;
                let acquired = ControlledRecord::new(next, usage, &registry, |_| {
                    attempts += 1;
                    if attempts == 2 {
                        drop(original.take());
                        registry.reconcile_releases();
                    }
                    Ok(())
                })
                .unwrap();
                assert!(original.is_none());
                drop(acquired);
                registry.reconcile_releases();
                assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
            }),
            "record construction entered; administration callback drops prior record",
        );
    }

    #[test]
    fn registration_retries_when_admission_prunes_its_previously_shared_index_prefix() {
        finish_worker(
            observed_worker(|| {
                let (ledger, registry) = new_registry();
                let original = value(1);
                let original_usage = supplied_usage(&original);
                let mut original = Some(
                    ControlledRecord::new(original, original_usage, &registry, unlimited).unwrap(),
                );
                let next = value(2);
                let usage = supplied_usage(&next);
                let mut attempts = 0;
                let acquired = ControlledRecord::new(next, usage, &registry, |_| {
                    attempts += 1;
                    // Payload and administration have been admitted. A registration
                    // plan now exists, with part of its ID path shared by the old atom.
                    if attempts == 3 {
                        drop(original.take());
                        registry.reconcile_releases();
                    }
                    Ok(())
                })
                .unwrap();
                assert!(original.is_none());
                let expected = oracle_with_records(&registry, &[], &[&acquired]);
                assert_eq!(ledger.total(), expected);
                drop(acquired);
                registry.reconcile_releases();
                assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
            }),
            "record construction entered; planned-path admission prunes prior index",
        );
    }

    #[test]
    fn reentrant_historical_capture_revalidates_one_active_page_and_record_custodian() {
        finish_worker(
            observed_worker(|| {
                let (_, registry) = new_registry();
                let mut historical = Bank::default();
                let raw = value(4);
                let totals = describe(&raw).unwrap();
                historical.set(0, raw, false, totals).unwrap();
                let mut nested = None;
                let mut attempted = false;
                let outer = historical
                    .adopt_controlled(
                        &registry,
                        |record| Ok(supplied_usage(record)),
                        |_| {
                            if !attempted {
                                attempted = true;
                                nested = Some(historical.adopt_controlled(
                                    &registry,
                                    |record| Ok(supplied_usage(record)),
                                    unlimited,
                                )?);
                            }
                            Ok(())
                        },
                    )
                    .unwrap();
                let nested = nested.unwrap();
                assert!(Arc::ptr_eq(
                    outer.root.as_ref().unwrap(),
                    nested.root.as_ref().unwrap()
                ));
                assert_union(&registry, &[(&historical, &outer), (&historical, &nested)]);
            }),
            "historical adoption entered; nested adoption uses same registry",
        );
    }

    #[test]
    fn concurrent_prepared_registrations_attach_only_one_custodian_for_the_same_identity() {
        use std::sync::mpsc;
        let (ledger, registry) = new_registry();
        let identity = MaterializationId::fresh().unwrap();
        let (planned, observed_plan) = mpsc::channel();
        let spawn = |resume: mpsc::Receiver<()>| {
            let registry = registry.clone();
            let planned = planned.clone();
            observed_worker(move || {
                drop_observation::registration_once(move || {
                    // Both registrations have reserved distinct nonces after
                    // observing absence, before either prepares or attaches.
                    planned.send(()).unwrap();
                    await_stage(
                        &resume,
                        "registration plan sent; attachment permission pending",
                    );
                });
                registry
                    .adopt_object(identity, Usage::new(8, 0), unlimited)
                    .unwrap()
            })
        };
        let (resume_first, first_resume) = mpsc::channel();
        let (resume_second, second_resume) = mpsc::channel();
        let first = spawn(first_resume);
        let second = spawn(second_resume);
        await_stage(
            &observed_plan,
            "both workers started; no post-plan observation yet",
        );
        await_stage(
            &observed_plan,
            "one worker reached post-plan; awaiting second plan",
        );
        resume_first.send(()).unwrap();
        resume_second.send(()).unwrap();
        let first = finish_worker(first, "both registration plans released for attachment");
        let second = finish_worker(
            second,
            "first registration completed; second attachment released",
        );
        assert!(Arc::ptr_eq(&first.0, &second.0));
        drop(first);
        drop(second);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
    }

    #[test]
    fn expired_weak_is_replaced_before_its_delayed_destructor_queues_the_old_nonce() {
        use std::sync::mpsc;
        let (ledger, registry) = new_registry();
        let identity = MaterializationId::fresh().unwrap();
        let old = registry
            .adopt_object(identity, Usage::new(8, 0), unlimited)
            .unwrap();
        let (at_destructor, observed_destructor) = mpsc::channel();
        let (resume_destructor, may_resume) = mpsc::channel();
        let dropping = observed_worker(move || {
            drop_observation::release_once(move || {
                at_destructor.send(()).unwrap();
                await_stage(
                    &may_resume,
                    "old Arc strong-zero/prequeue stage sent; waiting for re-adoption",
                );
            });
            drop(old);
        });
        await_stage(
            &observed_destructor,
            "old custodian moved to drop worker; awaiting strong-zero/prequeue stage",
        );
        {
            let state = registry.lock();
            assert!(
                state.released.is_none(),
                "old destructor has not queued its preallocated ticket"
            );
            let entry =
                registry_entry(state.root(RegistryKind::Record), identity.key(), ID_SHIFT).unwrap();
            let WeakCustody::Record(old) = &entry.custody else {
                unreachable!()
            };
            assert!(old.upgrade().is_none(), "actual old Arc is at strong zero");
        }
        let current = registry
            .adopt_object(identity, Usage::new(8, 0), unlimited)
            .unwrap();
        resume_destructor.send(()).unwrap();
        finish_worker(
            dropping,
            "same-ID replacement adopted; old destructor released to queue cleanup",
        );
        registry.reconcile_releases();
        let before_alias = ledger.total();
        let alias = registry
            .adopt_object(identity, Usage::new(8, 0), unlimited)
            .unwrap();
        assert!(Arc::ptr_eq(&current.0, &alias.0));
        assert_eq!(ledger.total(), before_alias);
        drop(current);
        drop(alias);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
    }

    struct Nested {
        children: Bank<u64>,
        label: u8,
    }

    struct ChildFrontier {
        custody: BankCustody<u64>,
    }

    fn child_totals(_: &u64) -> Result<Totals, DomError> {
        Ok(Totals {
            bytes: size_of::<u64>(),
            units: 0,
            occurrences: 1,
        })
    }

    fn nested_description(
        value: &Nested,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        visits: &AtomicUsize,
    ) -> Result<RecordDescription, DomError> {
        let custody = value.children.adopt_controlled(
            registry,
            |_| {
                visits.fetch_add(1, Ordering::Relaxed);
                Ok(Usage::new(size_of::<u64>(), 0))
            },
            &mut *admit,
        )?;
        let dependency = Dependency::new(ChildFrontier { custody }, registry.ledger(), admit)?;
        Ok(RecordDescription::with_dependency(
            Usage::new(size_of::<Nested>(), 0),
            dependency,
        ))
    }

    fn nested_totals(_: &Nested) -> Result<Totals, DomError> {
        Ok(Totals {
            bytes: size_of::<Nested>(),
            units: 0,
            occurrences: 1,
        })
    }

    fn nested_oracle(
        registry: &CustodyRegistry,
        bank: &Bank<Nested>,
        custody: &BankCustody<Nested>,
    ) -> Usage {
        let mut semantic = bank.inspect_graph(nested_totals).unwrap();
        let mut control = custody.inspect_control_graph();
        let mut bodies = std::collections::BTreeSet::new();
        for (index, value) in bank.iter() {
            let retained = bank
                .controlled_record(custody, index)
                .unwrap()
                .dependency::<ChildFrontier>()
                .unwrap();
            semantic
                .merge(&value.children.inspect_graph(child_totals).unwrap())
                .unwrap();
            control.merge(retained.custody.inspect_control_graph());
            bodies.insert(Arc::as_ptr(&retained.value) as usize);
        }
        Usage::new(semantic.index_charge().bytes, 0)
            .checked_add(Usage::new(semantic.record_totals().bytes, 0))
            .unwrap()
            .checked_add(control.usage())
            .unwrap()
            .checked_add(registry.inspect_storage())
            .unwrap()
            .checked_add(Usage::new(
                bodies.len() * (size_of::<ChildFrontier>() + size_of::<Debit>()),
                0,
            ))
            .unwrap()
    }

    #[test]
    fn unmaterialized_nested_record_keeps_child_custody_through_identity_payload_and_metadata_refusal()
     {
        #[derive(Clone, Copy, Debug)]
        enum Refusal {
            Identity,
            Payload,
            Metadata,
        }
        for refusal in [Refusal::Identity, Refusal::Payload, Refusal::Metadata] {
            let (ledger, registry) = new_registry();
            let mut children = Bank::default();
            children
                .set(0, 41u64, false, child_totals(&41).unwrap())
                .unwrap();
            let child_identity = children.root.as_ref().unwrap().identity;
            let child_custody = children
                .adopt_controlled(
                    &registry,
                    |_| Ok(Usage::new(size_of::<u64>(), 0)),
                    unlimited,
                )
                .unwrap();
            let dependency = Dependency::new(
                ChildFrontier {
                    custody: child_custody,
                },
                &ledger,
                unlimited,
            )
            .unwrap();
            let before = ledger.total();
            let observations = Arc::new(Mutex::new(Vec::new()));
            let output = observations.clone();
            let observed_ledger = ledger.clone();
            let observed_registry = registry.clone();
            let watch = drop_observation::pages(move |identity| {
                if identity == child_identity {
                    output.lock().unwrap().push((
                        observed_ledger.total(),
                        observed_registry.find_page(identity).is_some(),
                    ));
                }
            });
            let mut calls = 0;
            let result = ControlledRecord::new_with_identity(
                Nested { children, label: 1 },
                RecordDescription::with_dependency(Usage::new(size_of::<Nested>(), 0), dependency),
                &registry,
                &mut |_| {
                    calls += 1;
                    if matches!(
                        (refusal, calls),
                        (Refusal::Payload, 1) | (Refusal::Metadata, 2)
                    ) {
                        Err(DomError::AllocationFailed)
                    } else {
                        Ok(())
                    }
                },
                || {
                    // The real allocator's checked exhaustion is tested in
                    // custody; inject only that first fallible result here.
                    if matches!(refusal, Refusal::Identity) {
                        Err(DomError::RepresentationExhausted)
                    } else {
                        MaterializationId::fresh()
                    }
                },
            );
            assert!(matches!(
                result,
                Err(DomError::AllocationFailed | DomError::RepresentationExhausted)
            ));
            let retained_payload = if matches!(refusal, Refusal::Metadata) {
                Usage::new(size_of::<Nested>() + size_of::<Debit>(), 0)
            } else {
                Usage::ZERO
            };
            assert_eq!(
                *observations.lock().unwrap(),
                [(before.checked_add(retained_payload).unwrap(), true)],
                "actual child Page must be destroyed while its dependency is live: {refusal:?}"
            );
            drop(watch);
            assert_eq!(
                calls,
                match refusal {
                    Refusal::Identity => 0,
                    Refusal::Payload => 1,
                    Refusal::Metadata => 2,
                }
            );
            assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        }
    }

    #[test]
    fn dependency_accessor_outlives_outer_record_with_body_and_child_custody_intact() {
        let (ledger, registry) = new_registry();
        let mut children = Bank::default();
        children
            .set(0, 41u64, false, child_totals(&41).unwrap())
            .unwrap();
        let child_custody = children
            .adopt_controlled(
                &registry,
                |_| Ok(Usage::new(size_of::<u64>(), 0)),
                unlimited,
            )
            .unwrap();
        let dependency = Dependency::new(
            ChildFrontier {
                custody: child_custody,
            },
            &ledger,
            unlimited,
        )
        .unwrap();
        let nested = Nested { children, label: 1 };
        let reserved = Record::reserve().unwrap();
        let debit = ledger
            .retain_shared(Usage::new(size_of::<Nested>(), 0), unlimited)
            .unwrap();
        let record = ControlledRecord::from_parts(reserved.materialize(nested), debit)
            .with_dependency(dependency);
        let empty = (Bank::default(), BankCustody::default());
        let parent = empty
            .0
            .set_controlled(
                &empty.1,
                0,
                record,
                false,
                Totals {
                    bytes: size_of::<Nested>(),
                    units: 0,
                    occurrences: 1,
                },
                &registry,
                unlimited,
            )
            .unwrap();
        assert_eq!(
            ledger.total(),
            nested_oracle(&registry, &parent.0, &parent.1)
        );
        let external = parent.0.clone();
        let retained = parent
            .0
            .controlled_record(&parent.1, 0)
            .unwrap()
            .dependency::<ChildFrontier>()
            .unwrap();
        let retained_alias = retained.clone();
        drop(parent);
        registry.reconcile_releases();
        let child_graph = external[0].children.inspect_graph(child_totals).unwrap();
        let expected = Usage::new(
            child_graph.index_charge().bytes + child_graph.record_totals().bytes,
            0,
        )
        .checked_add(retained.custody.inspect_control_graph().usage())
        .unwrap()
        .checked_add(registry.inspect_storage())
        .unwrap()
        .checked_add(Usage::new(
            size_of::<ChildFrontier>() + size_of::<Debit>(),
            0,
        ))
        .unwrap();
        assert_eq!(ledger.total(), expected);
        assert_eq!(external[0].children[0], 41);
        drop(retained);
        assert_eq!(ledger.total(), expected);
        drop(retained_alias);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert_eq!(
            external[0].children[0], 41,
            "external immutable values carry no custody leases"
        );
    }

    #[test]
    fn historical_nested_frontier_adopts_only_new_children_and_survives_old_parent_drop() {
        let (ledger, registry) = new_registry();
        let visits = AtomicUsize::new(0);
        let mut children = Bank::default();
        children
            .set(0, 7u64, false, child_totals(&7).unwrap())
            .unwrap();
        let original = Nested { children, label: 1 };
        let description =
            nested_description(&original, &registry, &mut unlimited, &visits).unwrap();
        let record = ControlledRecord::new(original, description, &registry, unlimited).unwrap();
        let empty = (Bank::default(), BankCustody::default());
        let old = empty
            .0
            .set_controlled(
                &empty.1,
                0,
                record,
                false,
                Totals {
                    bytes: size_of::<Nested>(),
                    units: 0,
                    occurrences: 1,
                },
                &registry,
                unlimited,
            )
            .unwrap();
        let mut shared_children = old.0[0].children.clone();
        shared_children
            .set(1, 9u64, false, child_totals(&9).unwrap())
            .unwrap();
        let mut historical = old.0.clone();
        let new = Nested {
            children: shared_children,
            label: 2,
        };
        historical
            .set(
                17,
                new,
                false,
                Totals {
                    bytes: size_of::<Nested>(),
                    units: 0,
                    occurrences: 1,
                },
            )
            .unwrap();
        visits.store(0, Ordering::Relaxed);
        let mut described = Vec::new();
        let capture = historical
            .adopt_controlled(
                &registry,
                |value| {
                    described.push(value.label);
                    nested_description(value, &registry, &mut unlimited, &visits)
                },
                unlimited,
            )
            .unwrap();
        assert_eq!(described, [2]);
        assert_eq!(visits.load(Ordering::Relaxed), 1);
        drop(old);
        registry.reconcile_releases();
        assert_eq!(
            ledger.total(),
            nested_oracle(&registry, &historical, &capture)
        );
        let before = ledger.total();
        let alias = historical
            .adopt_controlled(
                &registry,
                |_| -> Result<RecordDescription, DomError> {
                    panic!("active nested root prunes every descriptor")
                },
                unlimited,
            )
            .unwrap();
        assert_eq!(ledger.total(), before);
        assert!(Arc::ptr_eq(
            capture.root.as_ref().unwrap(),
            alias.root.as_ref().unwrap()
        ));
        assert_eq!(historical[0].children[0], 7);
        assert_eq!(historical[17].children[1], 9);
        drop(capture);
        drop(alias);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
    }

    #[test]
    fn nested_frontier_exact_peak_and_dependency_body_one_short_refuse_without_leaks() {
        let (ledger, registry) = new_registry();
        let baseline = ledger.total();
        let body = size_of::<ChildFrontier>() + size_of::<Debit>();
        let refused = Dependency::new(
            ChildFrontier {
                custody: BankCustody::default(),
            },
            &ledger,
            |next| {
                if next.bytes() > baseline.bytes() + body - 1 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert_eq!(ledger.total(), baseline);
        let mut child = Bank::default();
        child
            .set(3, 17u64, false, child_totals(&17).unwrap())
            .unwrap();
        let mut historical = Bank::default();
        historical
            .set(
                0,
                Nested {
                    children: child,
                    label: 1,
                },
                false,
                Totals {
                    bytes: size_of::<Nested>(),
                    units: 0,
                    occurrences: 1,
                },
            )
            .unwrap();
        let visits = AtomicUsize::new(0);
        let capture = historical
            .adopt_controlled(
                &registry,
                |value| nested_description(value, &registry, &mut unlimited, &visits),
                unlimited,
            )
            .unwrap();
        let exact = nested_oracle(&registry, &historical, &capture);
        assert_eq!(ledger.total(), exact);
        drop(capture);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
        let attempts = AtomicUsize::new(0);
        let policy = |next: Usage| {
            attempts.fetch_add(1, Ordering::Relaxed);
            if next.bytes() > exact.bytes() - 1 {
                Err(DomError::Cancelled)
            } else {
                Ok(())
            }
        };
        let refused = historical.adopt_controlled(
            &registry,
            |value| nested_description(value, &registry, &mut |next| policy(next), &visits),
            policy,
        );
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert!(attempts.load(Ordering::Relaxed) > 0);
        assert_eq!(ledger.total(), baseline);
        let capture = historical
            .adopt_controlled(
                &registry,
                |value| {
                    nested_description(
                        value,
                        &registry,
                        &mut |next| {
                            assert!(next.bytes() <= exact.bytes());
                            Ok(())
                        },
                        &visits,
                    )
                },
                |next| {
                    assert!(next.bytes() <= exact.bytes());
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(ledger.total(), exact);
        drop(capture);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
    }

    struct CachedFrontier {
        ledger: Ledger,
        observed: Arc<Mutex<Vec<Usage>>>,
        drops: Arc<AtomicUsize>,
    }
    impl Drop for CachedFrontier {
        fn drop(&mut self) {
            self.observed.lock().unwrap().push(self.ledger.total());
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn prepared_cache(
        ledger: &Ledger,
        observed: &Arc<Mutex<Vec<Usage>>>,
        drops: &Arc<AtomicUsize>,
    ) -> Held<Arc<CachedFrontier>> {
        let debit = ledger
            .retain(Usage::new(size_of::<CachedFrontier>(), 0), unlimited)
            .unwrap();
        Held::new(
            Arc::new(CachedFrontier {
                ledger: ledger.clone(),
                observed: observed.clone(),
                drops: drops.clone(),
            }),
            debit,
        )
    }

    #[test]
    fn weak_frontier_cache_shares_active_capsules_prices_expiry_and_readopts_same_identity() {
        let (ledger, registry) = new_registry();
        let external = Arc::new(123u64);
        let identity = MaterializationId::fresh().unwrap();
        let root = registry
            .adopt_object(identity, Usage::new(size_of::<u64>(), 0), unlimited)
            .unwrap();
        let baseline = ledger.total();
        let body = Usage::new(size_of::<CachedFrontier>(), 0);
        let observed = Arc::new(Mutex::new(Vec::new()));
        let drops = Arc::new(AtomicUsize::new(0));
        assert!(root.attachment::<CachedFrontier>().is_none());
        assert!(
            ledger
                .retain(body, |next| {
                    if next.bytes() > baseline.bytes() + body.bytes() - 1 {
                        Err(DomError::Cancelled)
                    } else {
                        Ok(())
                    }
                })
                .is_err()
        );
        assert_eq!(ledger.total(), baseline);
        assert_eq!(
            drops.load(Ordering::Relaxed),
            0,
            "refused preadmission allocates no target body"
        );
        let current = root.attach(prepared_cache(&ledger, &observed, &drops));
        let alias = root.attachment::<CachedFrontier>().unwrap();
        assert!(current.shares_target(&alias));
        assert_eq!(ledger.total(), baseline.checked_add(body).unwrap());
        drop(current);
        drop(alias);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        assert!(root.attachment::<CachedFrontier>().is_none());
        assert_eq!(
            ledger.total(),
            baseline.checked_add(body).unwrap(),
            "expired Weak target body remains allocated and charged"
        );
        let replacement = root.attach(prepared_cache(&ledger, &observed, &drops));
        assert_eq!(ledger.total(), baseline.checked_add(body).unwrap());
        let same_root = registry
            .adopt_object(identity, Usage::new(size_of::<u64>(), 0), unlimited)
            .unwrap();
        assert!(Arc::ptr_eq(&same_root.0, &root.0));
        assert_eq!(ledger.total(), baseline.checked_add(body).unwrap());
        drop(root);
        drop(same_root);
        assert_eq!(ledger.total(), baseline.checked_add(body).unwrap());
        drop(replacement);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert_eq!(*external, 123);
        let reentered = registry
            .adopt_object(identity, Usage::new(size_of::<u64>(), 0), unlimited)
            .unwrap();
        assert_eq!(reentered.identity(), identity);
        assert!(reentered.attachment::<CachedFrontier>().is_none());
        drop(reentered);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert!(
            observed
                .lock()
                .unwrap()
                .iter()
                .all(|usage| usage.bytes() >= baseline.bytes() + body.bytes())
        );
    }

    #[test]
    fn concurrent_weak_cache_attachment_drops_losing_data_before_its_body_debit() {
        use std::sync::mpsc;
        let (ledger, registry) = new_registry();
        let root = registry
            .adopt_object(
                MaterializationId::fresh().unwrap(),
                Usage::new(8, 0),
                unlimited,
            )
            .unwrap();
        let baseline = ledger.total();
        let observed = Arc::new(Mutex::new(Vec::new()));
        let drops = Arc::new(AtomicUsize::new(0));
        let first = prepared_cache(&ledger, &observed, &drops);
        let second = prepared_cache(&ledger, &observed, &drops);
        let peak = baseline
            .checked_add(Usage::new(2 * size_of::<CachedFrontier>(), 0))
            .unwrap();
        assert_eq!(ledger.total(), peak);
        let (ready, observed_ready) = mpsc::channel();
        let spawn = |prepared, resume: mpsc::Receiver<()>| {
            let root = root.clone();
            let ready = ready.clone();
            observed_worker(move || {
                ready.send(()).unwrap();
                await_stage(
                    &resume,
                    "candidate capsule prepared; awaiting cache attachment release",
                );
                root.attach(prepared)
            })
        };
        let (resume_first, first_resume) = mpsc::channel();
        let (resume_second, second_resume) = mpsc::channel();
        let first = spawn(first, first_resume);
        let second = spawn(second, second_resume);
        await_stage(
            &observed_ready,
            "both candidate workers started; awaiting first ready",
        );
        await_stage(
            &observed_ready,
            "first candidate ready; awaiting second ready",
        );
        resume_first.send(()).unwrap();
        resume_second.send(()).unwrap();
        let first = finish_worker(first, "both capsule attachments released");
        let second = finish_worker(
            second,
            "one capsule attachment completed; awaiting second result",
        );
        assert!(first.shares_target(&second));
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        assert_eq!(observed.lock().unwrap()[0], peak);
        assert_eq!(
            ledger.total(),
            baseline
                .checked_add(Usage::new(size_of::<CachedFrontier>(), 0))
                .unwrap()
        );
        drop(first);
        drop(second);
        drop(root);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
    }

    #[test]
    fn mismatched_dependency_and_attachment_domains_refuse_at_private_invariants_without_early_data_release()
     {
        let (ledger, registry) = new_registry();
        let foreign = Ledger::default();
        let dependency = Dependency::new(1u64, &foreign, unlimited).unwrap();
        let reserved = Record::reserve().unwrap();
        let debit = ledger
            .retain_shared(Usage::new(size_of::<u64>(), 0), unlimited)
            .unwrap();
        let record = ControlledRecord::from_parts(reserved.materialize(1u64), debit)
            .with_dependency(dependency);
        let callbacks = AtomicUsize::new(0);
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Bank::default().set_controlled(
                &BankCustody::default(),
                0,
                record,
                false,
                child_totals(&1).unwrap(),
                &registry,
                |_| {
                    callbacks.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                },
            )
        }));
        assert!(refused.is_err());
        assert_eq!(callbacks.load(Ordering::Relaxed), 0);
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert_eq!(foreign.total(), Usage::ZERO);

        let dependency = Dependency::new(1u64, &foreign, unlimited).unwrap();
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ControlledRecord::new(
                1u64,
                RecordDescription::with_dependency(Usage::new(size_of::<u64>(), 0), dependency),
                &registry,
                |_| {
                    callbacks.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                },
            )
        }));
        assert!(refused.is_err());
        assert_eq!(callbacks.load(Ordering::Relaxed), 0);
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert_eq!(foreign.total(), Usage::ZERO);

        let mut historical = Bank::default();
        historical
            .set(0, 1u64, false, child_totals(&1).unwrap())
            .unwrap();
        let described = std::cell::Cell::new(false);
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            historical.adopt_controlled(
                &registry,
                |_| {
                    let dependency = Dependency::new(1u64, &foreign, unlimited)?;
                    described.set(true);
                    Ok(RecordDescription::with_dependency(
                        Usage::new(size_of::<u64>(), 0),
                        dependency,
                    ))
                },
                |_| {
                    // Page admission precedes discovering this descriptor;
                    // no subsequent admission may observe a foreign receipt.
                    if described.get() {
                        callbacks.fetch_add(1, Ordering::Relaxed);
                    }
                    Ok(())
                },
            )
        }));
        assert!(refused.is_err());
        assert!(described.get());
        assert_eq!(callbacks.load(Ordering::Relaxed), 0);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert_eq!(foreign.total(), Usage::ZERO);

        struct Body {
            ledger: Ledger,
            seen: Arc<Mutex<Option<Usage>>>,
        }
        impl Drop for Body {
            fn drop(&mut self) {
                *self.seen.lock().unwrap() = Some(self.ledger.total());
            }
        }
        let root = registry
            .adopt_object(
                MaterializationId::fresh().unwrap(),
                Usage::new(8, 0),
                unlimited,
            )
            .unwrap();
        let seen = Arc::new(Mutex::new(None));
        let bytes = Usage::new(size_of::<Body>(), 0);
        let debit = foreign.retain(bytes, unlimited).unwrap();
        let body = Held::new(
            Arc::new(Body {
                ledger: foreign.clone(),
                seen: seen.clone(),
            }),
            debit,
        );
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| root.attach(body)));
        assert!(refused.is_err());
        assert_eq!(*seen.lock().unwrap(), Some(bytes));
        assert_eq!(foreign.total(), Usage::ZERO);
        assert!(root.attachment::<Body>().is_none());
        drop(root);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
    }

    #[test]
    fn controlled_record_honors_an_original_prepaid_description() {
        let ledger = Ledger::default();
        let registry = CustodyRegistry::new(&ledger, unlimited).unwrap();
        let usage = Usage::new(size_of::<u64>(), 0);
        let prepaid = ledger.retain_shared(usage, unlimited).unwrap();
        let identity = prepaid.allocation_identity();
        let record = ControlledRecord::new(
            19u64,
            RecordDescription::from(usage).with_retained(prepaid.clone()),
            &registry,
            unlimited,
        )
        .unwrap();
        let RecordRetention::Registered(control) = &record.custody else {
            panic!("registered constructor");
        };
        assert_eq!(control._payload.allocation_identity(), identity);
        let external = record.value().clone();
        drop(prepaid);
        assert_eq!(**record.value(), 19);
        drop(record);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), CustodyRegistry::administration_usage());
        assert_eq!(*external, 19);
    }

    #[test]
    fn missing_path_charge_counts_the_root_it_inspects() {
        let mut bank = Bank::default();
        bank.set(0, 7u64, false, Totals::default()).unwrap();
        let origin = MaterializationId::fresh().unwrap();
        assert_eq!(
            bank.path_index_charge_not_shared_with(&bank, 16),
            IndexCharge::default()
        );
        assert_eq!(
            bank.origin_path_index_charge(16, origin).unwrap(),
            IndexCharge::default()
        );
        if !scale_metrics::ENABLED {
            return;
        }
        let measurement = scale_metrics::Measurement::begin().unwrap();
        bank.path_index_charge_not_shared_with(&bank, 16);
        let counts = measurement
            .finish(scale_metrics::Hooks::one(Domain::Bank))
            .unwrap();
        assert_eq!(counts.bank_page_visits, 1);
        let measurement = scale_metrics::Measurement::begin().unwrap();
        bank.origin_path_index_charge(16, origin).unwrap();
        let counts = measurement
            .finish(scale_metrics::Hooks::one(Domain::Bank))
            .unwrap();
        assert_eq!(counts.bank_page_visits, 1);
    }

    #[test]
    fn hosted_pages_keep_each_real_issuer_receipt_across_shared_descendants() {
        let first = hosted_page_source();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let second = crate::hosted::DomHostedSource::new(
            crate::hosted::DomHostedSourceId {
                owner: first.id().owner,
                serial: 1,
            },
            first.limits(),
            &cancel,
        )
        .unwrap();
        let used = |source: &crate::hosted::DomHostedSource| {
            source.limits().retained_bytes() - source.remaining_bytes().unwrap()
        };
        let before_first = used(&first);
        let before_second = used(&second);
        let origin = MaterializationId::fresh().unwrap();
        let mut old = Bank::default();
        for index in [0, 16] {
            old.set_hosted_admitted(
                index,
                Record::new(index as u64).unwrap(),
                false,
                Totals::default(),
                origin,
                |charge| first.retain(charge.bytes, &cancel),
            )
            .unwrap();
        }
        let old_graph = old.inspect_graph(|_| Ok(Totals::default())).unwrap();
        assert_eq!(used(&first), before_first + old_graph.index_charge().bytes);
        let mut next = old.clone();
        next.set_hosted_admitted(
            0,
            Record::new(41u64).unwrap(),
            false,
            Totals::default(),
            MaterializationId::fresh().unwrap(),
            |charge| second.retain(charge.bytes, &cancel),
        )
        .unwrap();
        let next_graph = next.inspect_graph(|_| Ok(Totals::default())).unwrap();
        let old_pages = old_graph
            .pages()
            .map(|(address, _, charge)| (address, charge.bytes))
            .collect::<std::collections::BTreeMap<_, _>>();
        let shared: usize = next_graph
            .pages()
            .filter(|(address, _, _)| old_pages.contains_key(address))
            .map(|(_, _, charge)| charge.bytes)
            .sum();
        let fresh: usize = next_graph
            .pages()
            .filter(|(address, _, _)| !old_pages.contains_key(address))
            .map(|(_, _, charge)| charge.bytes)
            .sum();
        assert!(shared > 0);
        assert!(fresh > shared);
        assert_eq!(used(&first), before_first + old_graph.index_charge().bytes);
        assert_eq!(used(&second), before_second + fresh);
        drop(old);
        assert_eq!(used(&first), before_first + shared);
        drop(first);
        assert_eq!(next.get(16), Some(&16));
        assert_eq!(next.get(0), Some(&41));
        let alias = next.clone();
        drop(next);
        assert_eq!(used(&second), before_second + fresh);
        drop(alias);
        assert_eq!(used(&second), before_second);
    }

    fn hosted_page_source() -> crate::hosted::DomHostedSource {
        use crate::hosted::{
            DomHostedLimits, DomHostedRecordLimits, DomHostedSource, DomHostedSourceId,
        };
        DomHostedSource::new(
            DomHostedSourceId {
                owner: super::super::DomOwnerId::allocate().unwrap(),
                serial: 0,
            },
            DomHostedLimits::new(
                DomHostedRecordLimits::new(0, 0, 0, 0, 0, 0, 0),
                8_000_000,
                8_000_000,
                10_000,
            ),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap()
    }

    #[test]
    fn hosted_page_aliases_keep_one_original_receipt_and_data_drops_first() {
        use std::sync::atomic::{AtomicBool, Ordering};
        struct Probe<'a> {
            source: &'a crate::hosted::DomHostedSource,
            before: usize,
            dropped: &'a AtomicBool,
        }
        impl Drop for Probe<'_> {
            fn drop(&mut self) {
                let used =
                    self.source.limits().retained_bytes() - self.source.remaining_bytes().unwrap();
                assert!(
                    used > self.before,
                    "page data must drop before original producer receipt"
                );
                self.dropped.store(true, Ordering::Relaxed);
            }
        }
        let source = hosted_page_source();
        let cancel = AtomicBool::new(false);
        let before = source.limits().retained_bytes() - source.remaining_bytes().unwrap();
        let dropped = AtomicBool::new(false);
        let mut bank = Bank::default();
        let charge = bank
            .set_hosted_admitted(
                0,
                Record::new(Probe {
                    source: &source,
                    before,
                    dropped: &dropped,
                })
                .unwrap(),
                false,
                Totals::default(),
                MaterializationId::fresh().unwrap(),
                |charge| source.retain(charge.bytes, &cancel),
            )
            .unwrap();
        assert_eq!(
            source.limits().retained_bytes() - source.remaining_bytes().unwrap(),
            before + charge.bytes
        );
        let alias = bank.clone();
        drop(bank);
        assert!(!dropped.load(Ordering::Relaxed));
        assert_eq!(
            source.limits().retained_bytes() - source.remaining_bytes().unwrap(),
            before + charge.bytes
        );
        drop(alias);
        assert!(dropped.load(Ordering::Relaxed));
        assert_eq!(
            source.limits().retained_bytes() - source.remaining_bytes().unwrap(),
            before
        );
    }

    #[test]
    fn hosted_page_failed_growth_reclaims_new_receipts_without_refunding_work() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let mut source = hosted_page_source();
        let cancel = AtomicBool::new(false);
        let limit = 3;
        let limits = source.limits();
        source
            .set_limits(
                crate::hosted::DomHostedLimits::new(
                    limits.records(),
                    limits.retained_bytes(),
                    limits.scratch_bytes(),
                    limit,
                ),
                &cancel,
            )
            .unwrap();
        let work = source.work(&cancel).unwrap();
        let mut bank = Bank::default();
        bank.set_hosted_admitted(
            0,
            Record::new(1u8).unwrap(),
            false,
            Totals::default(),
            MaterializationId::fresh().unwrap(),
            |charge| source.retain(charge.bytes, &cancel),
        )
        .unwrap();
        let alias = bank.clone();
        let before = source.limits().retained_bytes() - source.remaining_bytes().unwrap();
        let result = bank.set_hosted_admitted(
            4096,
            Record::new(2u8).unwrap(),
            false,
            Totals::default(),
            MaterializationId::fresh().unwrap(),
            |charge| {
                work.step(&cancel)?;
                source.retain(charge.bytes, &cancel)
            },
        );
        assert!(
            matches!(result, Err(crate::hosted::DomHostedError::WorkLimit { limit: observed }) if observed == limit),
            "expected original producer work refusal, got {result:?}"
        );
        assert!(bank.shares_root(&alias));
        assert_eq!(bank.get(0), Some(&1));
        assert_eq!(bank.get(4096), None);
        assert_eq!(work.spent.load(Ordering::Relaxed), limit);
        assert_eq!(
            source.limits().retained_bytes() - source.remaining_bytes().unwrap(),
            before
        );
        drop(source);
        assert_eq!(alias.get(0), Some(&1));
    }
}

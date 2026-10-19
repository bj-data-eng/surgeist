//! The one canonical navigable/entry/DocumentState/history-collection graph.
//! Rows use the original issuer's identities; no independent allocator or
//! Runtime activity/history mirror is admitted here.
use super::*;
use std::cell::Cell;
use std::ops::Deref;
use surgeist_html_semantics::HtmlAboutBaseUrl;

pub(in crate::store) mod documents;

#[cfg(test)]
#[inline]
fn semantic_item() {
    if crate::store::scale_metrics::ENABLED {
        crate::store::scale_metrics::record(
            crate::store::scale_metrics::Event::HostedSemanticItemVisited,
        );
    }
}

#[cfg(test)]
#[inline]
fn copied_items<T>(items: usize) {
    if crate::store::scale_metrics::ENABLED {
        crate::store::scale_metrics::record(
            crate::store::scale_metrics::Event::SequenceMaterialized {
                bytes: items * size_of::<T>(),
            },
        );
    }
}

/// One synchronous graph call borrows the original issuer and cumulative Work.
/// `scratch_base` is the caller's other simultaneously live formation extent.
pub(crate) struct Context<'a> {
    source: &'a DomHostedSource,
    work: &'a Work,
    cancel: &'a AtomicBool,
    scratch: Cell<usize>,
    formation: Option<&'a source::formation::Formation>,
    origin: Cell<Option<custody::MaterializationId>>,
}
impl<'a> Context<'a> {
    pub(crate) fn new(
        source: &'a DomHostedSource,
        work: &'a Work,
        cancel: &'a AtomicBool,
        scratch_base: usize,
    ) -> Self {
        Self {
            source,
            work,
            cancel,
            scratch: Cell::new(scratch_base),
            formation: None,
            origin: Cell::new(None),
        }
    }
    /// The caller owns a fresh fork for this single atomic graph call. On
    /// refusal it drops that fork; success absorbs only its live output.
    pub(crate) fn with_formation(mut self, formation: &'a source::formation::Formation) -> Self {
        self.formation = Some(formation);
        self
    }
    fn parent(
        &self,
        payload: source::storage::Payload,
    ) -> Result<ParentCredit<'a>, DomHostedError> {
        let mut credit = ParentCredit {
            formation: self.formation,
            payload: source::storage::Payload::default(),
        };
        credit.add(payload, self.cancel)?;
        Ok(credit)
    }
    fn origin(&self) -> Result<custody::MaterializationId, DomError> {
        if let Some(origin) = self.origin.get() {
            return Ok(origin);
        }
        let origin = custody::MaterializationId::fresh()?;
        self.origin.set(Some(origin));
        Ok(origin)
    }
    fn release_output(&self, payload: source::storage::Payload) {
        if let Some(formation) = self.formation {
            formation.release_payload(payload);
        }
        self.scratch.set(
            self.scratch
                .get()
                .checked_sub(payload.bytes)
                .expect("same call's discarded staged output"),
        );
    }
    fn call(&self) -> Call<'_, 'a> {
        Call {
            context: self,
            baseline: self.scratch.get(),
            previous_origin: self.origin.replace(None),
        }
    }
    fn step(&self) -> Result<(), DomHostedError> {
        self.work.step(self.cancel)
    }
    fn scratch(&self, bytes: usize) -> Result<Scratch<'_, 'a>, DomHostedError> {
        let mut guard = Scratch {
            context: self,
            bytes: 0,
        };
        guard.replace(bytes)?;
        Ok(guard)
    }
}
/// The original Context's one synchronous Graph-method transfer boundary.
/// Declared before candidate owners, it releases staged credit only after their
/// backing drops on refusal, or when finished output transfers to the caller.
struct Call<'a, 'b> {
    context: &'a Context<'b>,
    baseline: usize,
    previous_origin: Option<custody::MaterializationId>,
}
impl Drop for Call<'_, '_> {
    fn drop(&mut self) {
        assert!(
            self.context.scratch.get() >= self.baseline,
            "call cannot release its caller's scratch"
        );
        self.context.scratch.set(self.baseline);
        self.context.origin.set(self.previous_origin);
    }
}

/// This is a debit in the original Formation, never another budget owner.
/// Commit transfers already admitted output to that same fresh Formation.
struct ParentCredit<'a> {
    formation: Option<&'a source::formation::Formation>,
    payload: source::storage::Payload,
}
impl ParentCredit<'_> {
    fn add(
        &mut self,
        payload: source::storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let mut next = self.payload;
        next.append(payload)?;
        if let Some(formation) = self.formation {
            formation.retain_payload(payload, cancel)?;
        }
        self.payload = next;
        Ok(())
    }
    fn release(&mut self, payload: source::storage::Payload) {
        self.payload.bytes = self
            .payload
            .bytes
            .checked_sub(payload.bytes)
            .expect("owned formation bytes");
        self.payload.units = self
            .payload
            .units
            .checked_sub(payload.units)
            .expect("owned formation units");
        if let Some(formation) = self.formation {
            formation.release_payload(payload);
        }
    }
    fn commit(mut self) {
        self.payload = source::storage::Payload::default();
    }
}
impl Drop for ParentCredit<'_> {
    fn drop(&mut self) {
        if let Some(formation) = self.formation {
            formation.release_payload(self.payload);
        }
    }
}

struct Scratch<'a, 'b> {
    context: &'a Context<'b>,
    bytes: usize,
}
impl Scratch<'_, '_> {
    fn replace(&mut self, bytes: usize) -> Result<(), DomHostedError> {
        let total = self
            .context
            .scratch
            .get()
            .checked_sub(self.bytes)
            .and_then(|old| old.checked_add(bytes))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            total,
            self.context.source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        self.context.scratch.set(total);
        self.bytes = bytes;
        Ok(())
    }
    fn release(&mut self, bytes: usize) {
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .expect("owned scratch occurrence");
        self.context.scratch.set(
            self.context
                .scratch
                .get()
                .checked_sub(bytes)
                .expect("live scratch occurrence"),
        );
    }
    fn stage(mut self) {
        // Transfer this same counter credit to the synchronous Call scope.
        self.bytes = 0;
    }
}
impl Drop for Scratch<'_, '_> {
    fn drop(&mut self) {
        self.context
            .scratch
            .set(self.context.scratch.get() - self.bytes);
    }
}

pub(crate) struct Initial<'a> {
    pub(crate) creation: DomHostedCreation,
    pub(crate) initialization: DomHostedEntryInitialization,
    pub(crate) origin: crate::DomOpaqueOrigin,
    pub(crate) parent: Option<(DomHostedNavigableId, NodeId)>,
    pub(crate) target_name: Option<&'a DocumentString>,
    pub(crate) about_base_url: &'a HtmlAboutBaseUrl,
    pub(crate) initiator_origin: Option<&'a crate::DomOpaqueOrigin>,
}

#[derive(Debug)]
pub(crate) struct Values<T> {
    pub(crate) values: Vec<T>,
    _retained: Retained,
}
impl<T: Copy> Values<T> {
    fn form(values: &[T], context: &Context<'_>) -> Result<Arc<Self>, DomHostedError> {
        let mut candidate = Buffer::new(values.len(), context)?;
        for value in values {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            candidate.values.push(*value);
            #[cfg(test)]
            copied_items::<T>(1);
        }
        Ok(candidate.finish())
    }
    fn bytes(&self) -> Result<usize, DomError> {
        self.values
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or(DomError::RepresentationExhausted)
    }
}

/// Scratch and retained admission cover both buffers during exact replacement.
/// Vec/retained fields precede the scratch credit on every refusal and drop.
struct Buffer<'a, 'b, T> {
    values: Vec<T>,
    retained: Retained,
    scratch: Scratch<'a, 'b>,
    parent: ParentCredit<'b>,
}
impl<'a, 'b, T: Copy> Buffer<'a, 'b, T> {
    fn new(capacity: usize, context: &'a Context<'b>) -> Result<Self, DomHostedError> {
        context.step()?;
        let bytes = capacity
            .checked_mul(size_of::<T>())
            .and_then(|n| n.checked_add(size_of::<Values<T>>()))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let scratch = context.scratch(bytes)?;
        let retained = context.source.retain(bytes, context.cancel)?;
        let parent = context.parent(source::storage::Payload { bytes, units: 0 })?;
        let mut candidate = Self {
            values: Vec::new(),
            retained,
            scratch,
            parent,
        };
        candidate
            .values
            .try_reserve_exact(capacity)
            .map_err(|_| DomHostedError::AllocationFailed)?;
        #[cfg(test)]
        if candidate.values.capacity() != 0 {
            transaction::auxiliary::allocated(candidate.values.capacity() * size_of::<T>());
        }
        let actual = candidate
            .values
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|n| n.checked_add(size_of::<Values<T>>()))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        candidate.scratch.replace(actual)?;
        candidate.parent.add(
            source::storage::Payload {
                bytes: actual
                    .checked_sub(bytes)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                units: 0,
            },
            context.cancel,
        )?;
        candidate.retained.append(
            context.source.retain(
                actual
                    .checked_sub(bytes)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                context.cancel,
            )?,
        )?;
        Ok(candidate)
    }
    fn push(&mut self, value: T) -> Result<(), DomHostedError> {
        let context = self.scratch.context;
        context.step()?;
        if self.values.len() == self.values.capacity() {
            let mut next = Self::new(
                self.values
                    .len()
                    .checked_add(1)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                context,
            )?;
            for old in &self.values {
                context.step()?;
                next.values.push(*old);
                #[cfg(test)]
                copied_items::<T>(1);
            }
            std::mem::swap(self, &mut next);
        }
        self.values.push(value);
        #[cfg(test)]
        copied_items::<T>(1);
        Ok(())
    }
    fn contains(&self, value: &T) -> Result<bool, DomHostedError>
    where
        T: PartialEq,
    {
        for old in &self.values {
            #[cfg(test)]
            semantic_item();
            self.scratch.context.step()?;
            if old == value {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn finish(self) -> Arc<Values<T>> {
        let Self {
            values,
            retained,
            scratch,
            parent,
        } = self;
        scratch.stage();
        parent.commit();
        let values = Arc::new(Values {
            values,
            _retained: retained,
        });
        #[cfg(test)]
        transaction::auxiliary::allocated(size_of::<Values<T>>());
        values
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Navigable {
    pub(crate) id: DomHostedNavigableId,
    pub(crate) host: DomHostedHostId,
    pub(crate) context: DomHostedBrowsingContextId,
    pub(crate) initial_document: DocumentId,
    pub(crate) global: DomHostedGlobalId,
    pub(crate) parent: Option<DomHostedNavigableId>,
    pub(crate) traversable: DomHostedNavigableId,
    pub(crate) current_step: Option<u64>,
    /// Set only by acceptance of the original C queue handoff.
    creation_operation: Option<DomHostedOperationId>,
    history_attached: bool,
    pub(crate) former_parent: Option<DomHostedNavigableId>,
    pub(crate) container: Option<NodeId>,
    pub(crate) current: DomHostedEntryId,
    pub(crate) active: DomHostedEntryId,
    pub(crate) entries: Arc<Values<DomHostedEntryId>>,
    pub(crate) retiring: bool,
    pub(crate) retired: bool,
}
#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) id: DomHostedEntryId,
    pub(crate) document: Option<DocumentId>,
    pub(crate) initial_document: DocumentId,
    pub(crate) origin: crate::DomOpaqueOrigin,
    pub(crate) target_name: Option<DocumentString>,
    pub(crate) about_base_url: HtmlAboutBaseUrl,
    pub(crate) initiator_origin: Option<crate::DomOpaqueOrigin>,
    pub(crate) initialization: DomHostedEntryInitialization,
    pub(crate) step: Option<u64>,
    pub(crate) nested: Arc<Values<DomHostedNavigableId>>,
    pub(crate) history: Option<(usize, usize)>,
}
#[derive(Debug)]
pub(in crate::store) struct Row<T> {
    origin: custody::MaterializationId,
    value: T,
    _retained: Retained,
}
impl<T> Deref for Row<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
/// Fixed row custody is admitted before any retained-handle clone. Its
/// inline value precedes both credits on refusal and never escapes uncharged.
struct Changed<'a, 'b, T> {
    identity: bank::ReservedRecord<Row<T>>,
    origin: custody::MaterializationId,
    value: T,
    retained: Retained,
    scratch: Scratch<'a, 'b>,
    credit: ParentCredit<'b>,
}
impl<T> Deref for Changed<'_, '_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T> std::ops::DerefMut for Changed<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
fn changed<'a, 'b, T>(
    context: &'a Context<'b>,
    extra: source::storage::Payload,
    retained_extra: usize,
    scratch_extra: usize,
    form: impl FnOnce() -> Result<T, DomHostedError>,
) -> Result<Changed<'a, 'b, T>, DomHostedError> {
    context.step()?;
    let identity = bank::Record::<Row<T>>::reserve()?;
    let origin = context.origin()?;
    let bytes = size_of::<Row<T>>()
        .checked_add(retained_extra)
        .ok_or(DomHostedError::RepresentationExhausted)?;
    let scratch = context.scratch(
        size_of::<Row<T>>()
            .checked_add(scratch_extra)
            .ok_or(DomHostedError::RepresentationExhausted)?,
    )?;
    let retained = context.source.retain(bytes, context.cancel)?;
    let mut payload = extra;
    payload.add_bytes(size_of::<Row<T>>())?;
    let parent = context.parent(payload)?;
    Ok(Changed {
        identity,
        origin,
        value: form()?,
        retained,
        scratch,
        credit: parent,
    })
}
fn row<T>(
    value: Changed<'_, '_, T>,
    context: &Context<'_>,
) -> Result<bank::Record<Row<T>>, DomHostedError> {
    context.step()?;
    let Changed {
        identity,
        origin,
        value,
        retained,
        scratch,
        credit: parent,
    } = value;
    scratch.stage();
    parent.commit();
    Ok(identity.materialize(Row {
        origin,
        value,
        _retained: retained,
    }))
}
impl Entry {
    fn document_state_payload(
        target_name: Option<&DocumentString>,
        about_base_url: &HtmlAboutBaseUrl,
    ) -> Result<source::storage::Payload, DomError> {
        let mut payload = source::storage::Payload::default();
        if let Some(name) = target_name {
            payload.string(name)?;
        }
        if let HtmlAboutBaseUrl::KnownRecord(record) = about_base_url {
            payload.producer(record.retained_storage_usage())?;
        }
        Ok(payload)
    }
    fn retained_copy<'a, 'b>(
        &self,
        context: &'a Context<'b>,
    ) -> Result<Changed<'a, 'b, Self>, DomHostedError> {
        context.step()?;
        let fields = Self::document_state_payload(self.target_name.as_ref(), &self.about_base_url)?;
        let mut payload = fields;
        payload.add_bytes(self.nested.bytes()?)?;
        payload.add_units(72)?;
        changed(context, payload, fields.bytes, payload.bytes, || {
            Ok(Self {
                id: self.id,
                document: self.document,
                initial_document: self.initial_document,
                origin: self.origin.clone(),
                target_name: self.target_name.clone(),
                about_base_url: self.about_base_url.clone(),
                initiator_origin: self.initiator_origin.clone(),
                initialization: DomHostedEntryInitialization::new(
                    self.initialization.navigation_key,
                    self.initialization.navigation_id,
                ),
                step: self.step,
                nested: self.nested.clone(),
                history: self.history,
            })
        })
    }
}

impl Navigable {
    fn payload(&self) -> Result<source::storage::Payload, DomError> {
        Ok(source::storage::Payload {
            bytes: size_of::<Row<Self>>()
                .checked_add(self.entries.bytes()?)
                .ok_or(DomError::RepresentationExhausted)?,
            units: 0,
        })
    }
}
impl Entry {
    fn payload(&self) -> Result<source::storage::Payload, DomError> {
        let mut payload =
            Self::document_state_payload(self.target_name.as_ref(), &self.about_base_url)?;
        payload.add_bytes(size_of::<Row<Self>>())?;
        payload.add_bytes(self.nested.bytes()?)?;
        payload.add_units(72)?;
        Ok(payload)
    }
}

/// Captured nonchanging Global-job input. It does not acknowledge execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Job {
    pub(crate) navigable: DomHostedNavigableId,
    pub(crate) entry: DomHostedEntryId,
    pub(crate) document: DocumentId,
    pub(crate) global: DomHostedGlobalId,
    pub(crate) length: usize,
    pub(crate) index: usize,
}
#[derive(Debug)]
pub(crate) struct Update {
    pub(crate) traversable: DomHostedNavigableId,
    pub(crate) target_step: u64,
    pub(crate) jobs: Arc<Values<Job>>,
}
pub(crate) struct Graph {
    identity: custody::MaterializationId,
    origin: custody::MaterializationId,
    accounted: (usize, usize),
    navigables: bank::Bank<Row<Navigable>>,
    entries: bank::Bank<Row<Entry>>,
    navigable_ids: transaction::auxiliary::index::Index<transaction::auxiliary::index::Ordinal>,
    entry_ids: transaction::auxiliary::index::Index<transaction::auxiliary::index::Ordinal>,
    documents: documents::Index,
    parent_identity: Option<custody::MaterializationId>,
    discarded: source::storage::Payload,
    _retained: Retained,
}
impl std::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph")
            .field("identity", &self.identity)
            .field("accounted", &self.accounted)
            .field("navigables", &self.navigables.len())
            .field("entries", &self.entries.len())
            .finish()
    }
}
impl PartialEq for Graph {
    fn eq(&self, other: &Self) -> bool {
        self.navigables
            .same_records_by(&other.navigables, &|a, b| std::ptr::eq(a, b))
            && self
                .entries
                .same_records_by(&other.entries, &|a, b| std::ptr::eq(a, b))
    }
}
impl Eq for Graph {}
impl<T> Row<T> {
    pub(in crate::store) fn custody_origin(&self) -> custody::MaterializationId {
        self.origin
    }
}
impl Row<Navigable> {
    pub(in crate::store) fn custody_payload(&self) -> Result<custody::Usage, DomError> {
        let p = self.value.payload()?;
        Ok(custody::Usage::new(p.bytes, p.units))
    }
}
impl Row<Entry> {
    pub(in crate::store) fn custody_payload(&self) -> Result<custody::Usage, DomError> {
        let p = self.value.payload()?;
        Ok(custody::Usage::new(p.bytes, p.units))
    }
}
fn nav_key(id: DomHostedNavigableId) -> transaction::auxiliary::index::Key {
    transaction::auxiliary::index::Key::new(id.source, id.slot, id.generation)
}
fn entry_key(id: DomHostedEntryId) -> transaction::auxiliary::index::Key {
    transaction::auxiliary::index::Key::new(id.source, id.slot, id.generation)
}
fn history_lookup_error(error: DomError) -> DomHostedError {
    match error {
        DomError::ForeignOwner => DomHostedError::StaleIdentity,
        other => other.into(),
    }
}

impl Graph {
    pub(in crate::store) fn custody_identity(&self) -> custody::MaterializationId {
        self.identity
    }
    pub(in crate::store) fn custody_usage(&self) -> Result<custody::Usage, DomError> {
        Ok(custody::Usage::new(size_of::<Self>(), 0))
    }
    pub(in crate::store) fn custody_origin(&self) -> custody::MaterializationId {
        self.origin
    }
    pub(in crate::store) fn navigable_bank(&self) -> &bank::Bank<Row<Navigable>> {
        &self.navigables
    }
    pub(in crate::store) fn entry_bank(&self) -> &bank::Bank<Row<Entry>> {
        &self.entries
    }
    pub(in crate::store) fn navigable_index(
        &self,
    ) -> &transaction::auxiliary::index::Index<transaction::auxiliary::index::Ordinal> {
        &self.navigable_ids
    }
    pub(in crate::store) fn entry_index(
        &self,
    ) -> &transaction::auxiliary::index::Index<transaction::auxiliary::index::Ordinal> {
        &self.entry_ids
    }
    pub(in crate::store) fn document_index(&self) -> &documents::Index {
        &self.documents
    }
    fn discarded_payload(&self, next: &Self) -> Result<source::storage::Payload, DomError> {
        if next.parent_identity != Some(self.identity) {
            return Err(DomError::RepresentationExhausted);
        }
        Ok(next.discarded)
    }
    #[cfg(test)]
    pub(in crate::store) fn inspect_materializations(
        &self,
    ) -> Result<Vec<(custody::MaterializationId, custody::Usage)>, DomError> {
        let mut output = vec![(self.identity, custody::Usage::new(size_of::<Self>(), 0))];
        let nav = self.navigables.inspect_graph(|row| {
            Ok(bank::Totals {
                bytes: size_of::<Row<Navigable>>()
                    + size_of::<Values<DomHostedEntryId>>()
                    + row.entries.values.capacity() * size_of::<DomHostedEntryId>(),
                units: 0,
                occurrences: 0,
            })
        })?;
        output.extend(nav.materializations());
        let entries = self.entries.inspect_graph(|row| {
            let mut bytes = size_of::<Row<Entry>>()
                + size_of::<Values<DomHostedNavigableId>>()
                + row.nested.values.capacity() * size_of::<DomHostedNavigableId>();
            let mut units = 72;
            if let Some(name) = &row.target_name {
                let cap = name.shared_code_units().capacity();
                bytes += size_of::<Vec<u16>>() + cap * size_of::<u16>();
                units += cap;
            }
            if let HtmlAboutBaseUrl::KnownRecord(record) = &row.about_base_url {
                let p = record.retained_storage_usage();
                bytes += p.retained_bytes();
                units += p.utf16_extent();
            }
            Ok(bank::Totals {
                bytes,
                units,
                occurrences: 0,
            })
        })?;
        output.extend(entries.materializations());
        output.extend(self.navigable_ids.inspect_materializations(|_| {
            Ok(custody::Usage::new(
                size_of::<transaction::auxiliary::index::Ordinal>(),
                0,
            ))
        })?);
        output.extend(self.entry_ids.inspect_materializations(|_| {
            Ok(custody::Usage::new(
                size_of::<transaction::auxiliary::index::Ordinal>(),
                0,
            ))
        })?);
        output.extend(self.documents.inspect_materializations()?);
        Ok(output)
    }
    /// Window.navigable is selected from actual active Document equality.
    pub(in crate::store) fn active_navigable_for_document_with(
        &self,
        document: DocumentId,
        mut step: impl FnMut() -> Result<(), DomHostedError>,
    ) -> Result<Option<DomHostedNavigableId>, DomHostedError> {
        if self.navigables.is_empty() {
            return Ok(None);
        }
        step()?;
        let Some(member) = self
            .documents
            .get(document)
            .and_then(|group| group.rows.iter().next())
            .map(|(_, row)| row)
        else {
            return Ok(None);
        };
        step()?;
        self.navigables
            .get(member.navigable)
            .map(|row| Some(row.id))
            .ok_or(DomHostedError::StaleIdentity)
    }
    pub(crate) fn navigable(&self, id: DomHostedNavigableId) -> Result<&Navigable, DomHostedError> {
        self.navigable_with(id, || Ok(()))
    }
    pub(crate) fn entry(&self, id: DomHostedEntryId) -> Result<&Entry, DomHostedError> {
        self.entry_with(id, || Ok(()))
    }
    /// Lend the caller's original admission/cancellation to one fully qualified
    /// identity lookup. Historical observations supply a no-budget step.
    pub(crate) fn navigable_with(
        &self,
        id: DomHostedNavigableId,
        mut step: impl FnMut() -> Result<(), DomHostedError>,
    ) -> Result<&Navigable, DomHostedError> {
        step()?;
        let ordinal = self
            .navigable_ids
            .get(nav_key(id))
            .map_err(history_lookup_error)?
            .ok_or(DomHostedError::StaleIdentity)?;
        self.navigables
            .get(ordinal.value)
            .map(|row| &row.value)
            .ok_or(DomHostedError::StaleIdentity)
    }
    pub(crate) fn entry_with(
        &self,
        id: DomHostedEntryId,
        mut step: impl FnMut() -> Result<(), DomHostedError>,
    ) -> Result<&Entry, DomHostedError> {
        step()?;
        let ordinal = self
            .entry_ids
            .get(entry_key(id))
            .map_err(history_lookup_error)?
            .ok_or(DomHostedError::StaleIdentity)?;
        self.entries
            .get(ordinal.value)
            .map(|row| &row.value)
            .ok_or(DomHostedError::StaleIdentity)
    }
    #[cfg(test)]
    pub(crate) fn payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        let mut result = source::storage::Payload::default();
        for (_, usage) in self.inspect_materializations()? {
            step()?;
            result.add_bytes(usage.bytes())?;
            result.add_units(usage.units())?;
        }
        Ok(result)
    }
    fn form(
        old: Option<&Self>,
        nav: Option<bank::Record<Row<Navigable>>>,
        entry: Option<bank::Record<Row<Entry>>>,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        Self::form_documents(old, nav, entry, None, context)
    }
    fn form_documents(
        old: Option<&Self>,
        nav: Option<bank::Record<Row<Navigable>>>,
        entry: Option<bank::Record<Row<Entry>>>,
        clear_document: Option<DocumentId>,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        context.step()?;
        let identity = custody::MaterializationId::fresh()?;
        let origin = context.origin()?;
        let scratch = context.scratch(size_of::<Self>())?;
        let retained = context.source.retain(size_of::<Self>(), context.cancel)?;
        let parent = context.parent(source::storage::Payload {
            bytes: size_of::<Self>(),
            units: 0,
        })?;
        let mut candidate = Self {
            identity,
            origin,
            accounted: (0, 0),
            navigables: old.map_or_else(Default::default, |old| old.navigables.clone()),
            entries: old.map_or_else(Default::default, |old| old.entries.clone()),
            navigable_ids: old.map_or_else(Default::default, |old| old.navigable_ids.clone()),
            entry_ids: old.map_or_else(Default::default, |old| old.entry_ids.clone()),
            documents: old.map_or_else(Default::default, |old| old.documents.clone()),
            parent_identity: old.map(|old| old.identity),
            discarded: source::storage::Payload::default(),
            _retained: retained,
        };
        if old.is_some_and(|old| old.origin == origin) {
            candidate.discarded.add_bytes(size_of::<Self>())?;
        }
        let producer = transaction::auxiliary::index::Producer {
            source: context.source,
            work: context.work,
            formation: context.formation,
            cancel: context.cancel,
            origin,
            scratch: &context.scratch,
        };
        // This profile creates only paired initial rows; every other producer
        // replaces their fields without changing active/current Entry identity.
        let initial_document = match (&nav, &entry) {
            (Some(nav), Some(entry))
                if candidate
                    .navigable_ids
                    .get(nav_key(nav.value().id))?
                    .is_none()
                    && candidate
                        .entry_ids
                        .get(entry_key(entry.value().id))?
                        .is_none() =>
            {
                entry.value().document.map(|document| {
                    (
                        document,
                        candidate.navigables.len(),
                        candidate.entries.len(),
                    )
                })
            }
            _ => None,
        };
        if let Some(nav) = nav {
            context.step()?;
            let key = nav_key(nav.value().id);
            let ordinal = if let Some(ordinal) = candidate.navigable_ids.get(key)? {
                ordinal.value
            } else {
                let ordinal = candidate.navigables.len();
                let index = transaction::auxiliary::index::Ordinal::form(ordinal, &producer)?;
                let discarded = candidate.navigable_ids.insert(
                    key,
                    index,
                    custody::Usage::new(size_of::<transaction::auxiliary::index::Ordinal>(), 0),
                    false,
                    &producer,
                )?;
                candidate.discarded.add_bytes(discarded.bytes())?;
                ordinal
            };
            candidate.discarded.add_bytes(
                candidate
                    .navigables
                    .origin_path_index_charge(ordinal, origin)?
                    .bytes,
            )?;
            if let Some(prior) = candidate
                .navigables
                .get(ordinal)
                .filter(|row| row.origin == origin)
            {
                candidate.discarded.append(prior.value.payload()?)?;
            }
            let payload = nav.value().value.payload()?;
            producer.set(
                &mut candidate.navigables,
                ordinal,
                nav,
                custody::Usage::new(payload.bytes, payload.units),
            )?;
        }
        if let Some(entry) = entry {
            context.step()?;
            let key = entry_key(entry.value().id);
            let ordinal = if let Some(ordinal) = candidate.entry_ids.get(key)? {
                ordinal.value
            } else {
                let ordinal = candidate.entries.len();
                let index = transaction::auxiliary::index::Ordinal::form(ordinal, &producer)?;
                let discarded = candidate.entry_ids.insert(
                    key,
                    index,
                    custody::Usage::new(size_of::<transaction::auxiliary::index::Ordinal>(), 0),
                    false,
                    &producer,
                )?;
                candidate.discarded.add_bytes(discarded.bytes())?;
                ordinal
            };
            candidate.discarded.add_bytes(
                candidate
                    .entries
                    .origin_path_index_charge(ordinal, origin)?
                    .bytes,
            )?;
            if let Some(prior) = candidate
                .entries
                .get(ordinal)
                .filter(|row| row.origin == origin)
            {
                candidate.discarded.append(prior.value.payload()?)?;
            }
            let payload = entry.value().value.payload()?;
            producer.set(
                &mut candidate.entries,
                ordinal,
                entry,
                custody::Usage::new(payload.bytes, payload.units),
            )?;
        }
        if let Some((document, navigable, entry)) = initial_document {
            let discarded = candidate
                .documents
                .add(document, navigable, entry, &producer)?;
            candidate.discarded.add_bytes(discarded.bytes())?;
        }
        if let Some(document) = clear_document {
            let discarded = candidate.documents.clear(document, &producer)?;
            candidate.discarded.add_bytes(discarded.bytes())?;
        }
        let usage = custody::Usage::new(size_of::<Self>(), 0)
            .checked_add(transaction::auxiliary::index::bank_usage(
                &candidate.navigables,
            )?)?
            .checked_add(transaction::auxiliary::index::bank_usage(
                &candidate.entries,
            )?)?
            .checked_add(candidate.navigable_ids.usage()?)?
            .checked_add(candidate.entry_ids.usage()?)?
            .checked_add(candidate.documents.usage()?)?;
        candidate.accounted = (usage.bytes(), usage.units());
        parent.commit();
        scratch.stage();
        let candidate = Arc::new(candidate);
        #[cfg(test)]
        transaction::auxiliary::allocated(size_of::<Self>());
        Ok(candidate)
    }
    pub(crate) fn initial(
        old: Option<&Self>,
        input: Initial<'_>,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        let _call = context.call();
        let Initial {
            creation,
            initialization,
            origin,
            parent,
            target_name,
            about_base_url,
            initiator_origin,
        } = input;
        context.step()?;
        if let Some(old) = old {
            context.step()?;
            if old
                .navigable_ids
                .get(nav_key(creation.navigable))?
                .is_some()
            {
                return Err(DomHostedError::WrongPhase);
            }
            context.step()?;
            if old.entry_ids.get(entry_key(creation.entry))?.is_some() {
                return Err(DomHostedError::WrongPhase);
            }
        }
        let traversable = if let Some((id, _)) = parent {
            context.step()?;
            let parent = old.ok_or(DomHostedError::StaleIdentity)?.nav(id, context)?;
            if parent.retired || parent.retiring {
                return Err(DomHostedError::StaleIdentity);
            }
            parent.traversable
        } else {
            creation.navigable
        };
        let entries = Values::form(&[creation.entry], context)?;
        let nested = Values::form(&[], context)?;
        let nav = row(
            changed(context, source::storage::Payload::default(), 0, 0, || {
                Ok(Navigable {
                    id: creation.navigable,
                    host: creation.host,
                    context: creation.context,
                    initial_document: creation.document,
                    global: creation.global,
                    parent: parent.map(|p| p.0),
                    traversable,
                    current_step: parent.is_none().then_some(0),
                    creation_operation: None,
                    history_attached: parent.is_none(),
                    former_parent: None,
                    container: parent.map(|p| p.1),
                    current: creation.entry,
                    active: creation.entry,
                    entries,
                    retiring: false,
                    retired: false,
                })
            })?,
            context,
        )?;
        context.step()?;
        let fields = Entry::document_state_payload(target_name, about_base_url)?;
        let mut entry_payload = fields;
        entry_payload.add_units(72)?;
        let entry = row(
            changed(context, entry_payload, fields.bytes, fields.bytes, || {
                Ok(Entry {
                    id: creation.entry,
                    document: Some(creation.document),
                    initial_document: creation.document,
                    origin,
                    target_name: target_name.cloned(),
                    about_base_url: about_base_url.clone(),
                    initiator_origin: initiator_origin.cloned(),
                    initialization,
                    step: parent.is_none().then_some(0),
                    nested,
                    history: Some(if parent.is_none() { (1, 0) } else { (0, 0) }),
                })
            })?,
            context,
        )?;
        Self::form(old, Some(nav), Some(entry), context)
    }
}
impl Graph {
    fn nav(
        &self,
        id: DomHostedNavigableId,
        context: &Context<'_>,
    ) -> Result<&Navigable, DomHostedError> {
        self.navigable_with(id, || context.step())
    }
    fn ent(&self, id: DomHostedEntryId, context: &Context<'_>) -> Result<&Entry, DomHostedError> {
        self.entry_with(id, || context.step())
    }
    fn nav_copy<'a, 'b>(
        &self,
        id: DomHostedNavigableId,
        context: &'a Context<'b>,
    ) -> Result<Changed<'a, 'b, Navigable>, DomHostedError> {
        let old = self.nav(id, context)?;
        changed(
            context,
            source::storage::Payload {
                bytes: old.entries.bytes()?,
                units: 0,
            },
            0,
            old.entries.bytes()?,
            || Ok(old.clone()),
        )
    }

    /// This transition is called at the actual accepted traversal C handoff.
    pub(crate) fn mark_creation_queued(
        &self,
        child: DomHostedNavigableId,
        operation: DomHostedOperationId,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        let _call = context.call();
        let mut nav = self.nav_copy(child, context)?;
        context.step()?;
        let entry = self.ent(nav.active, context)?;
        if nav.history_attached
            || nav.creation_operation.is_some()
            || nav.current != nav.active
            || entry.step.is_some()
            || nav.id == nav.traversable
        {
            return Err(DomHostedError::WrongPhase);
        }
        nav.creation_operation = Some(operation);
        Self::form(Some(self), Some(row(nav, context)?), None, context)
    }

    /// The original lookup searches only history collections reachable from T.
    /// Detached retained collections never enter this search or used-steps.
    fn attached_entries(
        &self,
        traversable: DomHostedNavigableId,
        wanted: DomHostedNavigableId,
        context: &Context<'_>,
    ) -> Result<Option<&Values<DomHostedEntryId>>, DomHostedError> {
        context.step()?;
        let top = self.nav(traversable, context)?;
        if top.traversable != traversable || top.parent.is_some() {
            return Err(DomHostedError::WrongPhase);
        }
        let mut pending = Buffer::new(1, context)?;
        pending.push(traversable)?;
        let mut cursor = 0;
        while cursor < pending.values.len() {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            let id = pending.values[cursor];
            let nav = self.nav(id, context)?;
            if id == wanted {
                return Ok(Some(&nav.entries));
            }
            for entry in &nav.entries.values {
                #[cfg(test)]
                semantic_item();
                context.step()?;
                for child in &self.ent(*entry, context)?.nested.values {
                    #[cfg(test)]
                    semantic_item();
                    context.step()?;
                    if !pending.contains(child)? {
                        pending.push(*child)?;
                    }
                }
            }
            cursor += 1;
        }
        Ok(None)
    }

    /// Removal of H or an ancestor H is the only retained-collection fallback.
    fn removed_ancestry(
        &self,
        mut id: DomHostedNavigableId,
        traversable: DomHostedNavigableId,
        context: &Context<'_>,
    ) -> Result<bool, DomHostedError> {
        let mut seen = Buffer::new(0, context)?;
        let mut removed = false;
        loop {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            if seen.contains(&id)? {
                return Err(DomHostedError::WrongPhase);
            }
            seen.push(id)?;
            let nav = self.nav(id, context)?;
            if nav.traversable != traversable {
                return Err(DomHostedError::WrongPhase);
            }
            if id == traversable {
                return Ok(removed);
            }
            id = match (nav.parent, nav.former_parent) {
                (Some(parent), None) => parent,
                (None, Some(parent)) => {
                    removed = true;
                    parent
                }
                _ => return Err(DomHostedError::MissingPublication),
            };
        }
    }

    /// Execute C's actual synchronous graph changes, including C after removal.
    pub(crate) fn append_creation(
        &self,
        child: DomHostedNavigableId,
        operation: DomHostedOperationId,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        let _call = context.call();
        let mut nav = self.nav_copy(child, context)?;
        if nav.history_attached
            || nav.creation_operation != Some(operation)
            || nav.current != nav.active
        {
            return Err(DomHostedError::WrongPhase);
        }
        let parent_id = nav
            .parent
            .or(nav.former_parent)
            .ok_or(DomHostedError::WrongPhase)?;
        context.step()?;
        let parent = self.nav(parent_id, context)?;
        let parent_entry_id = parent.active;
        context.step()?;
        let parent_state = self.ent(parent_entry_id, context)?;
        let collection = match self.attached_entries(nav.traversable, parent_id, context)? {
            Some(collection) => collection,
            None if parent.history_attached
                && self.removed_ancestry(parent_id, nav.traversable, context)? =>
            {
                &parent.entries
            }
            None => return Err(DomHostedError::MissingPublication),
        };
        let mut step = None;
        for id in &collection.values {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            let entry = self.ent(*id, context)?;
            // This initial-entry profile has exactly one real DocumentState
            // per Entry; entry identity, rather than nullable Document, binds it.
            if entry.id == parent_state.id {
                step = entry.step;
                break;
            }
        }
        let step = step.ok_or(DomHostedError::MissingPublication)?;
        let mut child_entry = self.ent(nav.active, context)?.retained_copy(context)?;
        if child_entry.step.is_some() {
            return Err(DomHostedError::WrongPhase);
        }
        let mut nested = Buffer::new(
            parent_state
                .nested
                .values
                .len()
                .checked_add(1)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            context,
        )?;
        for id in &parent_state.nested.values {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            if *id == child {
                return Err(DomHostedError::WrongPhase);
            }
            nested.values.push(*id);
            #[cfg(test)]
            copied_items::<DomHostedNavigableId>(1);
        }
        nested.push(child)?;
        let mut parent_state = parent_state.retained_copy(context)?;
        let old_nested = parent_state.nested.bytes()?;
        parent_state.nested = nested.finish();
        parent_state.credit.release(source::storage::Payload {
            bytes: old_nested,
            units: 0,
        });
        parent_state.scratch.release(old_nested);
        child_entry.step = Some(step);
        nav.history_attached = true;
        // Consumed C is no longer available as a pending-initial proof.
        nav.creation_operation = None;
        let assigned = Self::form(
            Some(self),
            Some(row(nav, context)?),
            Some(row(child_entry, context)?),
            context,
        )?;
        let result = Self::form(
            Some(&assigned),
            None,
            Some(row(parent_state, context)?),
            context,
        )?;
        let extent = assigned.discarded_payload(&result)?;
        drop(assigned);
        context.release_output(extent);
        Ok(result)
    }

    /// Caller has already cleared container.contentNavigable in source order.
    /// `parent` is the actual current container node-Document's navigable.
    pub(crate) fn unlink_child(
        &self,
        child: DomHostedNavigableId,
        parent: DomHostedNavigableId,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        let _call = context.call();
        let mut nav = self.nav_copy(child, context)?;
        if nav.parent != Some(parent) || nav.former_parent.is_some() {
            return Err(DomHostedError::WrongPhase);
        }
        context.step()?;
        let parent = self.nav(parent, context)?;
        context.step()?;
        let mut parent_state = self.ent(parent.active, context)?.retained_copy(context)?;
        let mut nested = Buffer::new(parent_state.nested.values.len(), context)?;
        for id in &parent_state.nested.values {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            if *id != child {
                nested.values.push(*id);
                #[cfg(test)]
                copied_items::<DomHostedNavigableId>(1);
            }
        }
        let old_nested = parent_state.nested.bytes()?;
        parent_state.nested = nested.finish();
        parent_state.credit.release(source::storage::Payload {
            bytes: old_nested,
            units: 0,
        });
        parent_state.scratch.release(old_nested);
        nav.former_parent = nav.parent.take();
        Self::form(
            Some(self),
            Some(row(nav, context)?),
            Some(row(parent_state, context)?),
            context,
        )
    }

    fn target_entry(
        &self,
        nav: &Navigable,
        target: u64,
        context: &Context<'_>,
    ) -> Result<DomHostedEntryId, DomHostedError> {
        context.step()?;
        let active = self.ent(nav.active, context)?;
        // The selected bookkeeping-only exception requires actual queued C.
        if !nav.history_attached
            && nav.creation_operation.is_some()
            && nav.current == nav.active
            && active.step.is_none()
        {
            return Ok(active.id);
        }
        let entries = self
            .attached_entries(nav.traversable, nav.id, context)?
            .ok_or(DomHostedError::MissingPublication)?;
        let mut best = None;
        for id in &entries.values {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            if let Some(step) = self.ent(*id, context)?.step
                && step <= target
                && best.is_none_or(|(old, _)| step > old)
            {
                best = Some((step, *id));
            }
        }
        best.map(|(_, id)| id)
            .ok_or(DomHostedError::MissingPublication)
    }

    fn used_steps<'a, 'b>(
        &self,
        traversable: DomHostedNavigableId,
        context: &'a Context<'b>,
    ) -> Result<Buffer<'a, 'b, u64>, DomHostedError> {
        let mut pending = Buffer::new(1, context)?;
        let mut steps = Buffer::new(0, context)?;
        pending.push(traversable)?;
        let mut cursor = 0;
        while cursor < pending.values.len() {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            let nav = self.nav(pending.values[cursor], context)?;
            for id in &nav.entries.values {
                #[cfg(test)]
                semantic_item();
                context.step()?;
                let entry = self.ent(*id, context)?;
                let step = entry.step.ok_or(DomHostedError::MissingPublication)?;
                if !steps.contains(&step)? {
                    steps.push(step)?;
                    let mut index = steps.values.len() - 1;
                    while index > 0 {
                        #[cfg(test)]
                        semantic_item();
                        context.step()?;
                        if steps.values[index - 1] <= steps.values[index] {
                            break;
                        }
                        steps.values.swap(index - 1, index);
                        #[cfg(test)]
                        copied_items::<u64>(2);
                        index -= 1;
                    }
                }
                for child in &entry.nested.values {
                    #[cfg(test)]
                    semantic_item();
                    context.step()?;
                    if !pending.contains(child)? {
                        pending.push(*child)?;
                    }
                }
            }
            cursor += 1;
        }
        Ok(steps)
    }

    /// Capture obligations only. No Global task or History write has run yet.
    /// `children` lends actual source-ordered canonical live child relations.
    pub(crate) fn plan_update(
        &self,
        traversable: DomHostedNavigableId,
        context: &Context<'_>,
        mut children: impl FnMut(
            DomHostedNavigableId,
            &mut dyn FnMut(DomHostedNavigableId) -> Result<(), DomHostedError>,
        ) -> Result<(), DomHostedError>,
    ) -> Result<Update, DomHostedError> {
        let _call = context.call();
        context.step()?;
        let top = self.nav(traversable, context)?;
        if top.traversable != top.id {
            return Err(DomHostedError::WrongPhase);
        }
        let requested = top.current_step.ok_or(DomHostedError::MissingPublication)?;
        let steps = self.used_steps(traversable, context)?;
        let mut selected = None;
        for (index, step) in steps.values.iter().enumerate() {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            if *step <= requested {
                selected = Some((*step, index));
            }
        }
        let (target_step, index) = selected.ok_or(DomHostedError::MissingPublication)?;
        let mut pending = Buffer::new(1, context)?;
        let mut jobs = Buffer::new(0, context)?;
        pending.push(traversable)?;
        let mut cursor = 0;
        while cursor < pending.values.len() {
            #[cfg(test)]
            semantic_item();
            context.step()?;
            let nav = self.nav(pending.values[cursor], context)?;
            if nav.traversable != traversable {
                return Err(DomHostedError::WrongPhase);
            }
            let entry = self.target_entry(nav, target_step, context)?;
            // The selected initial-entry profile never reloads or replaces E.
            // A changing result is an explicit unsupported domain, not omitted.
            if entry != nav.current || nav.active != nav.current {
                return Err(DomHostedError::UnsupportedProfile);
            }
            context.step()?;
            let state = self.ent(entry, context)?;
            jobs.push(Job {
                navigable: nav.id,
                entry,
                document: state.initial_document,
                global: nav.global,
                length: steps.values.len(),
                index,
            })?;
            if state.document.is_some() {
                children(nav.id, &mut |child| {
                    context.step()?;
                    let record = self.nav(child, context)?;
                    if record.parent != Some(nav.id)
                        || record.traversable != traversable
                        || pending.contains(&child)?
                    {
                        return Err(DomHostedError::WrongPhase);
                    }
                    pending.push(child)
                })?;
            }
            cursor += 1;
        }
        Ok(Update {
            traversable,
            target_step,
            jobs: jobs.finish(),
        })
    }

    /// Called by the real original Global job; no write on stale/retired state.
    pub(crate) fn apply_history_job(
        &self,
        job: &Job,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        let _call = context.call();
        context.step()?;
        let nav = self.nav(job.navigable, context)?;
        if nav.active != job.entry || nav.current != job.entry || nav.global != job.global {
            return Err(DomHostedError::StaleBasis);
        }
        context.step()?;
        let mut entry = self.ent(job.entry, context)?.retained_copy(context)?;
        if entry.document != Some(job.document) || nav.retired {
            return Err(DomHostedError::StaleIdentity);
        }
        if job.index >= job.length {
            return Err(DomHostedError::WrongPhase);
        }
        entry.history = Some((job.length, job.index));
        Self::form(Some(self), None, Some(row(entry, context)?), context)
    }

    /// Caller reaches this only after every captured obligation has settled,
    /// keeping executed and exact-generation destruction-retired outcomes apart.
    pub(crate) fn finish_update(
        &self,
        update: &Update,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        let _call = context.call();
        let mut nav = self.nav_copy(update.traversable, context)?;
        if nav.traversable != nav.id {
            return Err(DomHostedError::WrongPhase);
        }
        nav.current_step = Some(update.target_step);
        Self::form(Some(self), Some(row(nav, context)?), None, context)
    }

    /// Entered by the actual recursive destruction owner before child capture.
    pub(crate) fn mark_retiring(
        &self,
        id: DomHostedNavigableId,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        let _call = context.call();
        let mut nav = self.nav_copy(id, context)?;
        nav.retiring = true;
        Self::form(Some(self), Some(row(nav, context)?), None, context)
    }

    /// Exact canonical cleanup evidence, checked before any Global/Window use.
    pub(crate) fn job_destroyed(
        &self,
        job: &Job,
        context: &Context<'_>,
    ) -> Result<bool, DomHostedError> {
        let nav = self.nav(job.navigable, context)?;
        let entry = self.ent(job.entry, context)?;
        Ok(nav.retired
            && nav.global == job.global
            && entry.initial_document == job.document
            && entry.document.is_none())
    }

    /// Only the real successful destroy-Document cleanup may invoke this.
    /// DocumentState becomes null; captured provenance and immutable history stay.
    pub(crate) fn clear_document(
        &self,
        document: DocumentId,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        let _call = context.call();
        let mut candidate = Self::form(Some(self), None, None, context)?;
        let Some(members) = self.documents.get(document) else {
            return Ok(candidate);
        };
        for (_, member) in members.rows.iter() {
            context.step()?;
            let entry = self
                .entries
                .get(member.entry)
                .ok_or(DomHostedError::StaleIdentity)?;
            if entry.document == Some(document) {
                #[cfg(test)]
                semantic_item();
                let mut next = entry.retained_copy(context)?;
                next.document = None;
                let prior = candidate;
                candidate = Self::form(Some(&prior), None, Some(row(next, context)?), context)?;
                let extent = prior.discarded_payload(&candidate)?;
                drop(prior);
                context.release_output(extent);
            }
        }
        for (_, member) in members.rows.iter() {
            context.step()?;
            let nav = self
                .navigables
                .get(member.navigable)
                .ok_or(DomHostedError::StaleIdentity)?;
            {
                context.step()?;
                if self.ent(nav.active, context)?.document == Some(document) {
                    #[cfg(test)]
                    semantic_item();
                    let mut next = changed(
                        context,
                        source::storage::Payload {
                            bytes: nav.entries.bytes()?,
                            units: 0,
                        },
                        0,
                        nav.entries.bytes()?,
                        || Ok(nav.value.clone()),
                    )?;
                    next.retired = true;
                    next.retiring = true;
                    let prior = candidate;
                    candidate = Self::form(Some(&prior), Some(row(next, context)?), None, context)?;
                    let extent = prior.discarded_payload(&candidate)?;
                    drop(prior);
                    context.release_output(extent);
                }
            }
        }
        if !members.rows.is_empty() {
            let prior = candidate;
            candidate = Self::form_documents(Some(&prior), None, None, Some(document), context)?;
            let extent = prior.discarded_payload(&candidate)?;
            drop(prior);
            context.release_output(extent);
        }
        Ok(candidate)
    }
}

impl DomSnapshot {
    pub fn hosted_navigable(
        &self,
        id: DomHostedNavigableId,
    ) -> Result<DomHostedNavigable<'_>, DomHostedError> {
        self.require_complete_source()?;
        let graph = self
            .state
            .hosted_history
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?;
        let record = graph.navigable(id)?;
        Ok(DomHostedNavigable {
            record,
            active: graph.entry(record.active)?,
        })
    }
    pub fn hosted_entry(&self, id: DomHostedEntryId) -> Result<DomHostedEntry<'_>, DomHostedError> {
        self.require_complete_source()?;
        let graph = self
            .state
            .hosted_history
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?;
        Ok(DomHostedEntry {
            record: graph.entry(id)?,
        })
    }
}

impl Dom {
    pub fn hosted_navigable(
        &self,
        id: DomHostedNavigableId,
    ) -> Result<DomHostedNavigable<'_>, DomHostedError> {
        self.require_sync_source_idle()?;
        let graph = self
            .state
            .hosted_history
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?;
        let record = graph.navigable(id)?;
        let document = graph
            .entry(record.active)?
            .document
            .ok_or(DomHostedError::StaleIdentity)?;
        let node = self.state.node(self.owner, document.node())?;
        let DomNodeKind::Document(metadata) = &node.kind else {
            return Err(DomError::WrongDocument.into());
        };
        metadata
            .document_environment()
            .hosted
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?
            .validate_current()?;
        if record.retired {
            return Err(DomHostedError::StaleIdentity);
        }
        Ok(DomHostedNavigable {
            record,
            active: graph.entry(record.active)?,
        })
    }
    pub fn hosted_entry(&self, id: DomHostedEntryId) -> Result<DomHostedEntry<'_>, DomHostedError> {
        self.require_sync_source_idle()?;
        let record = self
            .state
            .hosted_history
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?
            .entry(id)?;
        let document = record.document.ok_or(DomHostedError::StaleIdentity)?;
        let node = self.state.node(self.owner, document.node())?;
        let DomNodeKind::Document(metadata) = &node.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let environment = metadata.document_environment();
        let association = environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        association.validate_current()?;
        association
            .source
            .validate(id.source, Role::Entry, id.slot, id.generation)?;
        Ok(DomHostedEntry { record })
    }
}

#[cfg(test)]
pub(in crate::store) mod tests {
    use crate::DomSourceLimits;
    // Real issuer IDs and native Document identity payloads exercise this data
    // owner. They do not attest hosted Globals, entropy, loops or external jobs.
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn history_outer_bank_fixed_row_work_is_bounded() {
        fn measure(unrelated: usize) -> u64 {
            let cancel = AtomicBool::new(false);
            let mut fixture = Fixture::new();
            let (top, _, work) = fixture.issue(0);
            let mut graph = {
                let context = Context::new(&fixture.source, &work, &cancel, 0);
                Graph::initial(None, fixture.input(top, None), &context).unwrap()
            };
            drop(work);
            for _ in 0..unrelated {
                let (creation, _, work) = fixture.issue(2);
                let context = Context::new(&fixture.source, &work, &cancel, 0);
                graph =
                    Graph::initial(Some(&graph), fixture.input(creation, None), &context).unwrap();
            }
            let (child, operation, work) = fixture.issue(1);
            let context = Context::new(&fixture.source, &work, &cancel, 0);
            graph =
                Graph::initial(Some(&graph), fixture.input(child, Some(top)), &context).unwrap();
            let before = work.spent.load(Ordering::Relaxed);
            let changed = graph
                .mark_creation_queued(child.navigable, operation, &context)
                .unwrap();
            let steps = work.spent.load(Ordering::Relaxed) - before;
            assert_eq!(
                graph.navigable(child.navigable).unwrap().creation_operation,
                None
            );
            assert_eq!(
                changed
                    .navigable(child.navigable)
                    .unwrap()
                    .creation_operation,
                Some(operation)
            );
            steps
        }
        let small_steps = measure(0);
        let large_steps = measure(48);
        assert!(
            large_steps <= small_steps + 64,
            "one history row change spent {large_steps} steps with 48 unrelated rows, versus {small_steps} without them"
        );
    }

    pub(in crate::store) struct Fixture {
        pub(in crate::store) dom: Dom,
        pub(in crate::store) source: DomHostedSource,
        host: DomHostedHostId,
        documents: [DocumentId; 5],
        origin: crate::DomOpaqueOrigin,
        target_name: DocumentString,
        about_base_url: HtmlAboutBaseUrl,
    }
    impl Fixture {
        pub(in crate::store) fn new() -> Self {
            let cancel = AtomicBool::new(false);
            let mut dom = Dom::new(
                DomLimits::new(
                    64,
                    4096,
                    64,
                    0,
                    0,
                    16,
                    4_000_000,
                    crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let mut batch = DomBatch::new();
            let tickets = std::array::from_fn::<_, 5, _>(|_| batch.create_document());
            let commit = dom.apply(batch, &cancel).unwrap();
            let documents = tickets.map(|ticket| commit.document(&ticket).unwrap());
            let origin = dom
                .snapshot()
                .document(documents[0])
                .unwrap()
                .metadata()
                .origin()
                .clone();
            let mut source = dom
                .register_hosted_source(
                    DomHostedLimits::new(
                        DomHostedRecordLimits::new(64, 64, 64, 64, 64, 64, 64),
                        16_000_000,
                        16_000_000,
                        10_000_000,
                    ),
                    &cancel,
                )
                .unwrap();
            let host = source.issue_host(&cancel).unwrap();
            Self {
                dom,
                source,
                host,
                documents,
                origin,
                target_name: DocumentString::from(""),
                about_base_url: HtmlAboutBaseUrl::KnownNull,
            }
        }
        pub(in crate::store) fn new_for_outer_rows(rows: usize) -> Self {
            let mut fixture = Self::new();
            let count = rows.checked_add(8).unwrap().max(64);
            let bytes = count
                .checked_mul(65_536)
                .unwrap()
                .checked_add(16_000_000)
                .unwrap();
            fixture
                .source
                .set_limits(
                    DomHostedLimits::new(
                        DomHostedRecordLimits::new(count, count, count, count, count, count, count),
                        bytes,
                        bytes,
                        u64::try_from(count)
                            .unwrap()
                            .checked_mul(4096)
                            .unwrap()
                            .checked_add(10_000_000)
                            .unwrap(),
                    ),
                    &AtomicBool::new(false),
                )
                .unwrap();
            fixture
        }
        pub(in crate::store) fn issue(
            &mut self,
            document: usize,
        ) -> (DomHostedCreation, DomHostedOperationId, Arc<Work>) {
            let cancel = AtomicBool::new(false);
            let prefix = self.source.prepare_top_level(self.host, &cancel).unwrap();
            let work = prefix.work;
            let operation = self.source.begin_operation(&work, 10, 0, &cancel).unwrap();
            let mut request = Request {
                key: RequestKey {
                    operation,
                    phase: 0,
                },
                host: prefix.host,
                context: prefix.context,
                work: work.clone(),
                parent: None,
            };
            advance(&self.source, &mut request);
            let inputs = crate::document::HostedDocumentInputs::form(
                &self.source,
                &work,
                &self.origin,
                &cancel,
            )
            .unwrap();
            let execution = DomHostedExecutionRequest {
                request,
                origin: self.origin.clone(),
                inputs,
            };
            let global = self.source.issue_global(&execution, &cancel).unwrap();
            let mut request = execution.request;
            for _ in 0..3 {
                advance(&self.source, &mut request);
            }
            let entry_request = DomHostedEntryRequest {
                request,
                document: self.documents[document],
                global,
            };
            let entry = self.source.issue_entry(&entry_request, &cancel).unwrap();
            let navigable = self
                .source
                .issue_navigable(&entry_request, &cancel)
                .unwrap();
            let mut request = entry_request.request;
            advance(&self.source, &mut request);
            self.source
                .permit(&request, self.source.revision().unwrap(), true, &cancel)
                .unwrap()
                .commit();
            (
                DomHostedCreation {
                    document: self.documents[document],
                    host: self.host,
                    context: request.context,
                    global,
                    entry,
                    navigable,
                },
                operation,
                work,
            )
        }
        pub(in crate::store) fn input(
            &self,
            creation: DomHostedCreation,
            parent: Option<DomHostedCreation>,
        ) -> Initial<'_> {
            let key: [u16; 36] = "00000000-0000-4000-8000-000000000000"
                .encode_utf16()
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            Initial {
                creation,
                initialization: DomHostedEntryInitialization::new(key, key),
                origin: self.origin.clone(),
                parent: parent.map(|p| (p.navigable, p.document.node())),
                target_name: parent.is_none().then_some(&self.target_name),
                about_base_url: &self.about_base_url,
                initiator_origin: parent.is_some().then_some(&self.origin),
            }
        }
        fn used(&self) -> usize {
            self.source.limits().retained_bytes() - self.source.remaining_bytes().unwrap()
        }
        fn limits(&mut self, retained: usize, scratch: usize, work: u64) {
            self.source
                .set_limits(
                    DomHostedLimits::new(self.source.limits().records(), retained, scratch, work),
                    &AtomicBool::new(false),
                )
                .unwrap();
        }
    }
    fn advance(source: &DomHostedSource, request: &mut Request) {
        source
            .permit(
                request,
                source.revision().unwrap(),
                false,
                &AtomicBool::new(false),
            )
            .unwrap()
            .commit();
        request.key.phase += 1;
    }
    type ChildVisitor<'a> = dyn FnMut(DomHostedNavigableId) -> Result<(), DomHostedError> + 'a;

    fn children<'a>(
        rows: &'a [(DomHostedNavigableId, &'a [DomHostedNavigableId])],
    ) -> impl FnMut(DomHostedNavigableId, &mut ChildVisitor<'_>) -> Result<(), DomHostedError> + 'a
    {
        move |parent, visit| {
            if let Some((_, children)) = rows.iter().find(|(id, _)| *id == parent) {
                for child in *children {
                    visit(*child)?;
                }
            }
            Ok(())
        }
    }

    fn historical_about_base() -> HtmlAboutBaseUrl {
        use surgeist_html_semantics::{
            HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme,
        };
        let mut units = Vec::with_capacity(256);
        units.extend("https://creator.example/base/".encode_utf16());
        HtmlAboutBaseUrl::KnownRecord(
            HtmlBaseUrlRecord::observed(
                HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 7, 8, 9),
                DocumentString::from_utf16(units),
                HtmlBaseUrlScheme::Other,
                false,
                None,
            )
            .unwrap(),
        )
    }

    #[test]
    fn buffer_counters_distinguish_actual_capacity_from_completed_element_writes() {
        if !crate::store::scale_metrics::ENABLED {
            return;
        }
        let cancel = AtomicBool::new(false);
        let mut fixture = Fixture::new();
        let (_, _, work) = fixture.issue(0);
        let context = Context::new(&fixture.source, &work, &cancel, 0);
        let _call = context.call();
        let measurement = crate::store::scale_metrics::Measurement::begin().unwrap();
        let mut buffer = Buffer::<u64>::new(3, &context).unwrap();
        let capacity = buffer.values.capacity();
        buffer.push(7).unwrap();
        buffer.push(11).unwrap();
        let values = buffer.finish();
        let counts = measurement
            .finish(crate::store::scale_metrics::Hooks::default())
            .unwrap();
        assert_eq!(values.values, [7, 11]);
        assert!(capacity >= 3);
        assert_eq!(counts.structural_allocations, 2);
        assert_eq!(
            counts.structural_bytes,
            capacity * size_of::<u64>() + size_of::<Values<u64>>()
        );
        assert_eq!(counts.sequence_materializations, 2);
        assert_eq!(counts.sequence_materialized_bytes, 2 * size_of::<u64>());
        assert_eq!(counts.hosted_semantic_visits, 0);
        let measurement = crate::store::scale_metrics::Measurement::begin().unwrap();
        let alias = values.clone();
        let counts = measurement
            .finish(crate::store::scale_metrics::Hooks::default())
            .unwrap();
        assert!(Arc::ptr_eq(&values, &alias));
        assert_eq!(counts.structural_allocations, 0);
        assert_eq!(counts.sequence_materializations, 0);
    }

    #[test]
    fn historical_snapshot_foreign_issuer_lookups_remain_stale_identity() {
        let cancel = AtomicBool::new(false);
        let mut fixture = Fixture::new();
        let (top, _, work) = fixture.issue(0);
        let context = Context::new(&fixture.source, &work, &cancel, 0);
        let graph = Graph::initial(None, fixture.input(top, None), &context).unwrap();
        let mut foreign = Fixture::new();
        let (other, _, _work) = foreign.issue(0);
        let mut snapshot = fixture.dom.snapshot();
        // Only the external immutable snapshot retains State now. Installing
        // the independently issued history is a private observation fixture;
        // no current DOM/hosted execution or controlled budget is claimed.
        drop(fixture.dom);
        Arc::get_mut(&mut snapshot.state).unwrap().hosted_history = Some(graph);
        assert_eq!(
            snapshot.hosted_entry(top.entry).unwrap().document(),
            Some(top.document)
        );
        assert!(matches!(
            snapshot.hosted_navigable(other.navigable),
            Err(DomHostedError::StaleIdentity)
        ));
        assert!(matches!(
            snapshot.hosted_entry(other.entry),
            Err(DomHostedError::StaleIdentity)
        ));
    }

    #[test]
    fn creation_scratch_admits_actual_new_candidate_union_until_transfer() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (child, op, work) = f.issue(1);
        let queued = {
            let context = Context::new(&f.source, &work, &cancel, 0);
            let root = Graph::initial(None, f.input(top, None), &context).unwrap();
            let initial = Graph::initial(Some(&root), f.input(child, Some(top)), &context).unwrap();
            let queued = initial
                .mark_creation_queued(child.navigable, op, &context)
                .unwrap();
            assert_eq!(context.scratch.get(), 0);
            queued
        };
        // This fixture has two N/E rows, one one-entry collection per N,
        // empty nested collections and only the top's present-empty name.
        // Count actual represented capacities and declared owner headers here,
        // independently of the implementation's scratch/peak observations.
        let extent = |graph: &Graph| {
            assert_eq!(graph.navigables.len(), 2);
            assert_eq!(graph.entries.len(), 2);
            graph
                .inspect_materializations()
                .unwrap()
                .iter()
                .map(|(_, usage)| usage.bytes())
                .sum::<usize>()
        };
        let assigned_extent = extent(&queued);
        assert_eq!(queued.payload(|| Ok(())).unwrap().bytes, assigned_extent);
        let (output_extent, fresh_output, replaced_entry_page) = {
            let context = Context::new(&f.source, &work, &cancel, 0);
            let output = queued
                .append_creation(child.navigable, op, &context)
                .unwrap();
            assert_eq!(context.scratch.get(), 0);
            let bytes = extent(&output);
            assert_eq!(output.payload(|| Ok(())).unwrap().bytes, bytes);
            let original = queued
                .inspect_materializations()
                .unwrap()
                .into_iter()
                .map(|(id, _)| id)
                .collect::<std::collections::BTreeSet<_>>();
            let fresh = output
                .inspect_materializations()
                .unwrap()
                .iter()
                .filter(|(id, _)| !original.contains(id))
                .map(|(_, usage)| usage.bytes())
                .sum::<usize>();
            let entry_pages = output
                .entries
                .inspect_graph(|_| Ok(bank::Totals::default()))
                .unwrap()
                .index_charge();
            assert_eq!(entry_pages.pages, 1);
            (bytes, fresh, entry_pages.bytes)
        };
        // The second entry edit replaces one actual leaf while the assigned
        // Graph is still alive. Other new records and the navigable leaf are
        // shared by the two candidates; input rows and identity indexes retain
        // their original issuer receipts. Count those physical occurrences,
        // plus the intermediate header/leaf, independently of scratch counters.
        let caller_buffer = vec![0u8; 17];
        let caller_bytes = size_of::<Vec<u8>>() + caller_buffer.capacity();
        let exact_peak = caller_bytes + fresh_output + size_of::<Graph>() + replaced_entry_page;
        let limits = f.source.limits();
        let used = f.used();
        let before_work = work.spent.load(Ordering::Relaxed);
        f.limits(limits.retained_bytes(), exact_peak - 1, limits.work());
        {
            let context = Context::new(&f.source, &work, &cancel, caller_bytes);
            let actual = queued.append_creation(child.navigable, op, &context);
            assert!(
                matches!(actual,
                Err(DomHostedError::Limit {resource:DomHostedResource::ScratchBytes,limit})
                    if limit == exact_peak - 1),
                "actual: {actual:?}"
            );
            assert_eq!(context.scratch.get(), caller_bytes);
        }
        assert_eq!(f.used(), used);
        assert!(work.spent.load(Ordering::Relaxed) > before_work);
        assert_eq!(queued.entry(child.entry).unwrap().step, None);
        assert!(queued.entry(top.entry).unwrap().nested.values.is_empty());
        f.limits(limits.retained_bytes(), exact_peak, limits.work());
        let context = Context::new(&f.source, &work, &cancel, caller_bytes);
        for _ in 0..2 {
            let output = queued
                .append_creation(child.navigable, op, &context)
                .unwrap();
            assert_eq!(context.scratch.get(), caller_bytes);
            assert_eq!(output.entry(child.entry).unwrap().step, Some(0));
            assert_eq!(
                output.entry(top.entry).unwrap().nested.values,
                [child.navigable]
            );
            assert_eq!(extent(&output), output_extent);
            drop(output);
            assert_eq!(f.used(), used);
        }
        assert_eq!(caller_buffer, [0; 17]);
    }

    #[test]
    fn entry_preserves_sampled_null_empty_and_exact_names_and_original_document_state() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (absent, _, work) = f.issue(1);
        drop(work);
        let (empty, _, work) = f.issue(2);
        drop(work);
        let (named, op, work) = f.issue(3);
        let context = Context::new(&f.source, &work, &cancel, 0);
        let mut graph = Graph::initial(None, f.input(top, None), &context).unwrap();
        let empty_name = DocumentString::from("");
        let mut units = Vec::with_capacity(128);
        units.extend([0xd800, 0, u16::from(b'N')]);
        let mut current_name = DocumentString::from_utf16(units);
        let mut current_about = historical_about_base();
        let sampled_name = current_name.clone();
        let sampled_about = current_about.clone();
        for (child, name) in [
            (absent, None),
            (empty, Some(&empty_name)),
            (named, Some(&current_name)),
        ] {
            let mut input = f.input(child, Some(top));
            input.target_name = name;
            input.about_base_url = &current_about;
            graph = Graph::initial(Some(&graph), input, &context).unwrap();
        }
        let captured = graph.clone();
        // The raw child algorithm distinguishes missing name from present-empty;
        // the no-opener top profile has an actual empty-string operand.
        let top_entry = graph.entry(top.entry).unwrap();
        assert_eq!(top_entry.target_name.as_ref(), Some(&empty_name));
        assert_eq!(top_entry.about_base_url, HtmlAboutBaseUrl::KnownNull);
        assert_eq!(top_entry.initiator_origin, None);
        assert_eq!(graph.entry(absent.entry).unwrap().target_name, None);
        assert_eq!(
            graph.entry(empty.entry).unwrap().target_name.as_ref(),
            Some(&empty_name)
        );
        let entry = graph.entry(named.entry).unwrap();
        assert_eq!(
            entry.target_name.as_ref().unwrap().code_units(),
            &[0xd800, 0, 78]
        );
        assert!(Arc::ptr_eq(
            entry.target_name.as_ref().unwrap().shared_code_units(),
            sampled_name.shared_code_units()
        ));
        assert_eq!(entry.about_base_url, sampled_about);
        assert_eq!(entry.initiator_origin.as_ref(), Some(&entry.origin));
        // Later embedder attributes/base inputs cannot replace the sampled
        // DocumentState, including while the initial entry awaits and executes C.
        current_name = DocumentString::from("later");
        current_about = historical_about_base();
        assert_ne!(entry.target_name.as_ref(), Some(&current_name));
        assert_ne!(entry.about_base_url, current_about);
        graph = graph
            .mark_creation_queued(named.navigable, op, &context)
            .unwrap();
        graph = graph
            .append_creation(named.navigable, op, &context)
            .unwrap();
        for owner in [&captured, &graph] {
            let entry = owner.entry(named.entry).unwrap();
            assert_eq!(entry.target_name.as_ref(), Some(&sampled_name));
            assert_eq!(entry.about_base_url, sampled_about);
            assert_eq!(entry.initiator_origin.as_ref(), Some(&f.origin));
        }
    }

    #[test]
    fn entry_copy_admits_actual_name_and_about_backing_before_retaining_them() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (child, op, work) = f.issue(1);
        let mut units = Vec::with_capacity(128);
        units.push(u16::from(b'n'));
        let name = DocumentString::from_utf16(units);
        let about = historical_about_base();
        let graph = {
            let context = Context::new(&f.source, &work, &cancel, 0);
            let root = Graph::initial(None, f.input(top, None), &context).unwrap();
            let mut input = f.input(child, Some(top));
            input.target_name = Some(&name);
            input.about_base_url = &about;
            Graph::initial(Some(&root), input, &context).unwrap()
        };
        let entry = graph.entry(child.entry).unwrap();
        let HtmlAboutBaseUrl::KnownRecord(record) = &about else {
            unreachable!()
        };
        let about_usage = record.retained_storage_usage();
        let fields_bytes = size_of::<Vec<u16>>()
            + 2 * name.shared_code_units().capacity()
            + about_usage.retained_bytes();
        let fields_units = name.shared_code_units().capacity() + about_usage.utf16_extent();
        assert!(fields_units > name.code_units().len() + record.serialization().code_units().len());
        let retained_copy_bytes = size_of::<Row<Entry>>() + fields_bytes;
        let used = f.used();
        let limits = f.source.limits();
        let one_short = used + retained_copy_bytes - 1;
        f.limits(one_short, limits.scratch_bytes(), limits.work());
        let context = Context::new(&f.source, &work, &cancel, 0);
        assert!(matches!(entry.retained_copy(&context),
            Err(DomHostedError::Limit {resource:DomHostedResource::RetainedBytes,limit}) if limit==one_short));
        assert_eq!(f.used(), used);
        f.limits(
            limits.retained_bytes(),
            limits.scratch_bytes(),
            limits.work(),
        );

        let mut draft = Draft::new_hosted_source(
            &f.dom,
            DomBatch::new().owner,
            DomSourceLimits::new(16_000_000, 4096, 10_000_000, 0, 0, 0, 1),
            work.spent.clone(),
            work.spent_id,
            op,
            &cancel,
        )
        .unwrap();
        let base_units = draft.source_storage_units(&cancel).unwrap() + draft.source_other_units;
        let actual_units = 72 + fields_units;
        let short_units = base_units + actual_units - 1;
        draft.source_limits = Some(DomSourceLimits::new(
            16_000_000,
            short_units,
            10_000_000,
            0,
            0,
            0,
            1,
        ));
        let before_work = work.spent.load(Ordering::Relaxed);
        {
            let fresh = source::formation::Formation::new(&draft, &cancel)
                .unwrap()
                .unwrap();
            let context = Context::new(&f.source, &work, &cancel, 0).with_formation(&fresh);
            assert!(matches!(entry.retained_copy(&context),
                Err(DomHostedError::Dom(DomError::ResourceLimit {resource:crate::DomResource::SourceRetainedUnits,limit})) if limit==short_units));
            assert_eq!(fresh.owned_payload().bytes, 0);
            assert_eq!(fresh.owned_payload().units, 0);
        }
        assert_eq!(f.used(), used);
        assert!(work.spent.load(Ordering::Relaxed) > before_work);
        draft.source_limits = Some(DomSourceLimits::new(
            16_000_000,
            base_units + actual_units,
            10_000_000,
            0,
            0,
            0,
            1,
        ));
        let fresh = source::formation::Formation::new(&draft, &cancel)
            .unwrap()
            .unwrap();
        let context = Context::new(&f.source, &work, &cancel, 0).with_formation(&fresh);
        let copy = entry.retained_copy(&context).unwrap();
        assert_eq!(f.used(), used + retained_copy_bytes);
        assert_eq!(fresh.owned_payload().units, actual_units);
        assert_eq!(
            fresh.owned_payload().bytes,
            retained_copy_bytes + entry.nested.bytes().unwrap()
        );
        assert_eq!(copy.target_name.as_ref(), Some(&name));
        assert_eq!(copy.about_base_url, about);
        drop(copy);
        assert_eq!(f.used(), used);
        assert_eq!(fresh.owned_payload().bytes, 0);
        assert_eq!(fresh.owned_payload().units, 0);
    }

    #[test]
    fn queued_initial_siblings_and_nested_child_get_real_jobs_without_early_steps() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (a, a_op, work) = f.issue(1);
        drop(work);
        let (b, b_op, work) = f.issue(2);
        drop(work);
        let (nested, n_op, work) = f.issue(3);
        let context = Context::new(&f.source, &work, &cancel, 0);
        let mut graph = Graph::initial(None, f.input(top, None), &context).unwrap();
        for (creation, parent, op) in [(a, top, a_op), (b, top, b_op), (nested, a, n_op)] {
            graph =
                Graph::initial(Some(&graph), f.input(creation, Some(parent)), &context).unwrap();
            graph = graph
                .mark_creation_queued(creation.navigable, op, &context)
                .unwrap();
        }
        let original = graph.clone();
        let root_children = [b.navigable, a.navigable];
        let nested_children = [nested.navigable];
        let rows = [
            (top.navigable, root_children.as_slice()),
            (a.navigable, nested_children.as_slice()),
        ];
        let update = graph
            .plan_update(top.navigable, &context, children(&rows))
            .unwrap();
        assert_eq!(update.target_step, 0);
        assert_eq!(
            update
                .jobs
                .values
                .iter()
                .map(|job| job.navigable)
                .collect::<Vec<_>>(),
            [top.navigable, b.navigable, a.navigable, nested.navigable]
        );
        for creation in [a, b, nested] {
            assert_eq!(graph.entry(creation.entry).unwrap().step, None);
            assert_eq!(graph.entry(creation.entry).unwrap().history, Some((0, 0)));
        }
        assert!(graph.entry(top.entry).unwrap().nested.values.is_empty());
        for job in &update.jobs.values {
            graph = graph.apply_history_job(job, &context).unwrap();
        }
        graph = graph.finish_update(&update, &context).unwrap();
        for creation in [a, b, nested] {
            assert_eq!(graph.entry(creation.entry).unwrap().history, Some((1, 0)));
            assert_eq!(graph.entry(creation.entry).unwrap().step, None);
            assert_eq!(
                original.entry(creation.entry).unwrap().history,
                Some((0, 0))
            );
        }
        graph = graph.append_creation(a.navigable, a_op, &context).unwrap();
        graph = graph.append_creation(b.navigable, b_op, &context).unwrap();
        graph = graph
            .append_creation(nested.navigable, n_op, &context)
            .unwrap();
        assert_eq!(
            graph.entry(top.entry).unwrap().nested.values,
            [a.navigable, b.navigable]
        );
        assert_eq!(
            graph.entry(a.entry).unwrap().nested.values,
            [nested.navigable]
        );
        for creation in [a, b, nested] {
            assert_eq!(graph.entry(creation.entry).unwrap().step, Some(0));
        }
        assert!(matches!(
            graph.append_creation(a.navigable, a_op, &context),
            Err(DomHostedError::WrongPhase)
        ));
    }

    #[test]
    fn removal_before_creation_keeps_the_later_literal_history_append_detached() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (child, op, work) = f.issue(1);
        let context = Context::new(&f.source, &work, &cancel, 0);
        let top_graph = Graph::initial(None, f.input(top, None), &context).unwrap();
        let initial =
            Graph::initial(Some(&top_graph), f.input(child, Some(top)), &context).unwrap();
        let collection = initial.navigable(child.navigable).unwrap().entries.clone();
        let queued = initial
            .mark_creation_queued(child.navigable, op, &context)
            .unwrap();
        let removed = queued
            .unlink_child(child.navigable, top.navigable, &context)
            .unwrap();
        assert_eq!(removed.navigable(child.navigable).unwrap().parent, None);
        assert_eq!(
            removed.navigable(child.navigable).unwrap().former_parent,
            Some(top.navigable)
        );
        assert!(removed.entry(top.entry).unwrap().nested.values.is_empty());
        let appended = removed
            .append_creation(child.navigable, op, &context)
            .unwrap();
        assert_eq!(
            appended.entry(top.entry).unwrap().nested.values,
            [child.navigable]
        );
        assert_eq!(appended.entry(child.entry).unwrap().step, Some(0));
        assert!(Arc::ptr_eq(
            &collection,
            &appended.navigable(child.navigable).unwrap().entries
        ));
        // H is retained; live enumeration is empty after the source unlink.
        let update = appended
            .plan_update(top.navigable, &context, children(&[]))
            .unwrap();
        assert_eq!(update.jobs.values.len(), 1);
        assert_eq!(update.jobs.values[0].navigable, top.navigable);
        assert_eq!(appended.entry(child.entry).unwrap().history, Some((0, 0)));
        assert!(removed.entry(top.entry).unwrap().nested.values.is_empty());
    }

    #[test]
    fn removed_established_parent_retains_its_same_collection_for_nested_creation() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (parent, p_op, work) = f.issue(1);
        drop(work);
        let (child, c_op, work) = f.issue(2);
        let context = Context::new(&f.source, &work, &cancel, 0);
        let mut graph = Graph::initial(None, f.input(top, None), &context).unwrap();
        graph = Graph::initial(Some(&graph), f.input(parent, Some(top)), &context).unwrap();
        graph = graph
            .mark_creation_queued(parent.navigable, p_op, &context)
            .unwrap();
        graph = Graph::initial(Some(&graph), f.input(child, Some(parent)), &context).unwrap();
        graph = graph
            .mark_creation_queued(child.navigable, c_op, &context)
            .unwrap();
        // C_child cannot invent the parent's not-yet-attached collection.
        assert!(matches!(
            graph.append_creation(child.navigable, c_op, &context),
            Err(DomHostedError::MissingPublication)
        ));
        graph = graph
            .append_creation(parent.navigable, p_op, &context)
            .unwrap();
        let collection = graph.navigable(parent.navigable).unwrap().entries.clone();
        graph = graph
            .unlink_child(parent.navigable, top.navigable, &context)
            .unwrap();
        graph = graph
            .append_creation(child.navigable, c_op, &context)
            .unwrap();
        assert!(Arc::ptr_eq(
            &collection,
            &graph.navigable(parent.navigable).unwrap().entries
        ));
        assert_eq!(
            graph.entry(parent.entry).unwrap().nested.values,
            [child.navigable]
        );
        assert_eq!(graph.entry(child.entry).unwrap().step, Some(0));
        assert!(graph.entry(top.entry).unwrap().nested.values.is_empty());
        assert!(
            graph
                .attached_entries(top.navigable, parent.navigable, &context)
                .unwrap()
                .is_none()
        );
        assert!(
            graph
                .attached_entries(top.navigable, child.navigable, &context)
                .unwrap()
                .is_none()
        );
        let update = graph
            .plan_update(top.navigable, &context, children(&[]))
            .unwrap();
        assert_eq!(update.jobs.values.len(), 1);
    }

    #[test]
    fn document_state_clear_retains_managed_document_and_captured_job_provenance() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (child, op, work) = f.issue(1);
        let context = Context::new(&f.source, &work, &cancel, 0);
        let mut graph = Graph::initial(None, f.input(top, None), &context).unwrap();
        graph = Graph::initial(Some(&graph), f.input(child, Some(top)), &context).unwrap();
        graph = graph
            .mark_creation_queued(child.navigable, op, &context)
            .unwrap();
        graph = graph
            .append_creation(child.navigable, op, &context)
            .unwrap();
        let child_ids = [child.navigable];
        let rows = [(top.navigable, child_ids.as_slice())];
        let update = graph
            .plan_update(top.navigable, &context, children(&rows))
            .unwrap();
        let job = update.jobs.values[1];
        assert!(!graph.job_destroyed(&job, &context).unwrap());
        // The data-owner transition clears only DocumentState. Actual hosted
        // teardown must keep the managed Document/tree and establish completion
        // through its own canonical destructor before calling this transition.
        let canonical = f.dom.snapshot();
        assert!(canonical.document(child.document).is_ok());
        let destroyed = graph.clear_document(child.document, &context).unwrap();
        assert!(f.dom.snapshot().document(child.document).is_ok());
        assert_eq!(f.dom.snapshot().stamp(), canonical.stamp());
        assert_eq!(
            graph.entry(child.entry).unwrap().document,
            Some(child.document)
        );
        assert_eq!(destroyed.entry(child.entry).unwrap().document, None);
        assert_eq!(
            destroyed.entry(child.entry).unwrap().initial_document,
            child.document
        );
        assert_eq!(destroyed.entry(child.entry).unwrap().history, Some((0, 0)));
        assert!(destroyed.job_destroyed(&job, &context).unwrap());
        assert!(matches!(
            destroyed.apply_history_job(&job, &context),
            Err(DomHostedError::StaleIdentity)
        ));
        assert_eq!(job.document, child.document);
        assert_eq!(job.global, child.global);
    }

    #[test]
    fn reused_issuer_slots_preserve_old_full_generation_history_and_destruction_evidence() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (old, op, work) = f.issue(1);
        let context = Context::new(&f.source, &work, &cancel, 0);
        let mut graph = Graph::initial(None, f.input(top, None), &context).unwrap();
        graph = Graph::initial(Some(&graph), f.input(old, Some(top)), &context).unwrap();
        graph = graph
            .mark_creation_queued(old.navigable, op, &context)
            .unwrap();
        graph = graph
            .unlink_child(old.navigable, top.navigable, &context)
            .unwrap();
        // Literal C after removal leaves the old H in the real parent state.
        graph = graph.append_creation(old.navigable, op, &context).unwrap();
        let old_job = Job {
            navigable: old.navigable,
            entry: old.entry,
            document: old.document,
            global: old.global,
            length: 1,
            index: 0,
        };
        let canonical = f.dom.snapshot();
        assert!(canonical.document(old.document).is_ok());
        graph = graph.clear_document(old.document, &context).unwrap();
        assert!(f.dom.snapshot().document(old.document).is_ok());
        assert_eq!(f.dom.snapshot().stamp(), canonical.stamp());
        drop(work);
        for (role, slot, generation) in [
            (
                Role::Navigable,
                old.navigable.slot,
                old.navigable.generation,
            ),
            (Role::Entry, old.entry.slot, old.entry.generation),
            (Role::Global, old.global.slot, old.global.generation),
        ] {
            f.source
                .retire(role, old.navigable.source, slot, generation, &cancel)
                .unwrap();
        }
        let (new, op, work) = f.issue(2);
        assert_eq!(old.navigable.slot, new.navigable.slot);
        assert_eq!(old.entry.slot, new.entry.slot);
        assert_eq!(old.global.slot, new.global.slot);
        assert_ne!(old.navigable, new.navigable);
        let context = Context::new(&f.source, &work, &cancel, 0);
        let captured = graph.clone();
        graph = Graph::initial(Some(&graph), f.input(new, Some(top)), &context).unwrap();
        graph = graph
            .mark_creation_queued(new.navigable, op, &context)
            .unwrap();
        graph = graph.append_creation(new.navigable, op, &context).unwrap();
        assert_eq!(
            graph.entry(top.entry).unwrap().nested.values,
            [old.navigable, new.navigable]
        );
        assert_eq!(graph.entry(old.entry).unwrap().document, None);
        assert_eq!(graph.entry(new.entry).unwrap().document, Some(new.document));
        assert!(graph.job_destroyed(&old_job, &context).unwrap());
        let live = [new.navigable];
        let rows = [(top.navigable, live.as_slice())];
        let update = graph
            .plan_update(top.navigable, &context, children(&rows))
            .unwrap();
        assert_eq!(update.jobs.values.len(), 2);
        assert_eq!(update.jobs.values[1].navigable, new.navigable);
        assert!(
            !graph
                .job_destroyed(&update.jobs.values[1], &context)
                .unwrap()
        );
        assert!(matches!(
            captured.navigable(new.navigable),
            Err(DomHostedError::StaleIdentity)
        ));
        assert_eq!(captured.navigable(old.navigable).unwrap().id, old.navigable);
    }

    #[test]
    fn refused_history_candidate_releases_storage_without_refunding_original_work() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (child, op, work) = f.issue(1);
        let mut graph = {
            let context = Context::new(&f.source, &work, &cancel, 0);
            let top_graph = Graph::initial(None, f.input(top, None), &context).unwrap();
            let child_graph =
                Graph::initial(Some(&top_graph), f.input(child, Some(top)), &context).unwrap();
            child_graph
                .mark_creation_queued(child.navigable, op, &context)
                .unwrap()
        };
        let old = graph.clone();
        let limits = f.source.limits();
        let used = f.used();
        let spent = work.spent.load(Ordering::Relaxed);
        // The replacement row is admitted before its first retained-handle clone.
        let first_row = size_of::<Row<Navigable>>();
        let one_short = used + first_row - 1;
        f.limits(one_short, limits.scratch_bytes(), limits.work());
        let context = Context::new(&f.source, &work, &cancel, 0);
        assert!(
            matches!(graph.append_creation(child.navigable,op,&context),Err(DomHostedError::Limit {resource:DomHostedResource::RetainedBytes,limit}) if limit==one_short)
        );
        assert_eq!(context.scratch.get(), 0);
        assert_eq!(f.used(), used);
        assert!(work.spent.load(Ordering::Relaxed) > spent);
        assert!(Arc::ptr_eq(&old, &graph));
        assert_eq!(graph.entry(child.entry).unwrap().step, None);
        f.limits(
            limits.retained_bytes(),
            limits.scratch_bytes(),
            limits.work(),
        );
        let context = Context::new(&f.source, &work, &cancel, limits.scratch_bytes());
        assert!(matches!(
            graph.append_creation(child.navigable, op, &context),
            Err(DomHostedError::Limit {
                resource: DomHostedResource::ScratchBytes,
                ..
            })
        ));
        assert_eq!(context.scratch.get(), limits.scratch_bytes());
        assert_eq!(f.used(), used);
        cancel.store(true, Ordering::Relaxed);
        let before_cancel = work.spent.load(Ordering::Relaxed);
        let context = Context::new(&f.source, &work, &cancel, 0);
        assert!(matches!(
            graph.append_creation(child.navigable, op, &context),
            Err(DomHostedError::Dom(DomError::Cancelled))
        ));
        assert_eq!(context.scratch.get(), 0);
        assert_eq!(work.spent.load(Ordering::Relaxed), before_cancel);
        assert_eq!(f.used(), used);
        cancel.store(false, Ordering::Relaxed);
        // In this two-row graph, twenty reads admit both lookup and nested
        // candidate formation, then stop before publishing either changed row.
        let work_limit = work.spent.load(Ordering::Relaxed) + 20;
        f.limits(limits.retained_bytes(), limits.scratch_bytes(), work_limit);
        let context = Context::new(&f.source, &work, &cancel, 0);
        assert!(
            matches!(graph.append_creation(child.navigable,op,&context),Err(DomHostedError::WorkLimit {limit}) if limit==work_limit)
        );
        assert_eq!(context.scratch.get(), 0);
        assert_eq!(f.used(), used);
        assert_eq!(work.spent.load(Ordering::Relaxed), work_limit);
        assert_eq!(graph.entry(child.entry).unwrap().step, None);
        f.limits(
            limits.retained_bytes(),
            limits.scratch_bytes(),
            limits.work(),
        );
        let context = Context::new(&f.source, &work, &cancel, 0);
        graph = graph
            .append_creation(child.navigable, op, &context)
            .unwrap();
        assert_eq!(graph.entry(child.entry).unwrap().step, Some(0));
        assert_eq!(old.entry(child.entry).unwrap().step, None);
    }

    #[test]
    fn auxiliary_union_matches_raw_rows_and_releases_shared_fields_by_row_identity() {
        use crate::store::transaction::auxiliary;
        use std::collections::BTreeMap;
        let cancel = AtomicBool::new(false);
        let mut fixture = Fixture::new();
        let (top, _, work) = fixture.issue(0);
        let context = Context::new(&fixture.source, &work, &cancel, 0);
        let first = Graph::initial(None, fixture.input(top, None), &context).unwrap();
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), |_| Ok(())).unwrap();
        let first_control =
            auxiliary::adopt_history(&first, &registry, &mut |_| Ok(()), &mut || Ok(())).unwrap();
        assert_eq!(
            first_control
                .inspect_claimed_materializations()
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
            first
                .inspect_materializations()
                .unwrap()
                .into_iter()
                .collect()
        );
        drop(work);
        let (child, operation, work) = fixture.issue(1);
        let context = Context::new(&fixture.source, &work, &cancel, 0);
        let second =
            Graph::initial(Some(&first), fixture.input(child, Some(top)), &context).unwrap();
        let second_control =
            auxiliary::adopt_history(&second, &registry, &mut |_| Ok(()), &mut || Ok(())).unwrap();
        let queued = second
            .mark_creation_queued(child.navigable, operation, &context)
            .unwrap();
        let queued_control =
            auxiliary::adopt_history(&queued, &registry, &mut |_| Ok(()), &mut || Ok(())).unwrap();
        // A new Navigable row aliases the existing Values backing, but owns a
        // conservative full field occurrence under its new record identity.
        assert!(Arc::ptr_eq(
            &second.navigables.get(1).unwrap().entries,
            &queued.navigables.get(1).unwrap().entries
        ));
        assert!(!Arc::ptr_eq(
            second.navigables.get_shared(1).unwrap(),
            queued.navigables.get_shared(1).unwrap()
        ));
        let mut expected = BTreeMap::new();
        let mut actual = BTreeMap::new();
        for graph in [&first, &second, &queued] {
            for (id, usage) in graph.inspect_materializations().unwrap() {
                if let Some(old) = expected.insert(id, usage) {
                    assert_eq!(old, usage);
                }
            }
        }
        for control in [&first_control, &second_control, &queued_control] {
            for (id, usage) in control.inspect_claimed_materializations() {
                if let Some(old) = actual.insert(id, usage) {
                    assert_eq!(old, usage);
                }
            }
        }
        assert_eq!(actual, expected);
        drop(first_control);
        drop(second_control);
        registry.reconcile_releases();
        assert_eq!(
            queued_control
                .inspect_claimed_materializations()
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
            queued
                .inspect_materializations()
                .unwrap()
                .into_iter()
                .collect()
        );
        drop(queued_control);
        registry.reconcile_releases();
        assert_eq!(
            registry.total(),
            bank::CustodyRegistry::administration_usage()
        );
        // Semantic historical graphs remain usable after owner custody ends.
        assert_eq!(first.entry(top.entry).unwrap().step, Some(0));
        assert_eq!(queued.entry(child.entry).unwrap().step, None);
    }

    #[test]
    fn source_formation_keeps_only_the_actual_successful_graph_occurrence() {
        fn fresh_receipt(graph: &Graph, old: Option<&Graph>) -> (usize, usize) {
            let old = old
                .map(|old| {
                    old.inspect_materializations()
                        .unwrap()
                        .into_iter()
                        .collect::<std::collections::BTreeMap<_, _>>()
                })
                .unwrap_or_default();
            graph
                .inspect_materializations()
                .unwrap()
                .iter()
                .filter(|(id, _)| !old.contains_key(id))
                .fold((0, 0), |(bytes, units), (_, usage)| {
                    (bytes + usage.bytes(), units + usage.units())
                })
        }
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, _, work) = f.issue(0);
        drop(work);
        let (child, op, work) = f.issue(1);
        let draft = Draft::new_hosted_source(
            &f.dom,
            DomBatch::new().owner,
            DomSourceLimits::new(16_000_000, 4096, 10_000_000, 0, 0, 0, 1),
            work.spent.clone(),
            work.spent_id,
            op,
            &cancel,
        )
        .unwrap();
        let root_form = source::formation::Formation::new(&draft, &cancel)
            .unwrap()
            .unwrap();
        let context = Context::new(&f.source, &work, &cancel, 0).with_formation(&root_form);
        let root = Graph::initial(None, f.input(top, None), &context).unwrap();
        let receipt = root.payload(|| Ok(())).unwrap();
        assert_eq!(
            (
                root_form.owned_payload().bytes,
                root_form.owned_payload().units
            ),
            (receipt.bytes, 72)
        );
        assert_eq!(root.accounted, (receipt.bytes, receipt.units));
        let child_form = root_form.fork();
        let context = Context::new(&f.source, &work, &cancel, 0).with_formation(&child_form);
        let initial = Graph::initial(Some(&root), f.input(child, Some(top)), &context).unwrap();
        assert_eq!(
            (
                child_form.owned_payload().bytes,
                child_form.owned_payload().units
            ),
            fresh_receipt(&initial, Some(&root))
        );
        let queued_form = root_form.fork();
        let context = Context::new(&f.source, &work, &cancel, 0).with_formation(&queued_form);
        let queued = initial
            .mark_creation_queued(child.navigable, op, &context)
            .unwrap();
        assert_eq!(
            (
                queued_form.owned_payload().bytes,
                queued_form.owned_payload().units
            ),
            fresh_receipt(&queued, Some(&initial))
        );
        let append_form = root_form.fork();
        let context = Context::new(&f.source, &work, &cancel, 0).with_formation(&append_form);
        let appended = queued
            .append_creation(child.navigable, op, &context)
            .unwrap();
        let receipt = appended.payload(|| Ok(())).unwrap();
        assert_eq!(context.scratch.get(), 0);
        // The discarded assignment graph and lookup queues have released their
        // debits; shared row/collection occurrences in this output remain live.
        assert_eq!(
            (
                append_form.owned_payload().bytes,
                append_form.owned_payload().units
            ),
            fresh_receipt(&appended, Some(&queued))
        );
        assert_eq!(receipt.units, 144);
        assert_eq!(appended.accounted, (receipt.bytes, receipt.units));
        assert!(root.entry(top.entry).unwrap().nested.values.is_empty());
        assert_eq!(
            appended.entry(top.entry).unwrap().nested.values,
            [child.navigable]
        );
    }

    #[test]
    fn source_entry_unit_refusal_drops_the_fresh_candidate_debit_without_work_refund() {
        let cancel = AtomicBool::new(false);
        let mut f = Fixture::new();
        let (top, op, work) = f.issue(0);
        let mut draft = Draft::new_hosted_source(
            &f.dom,
            DomBatch::new().owner,
            DomSourceLimits::new(16_000_000, 4096, 10_000_000, 0, 0, 0, 1),
            work.spent.clone(),
            work.spent_id,
            op,
            &cancel,
        )
        .unwrap();
        let base_units = draft.source_storage_units(&cancel).unwrap() + draft.source_other_units;
        let limit = base_units + 71;
        draft.source_limits = Some(DomSourceLimits::new(
            16_000_000, limit, 10_000_000, 0, 0, 0, 1,
        ));
        let live = draft.source_formation_bytes.clone();
        let live_units = draft.source_temporary_units.clone();
        let original_live = live.load(Ordering::Relaxed);
        let original_units = live_units.load(Ordering::Relaxed);
        let original_work = work.spent.load(Ordering::Relaxed);
        let original_bytes = f.used();
        {
            let fresh = source::formation::Formation::new(&draft, &cancel)
                .unwrap()
                .unwrap();
            let context = Context::new(&f.source, &work, &cancel, 0).with_formation(&fresh);
            assert!(matches!(Graph::initial(None,f.input(top,None),&context),
                Err(DomHostedError::Dom(DomError::ResourceLimit {resource:crate::DomResource::SourceRetainedUnits,limit:actual})) if actual==limit));
            assert_eq!(context.scratch.get(), 0);
        }
        assert_eq!(live.load(Ordering::Relaxed), original_live);
        assert_eq!(live_units.load(Ordering::Relaxed), original_units);
        assert_eq!(f.used(), original_bytes);
        assert!(work.spent.load(Ordering::Relaxed) > original_work);
    }

    mod document_index_tests;
    mod lookup_work_tests;
}

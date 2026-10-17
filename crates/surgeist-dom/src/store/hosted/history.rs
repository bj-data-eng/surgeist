//! The one canonical navigable/entry/DocumentState/history-collection graph.
//! Rows use the original issuer's identities; no independent allocator or
//! Runtime activity/history mirror is admitted here.
use super::*;
use std::cell::Cell;
use std::ops::Deref;
use surgeist_html_semantics::HtmlAboutBaseUrl;

/// One synchronous graph call borrows the original issuer and cumulative Work.
/// `scratch_base` is the caller's other simultaneously live formation extent.
pub(crate) struct Context<'a> {
    source: &'a DomHostedSource,
    work: &'a Work,
    cancel: &'a AtomicBool,
    scratch: Cell<usize>,
    formation: Option<&'a source::formation::Formation>,
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
}
impl Drop for Call<'_, '_> {
    fn drop(&mut self) {
        assert!(
            self.context.scratch.get() >= self.baseline,
            "call cannot release its caller's scratch"
        );
        self.context.scratch.set(self.baseline);
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
            context.step()?;
            candidate.values.push(*value);
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
            }
            std::mem::swap(self, &mut next);
        }
        self.values.push(value);
        Ok(())
    }
    fn contains(&self, value: &T) -> Result<bool, DomHostedError>
    where
        T: PartialEq,
    {
        for old in &self.values {
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
        Arc::new(Values {
            values,
            _retained: retained,
        })
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
struct Row<T> {
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
        value: form()?,
        retained,
        scratch,
        credit: parent,
    })
}
fn row<T>(value: Changed<'_, '_, T>, context: &Context<'_>) -> Result<Arc<Row<T>>, DomHostedError> {
    context.step()?;
    let Changed {
        value,
        retained,
        scratch,
        credit: parent,
    } = value;
    scratch.stage();
    parent.commit();
    Ok(Arc::new(Row {
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
#[derive(Debug)]
pub(crate) struct Graph {
    accounted: (usize, usize),
    navigables: Vec<Option<Arc<Row<Navigable>>>>,
    entries: Vec<Option<Arc<Row<Entry>>>>,
    _retained: Retained,
}
impl PartialEq for Graph {
    fn eq(&self, other: &Self) -> bool {
        fn bank<T>(a: &[Option<Arc<T>>], b: &[Option<Arc<T>>]) -> bool {
            a.len() == b.len()
                && a.iter().zip(b).all(|(a, b)| match (a, b) {
                    (None, None) => true,
                    (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                    _ => false,
                })
        }
        bank(&self.navigables, &other.navigables) && bank(&self.entries, &other.entries)
    }
}
impl Eq for Graph {}
impl Graph {
    pub(crate) fn navigable(&self, id: DomHostedNavigableId) -> Result<&Navigable, DomHostedError> {
        self.navigable_with(id, || Ok(()))
    }
    pub(crate) fn entry(&self, id: DomHostedEntryId) -> Result<&Entry, DomHostedError> {
        self.entry_with(id, || Ok(()))
    }
    /// Lend the caller's original admission/cancellation to every physical row,
    /// including empty slots. Historical observations supply a no-budget step.
    pub(crate) fn navigable_with(
        &self,
        id: DomHostedNavigableId,
        mut step: impl FnMut() -> Result<(), DomHostedError>,
    ) -> Result<&Navigable, DomHostedError> {
        for row in &self.navigables {
            step()?;
            if let Some(row) = row
                && row.id == id
            {
                return Ok(&row.value);
            }
        }
        Err(DomHostedError::StaleIdentity)
    }
    pub(crate) fn entry_with(
        &self,
        id: DomHostedEntryId,
        mut step: impl FnMut() -> Result<(), DomHostedError>,
    ) -> Result<&Entry, DomHostedError> {
        for row in &self.entries {
            step()?;
            if let Some(row) = row
                && row.id == id
            {
                return Ok(&row.value);
            }
        }
        Err(DomHostedError::StaleIdentity)
    }
    pub(crate) fn payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        step()?;
        let mut p = source::storage::Payload::default();
        p.add_bytes(size_of::<Self>())?;
        p.bank::<Option<Arc<Row<Navigable>>>>(self.navigables.capacity())?;
        p.bank::<Option<Arc<Row<Entry>>>>(self.entries.capacity())?;
        for nav in &self.navigables {
            step()?;
            let Some(nav) = nav else { continue };
            p.add_bytes(size_of::<Row<Navigable>>())?;
            p.add_bytes(nav.entries.bytes()?)?;
        }
        for entry in &self.entries {
            step()?;
            let Some(entry) = entry else { continue };
            p.append(entry.value.payload()?)?;
        }
        Ok(p)
    }
    fn form(
        old: Option<&Self>,
        nav: Option<Arc<Row<Navigable>>>,
        entry: Option<Arc<Row<Entry>>>,
        context: &Context<'_>,
    ) -> Result<Arc<Self>, DomHostedError> {
        context.step()?;
        let mut nav_index = None;
        let mut entry_index = None;
        if let Some(old) = old {
            for (index, prior) in old.navigables.iter().enumerate() {
                context.step()?;
                if prior
                    .as_ref()
                    .is_some_and(|prior| nav.as_ref().is_some_and(|next| prior.id == next.id))
                {
                    nav_index = Some(index);
                }
            }
            for (index, prior) in old.entries.iter().enumerate() {
                context.step()?;
                if prior
                    .as_ref()
                    .is_some_and(|prior| entry.as_ref().is_some_and(|next| prior.id == next.id))
                {
                    entry_index = Some(index);
                }
            }
        }
        let old_nav_len = old.map_or(0, |g| g.navigables.len());
        let old_entry_len = old.map_or(0, |g| g.entries.len());
        let nav_len = old_nav_len
            .checked_add(usize::from(nav.is_some() && nav_index.is_none()))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let entry_len = old_entry_len
            .checked_add(usize::from(entry.is_some() && entry_index.is_none()))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let extent = |n: usize, e: usize| {
            n.checked_mul(size_of::<Option<Arc<Row<Navigable>>>>())
                .and_then(|n| {
                    e.checked_mul(size_of::<Option<Arc<Row<Entry>>>>())
                        .and_then(|e| n.checked_add(e))
                })
                .and_then(|n| n.checked_add(size_of::<Self>()))
                .ok_or(DomHostedError::RepresentationExhausted)
        };
        let requested = extent(nav_len, entry_len)?;
        // Supplied private rows/lists still own their staged credits. Only this
        // new Graph bank and newly cloned old-row occurrences are additional.
        let mut scratch = context.scratch(requested)?;
        let retained = context.source.retain(requested, context.cancel)?;
        let mut parent = context.parent(source::storage::Payload {
            bytes: requested,
            units: 0,
        })?;
        let mut candidate = Self {
            accounted: (0, 0),
            navigables: Vec::new(),
            entries: Vec::new(),
            _retained: retained,
        };
        candidate
            .navigables
            .try_reserve_exact(nav_len)
            .map_err(|_| DomHostedError::AllocationFailed)?;
        let first = extent(candidate.navigables.capacity(), entry_len)?;
        scratch.replace(first)?;
        parent.add(
            source::storage::Payload {
                bytes: first
                    .checked_sub(requested)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                units: 0,
            },
            context.cancel,
        )?;
        candidate._retained.append(
            context.source.retain(
                first
                    .checked_sub(requested)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                context.cancel,
            )?,
        )?;
        candidate
            .entries
            .try_reserve_exact(entry_len)
            .map_err(|_| DomHostedError::AllocationFailed)?;
        let actual = extent(
            candidate.navigables.capacity(),
            candidate.entries.capacity(),
        )?;
        scratch.replace(actual)?;
        parent.add(
            source::storage::Payload {
                bytes: actual
                    .checked_sub(first)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                units: 0,
            },
            context.cancel,
        )?;
        candidate._retained.append(
            context.source.retain(
                actual
                    .checked_sub(first)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                context.cancel,
            )?,
        )?;
        let mut total = source::storage::Payload {
            bytes: actual,
            units: 0,
        };
        for i in 0..nav_len {
            context.step()?;
            if Some(i) == nav_index || (nav_index.is_none() && i == old_nav_len && nav.is_some()) {
                let value = nav.as_ref().expect("new navigable row");
                total.append(value.value.payload()?)?;
                candidate.navigables.push(None);
            } else {
                let value = old
                    .and_then(|g| g.navigables.get(i))
                    .and_then(Option::as_ref);
                let mut occurrence = None;
                if let Some(value) = value {
                    let payload = value.value.payload()?;
                    occurrence = Some(context.scratch(payload.bytes)?);
                    parent.add(payload, context.cancel)?;
                    total.append(payload)?;
                }
                candidate.navigables.push(value.cloned());
                if let Some(occurrence) = occurrence {
                    occurrence.stage();
                }
            }
        }
        for i in 0..entry_len {
            context.step()?;
            if Some(i) == entry_index
                || (entry_index.is_none() && i == old_entry_len && entry.is_some())
            {
                let value = entry.as_ref().expect("new entry row");
                total.append(value.value.payload()?)?;
                candidate.entries.push(None);
            } else {
                let value = old.and_then(|g| g.entries.get(i)).and_then(Option::as_ref);
                let mut occurrence = None;
                if let Some(value) = value {
                    let payload = value.value.payload()?;
                    occurrence = Some(context.scratch(payload.bytes)?);
                    parent.add(payload, context.cancel)?;
                    total.append(payload)?;
                }
                candidate.entries.push(value.cloned());
                if let Some(occurrence) = occurrence {
                    occurrence.stage();
                }
            }
        }
        if let Some(nav) = nav {
            context.step()?;
            candidate.navigables[nav_index.unwrap_or(old_nav_len)] = Some(nav);
        }
        if let Some(entry) = entry {
            context.step()?;
            candidate.entries[entry_index.unwrap_or(old_entry_len)] = Some(entry);
        }
        candidate.accounted = (total.bytes, total.units);
        parent.commit();
        scratch.stage();
        Ok(Arc::new(candidate))
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
            for prior in &old.navigables {
                context.step()?;
                if prior
                    .as_ref()
                    .is_some_and(|prior| prior.id == creation.navigable)
                {
                    return Err(DomHostedError::WrongPhase);
                }
            }
            for prior in &old.entries {
                context.step()?;
                if prior
                    .as_ref()
                    .is_some_and(|prior| prior.id == creation.entry)
                {
                    return Err(DomHostedError::WrongPhase);
                }
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
            context.step()?;
            let id = pending.values[cursor];
            let nav = self.nav(id, context)?;
            if id == wanted {
                return Ok(Some(&nav.entries));
            }
            for entry in &nav.entries.values {
                context.step()?;
                for child in &self.ent(*entry, context)?.nested.values {
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
            context.step()?;
            if *id == child {
                return Err(DomHostedError::WrongPhase);
            }
            nested.values.push(*id);
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
        let extent = source::storage::Payload {
            bytes: assigned.accounted.0,
            units: assigned.accounted.1,
        };
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
            context.step()?;
            if *id != child {
                nested.values.push(*id);
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
            context.step()?;
            let nav = self.nav(pending.values[cursor], context)?;
            for id in &nav.entries.values {
                context.step()?;
                let entry = self.ent(*id, context)?;
                let step = entry.step.ok_or(DomHostedError::MissingPublication)?;
                if !steps.contains(&step)? {
                    steps.push(step)?;
                    let mut index = steps.values.len() - 1;
                    while index > 0 {
                        context.step()?;
                        if steps.values[index - 1] <= steps.values[index] {
                            break;
                        }
                        steps.values.swap(index - 1, index);
                        index -= 1;
                    }
                }
                for child in &entry.nested.values {
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
        for slot in 0..self.entries.len() {
            context.step()?;
            if let Some(entry) = &self.entries[slot]
                && entry.document == Some(document)
            {
                let mut next = entry.retained_copy(context)?;
                next.document = None;
                let prior = candidate;
                candidate = Self::form(Some(&prior), None, Some(row(next, context)?), context)?;
                let extent = source::storage::Payload {
                    bytes: prior.accounted.0,
                    units: prior.accounted.1,
                };
                drop(prior);
                context.release_output(extent);
            }
        }
        for slot in 0..self.navigables.len() {
            context.step()?;
            if let Some(nav) = &self.navigables[slot] {
                context.step()?;
                if self.ent(nav.active, context)?.document == Some(document) {
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
                    let extent = source::storage::Payload {
                        bytes: prior.accounted.0,
                        units: prior.accounted.1,
                    };
                    drop(prior);
                    context.release_output(extent);
                }
            }
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
mod tests {
    use crate::DomSourceLimits;
    // Real issuer IDs and native Document identity payloads exercise this data
    // owner. They do not attest hosted Globals, entropy, loops or external jobs.
    use super::*;
    use std::sync::atomic::Ordering;

    struct Fixture {
        dom: Dom,
        source: DomHostedSource,
        host: DomHostedHostId,
        documents: [DocumentId; 5],
        origin: crate::DomOpaqueOrigin,
        target_name: DocumentString,
        about_base_url: HtmlAboutBaseUrl,
    }
    impl Fixture {
        fn new() -> Self {
            let cancel = AtomicBool::new(false);
            let mut dom =
                Dom::new(DomLimits::new(64, 4096, 64, 0, 0, 16, 4_000_000).unwrap()).unwrap();
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
        fn issue(
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
        fn input(
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
    fn creation_scratch_admits_both_complete_candidate_occurrences_until_transfer() {
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
            let banks = size_of::<Graph>()
                + graph.navigables.capacity() * size_of::<Option<Arc<Row<Navigable>>>>()
                + graph.entries.capacity() * size_of::<Option<Arc<Row<Entry>>>>();
            let collections = graph
                .navigables
                .iter()
                .map(|row| {
                    let row = row.as_ref().unwrap();
                    assert_eq!(row.entries.values.len(), 1);
                    size_of::<Values<DomHostedEntryId>>()
                        + row.entries.values.capacity() * size_of::<DomHostedEntryId>()
                })
                .sum::<usize>();
            let nested = graph
                .entries
                .iter()
                .map(|row| {
                    let row = row.as_ref().unwrap();
                    assert_eq!(row.about_base_url, HtmlAboutBaseUrl::KnownNull);
                    size_of::<Values<DomHostedNavigableId>>()
                        + row.nested.values.capacity() * size_of::<DomHostedNavigableId>()
                })
                .sum::<usize>();
            assert_eq!(
                graph
                    .entry(top.entry)
                    .unwrap()
                    .target_name
                    .as_ref()
                    .unwrap()
                    .shared_code_units()
                    .capacity(),
                0
            );
            assert_eq!(graph.entry(child.entry).unwrap().target_name, None);
            banks
                + 2 * size_of::<Row<Navigable>>()
                + 2 * size_of::<Row<Entry>>()
                + collections
                + nested
                + size_of::<Vec<u16>>()
        };
        let assigned_extent = extent(&queued);
        assert_eq!(queued.payload(|| Ok(())).unwrap().bytes, assigned_extent);
        let output_extent = {
            let context = Context::new(&f.source, &work, &cancel, 0);
            let output = queued
                .append_creation(child.navigable, op, &context)
                .unwrap();
            assert_eq!(context.scratch.get(), 0);
            let bytes = extent(&output);
            assert_eq!(output.payload(|| Ok(())).unwrap().bytes, bytes);
            bytes
        };
        // C first forms a complete assigned Graph with only scalar changes;
        // it stays live while the second complete Graph adds the real H list.
        // The caller's unrelated live formation backing remains present too.
        let caller_buffer = vec![0u8; 17];
        let caller_bytes = size_of::<Vec<u8>>() + caller_buffer.capacity();
        let exact_peak = caller_bytes + assigned_extent + output_extent;
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
    fn source_formation_keeps_only_the_actual_successful_graph_occurrence() {
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
        let queued_form = root_form.fork();
        let context = Context::new(&f.source, &work, &cancel, 0).with_formation(&queued_form);
        let queued = initial
            .mark_creation_queued(child.navigable, op, &context)
            .unwrap();
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
            (receipt.bytes, receipt.units)
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

    mod lookup_work_tests;
}

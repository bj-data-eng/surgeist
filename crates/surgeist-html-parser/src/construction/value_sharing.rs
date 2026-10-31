//! Terminal report promotion. No mutable parser session can acquire these pools.
use super::*;
#[cfg(test)]
use crate::request::ValueSharingMode;
use surgeist_html::bounded::{
    HtmlAttributePool, HtmlAttributePoolOperation, HtmlAttributePoolResult, HtmlStringPool,
    HtmlStringPoolOperation, HtmlStringPoolResult, HtmlValuePoolActionKind, HtmlValuePoolError,
    HtmlValuePoolProgress, HtmlValuePoolRoot, HtmlValuePoolSeal, HtmlValuePoolUsage,
    HtmlValuePoolWork,
};

fn pool_error(c: &mut ParseControl, error: HtmlValuePoolError) -> SessionError {
    if matches!(error, HtmlValuePoolError::RepresentationExhausted) {
        c.representation::<()>(HtmlParseResource::ManagedCapacityBytes)
            .unwrap_err()
            .into()
    } else {
        c.fail_invariant(crate::parser::control::Invariant::CapacityMismatch)
            .into()
    }
}
fn work(c: &mut ParseControl, work: HtmlValuePoolWork) -> Result<(), SessionError> {
    c.admit_pool_work(work)?;
    Ok(())
}
#[derive(Clone, Copy, Default)]
struct Usage {
    bytes: u64,
    units: u64,
}
impl Usage {
    fn pool(value: HtmlValuePoolUsage) -> Self {
        Self {
            bytes: value.bytes() as u64,
            units: value.utf16_units() as u64,
        }
    }
    fn retain(self, c: &mut ParseControl) -> Result<(), SessionError> {
        c.acquire_retained_provider(self.bytes, self.units, 0)?;
        Ok(())
    }
    fn release(self, c: &mut ParseControl) {
        c.release_retained_provider(&c.identity(), self.bytes, self.units, 0);
    }
    fn grow(&mut self, target: Self, c: &mut ParseControl) -> Result<(), SessionError> {
        self.adjust(
            Self {
                bytes: self.bytes.max(target.bytes),
                units: self.units.max(target.units),
            },
            c,
        )
    }
    fn adjust(&mut self, target: Self, c: &mut ParseControl) -> Result<(), SessionError> {
        let added = Self {
            bytes: target.bytes.saturating_sub(self.bytes),
            units: target.units.saturating_sub(self.units),
        };
        if added.bytes != 0 || added.units != 0 {
            added.retain(c)?;
        }
        let removed = Self {
            bytes: self.bytes.saturating_sub(target.bytes),
            units: self.units.saturating_sub(target.units),
        };
        removed.release(c);
        *self = target;
        Ok(())
    }
}
/// A scoped terminal value never outlives its exact original-controller credit.
/// The disjoint borrow lets useful work observe the value while Drop retains
/// exclusive cleanup authority on both refusal and unwind.
trait Temporary {
    fn release(self, c: &mut ParseControl);
}
struct Pending<'c, T: Temporary> {
    value: Option<T>,
    control: &'c mut ParseControl,
}
impl<'c, T: Temporary> Pending<'c, T> {
    fn new(value: T, control: &'c mut ParseControl) -> Self {
        Self {
            value: Some(value),
            control,
        }
    }
    fn parts(&mut self) -> (&mut T, &mut ParseControl) {
        (
            self.value.as_mut().expect("owned terminal temporary"),
            self.control,
        )
    }
}
impl<T: Temporary> Drop for Pending<'_, T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            value.release(self.control);
        }
    }
}
struct Selected<T> {
    value: T,
    usage: Usage,
}
impl<T> Temporary for Selected<T> {
    fn release(self, c: &mut ParseControl) {
        let Self { value, usage } = self;
        drop(value);
        usage.release(c);
    }
}
// Authority is affine and remains inside the same controller-borrowing guard
// until the receiver has admitted its final transfer and polled cancellation.
enum Authority {
    Closed,
    Open(HtmlValuePoolRoot),
    Prepared(HtmlValuePoolSeal),
}
pub(super) struct Sharing<'c> {
    pools: Option<Pools>,
    control: &'c mut ParseControl,
}
impl<'c> Sharing<'c> {
    fn new(
        control: &'c mut ParseControl,
        #[cfg(test)] mode: ValueSharingMode,
    ) -> Result<Self, SessionError> {
        let pools = Pools::new(
            control,
            #[cfg(test)]
            mode,
        )?;
        let mut result = Self {
            pools: Some(pools),
            control,
        };
        result.pools.as_mut().unwrap().open(result.control)?;
        Ok(result)
    }
    pub(super) fn control(&mut self) -> &mut ParseControl {
        #[cfg(test)]
        self.pools.as_ref().unwrap().observe_transfer(
            crate::parser::control::FinalizationTransferStage::Receiver,
            self.control,
        );
        self.control
    }
    pub(super) fn commit(mut self) {
        let mut pools = self.pools.take().expect("prepared pair");
        pools.seal();
        pools.release(self.control);
    }
}
impl Drop for Sharing<'_> {
    fn drop(&mut self) {
        if let Some(mut pools) = self.pools.take() {
            pools.abort();
            // Pools release their actual data before each matching credit.
            // Disposal/root/action closing work was admitted before retention.
            pools.release(self.control);
        }
    }
}
struct Pools {
    strings: HtmlStringPool,
    attributes: HtmlAttributePool,
    strings_usage: Usage,
    attributes_usage: Usage,
    strings_root: Authority,
    attributes_root: Authority,
    #[cfg(test)]
    mode: ValueSharingMode,
    #[cfg(test)]
    string_insertions: usize,
    #[cfg(test)]
    attribute_insertions: usize,
}
macro_rules! drive {
    ($name:ident, $guard:ident, $pool:ident, $usage:ident, $insertions:ident, $operation:ident, $result:ident, $value:ty) => {
        fn $name(
            &mut self,
            input: &$value,
            offer: bool,
            c: &mut ParseControl,
        ) -> Result<Option<Selected<$value>>, SessionError> {
            let mut pending = Pending::new(
                $guard {
                    owner: Some(if offer {
                        self.$pool.offer(input)
                    } else {
                        self.$pool.probe(input)
                    }),
                    candidate: Usage::default(),
                },
                c,
            );
            let (state, control) = pending.parts();
            let owner = &mut state.owner;
            let candidate = &mut state.candidate;
            let result = (|| {
                while let Some(action) = self
                    .$pool
                    .next_action(owner.as_mut().unwrap())
                    .map_err(|e| pool_error(control, e))?
                {
                    let layout = action.layout();
                    work(control, layout.work())?;
                    work(control, layout.closing_work())?;
                    self.$usage.grow(Usage::pool(layout.pool()), control)?;
                    candidate.grow(Usage::pool(layout.candidate()), control)?;
                    let progress = match action.kind() {
                        HtmlValuePoolActionKind::Read => owner.as_mut().unwrap().read(action),
                        HtmlValuePoolActionKind::Pool => {
                            self.$pool.perform(owner.as_mut().unwrap(), action)
                        }
                    }
                    .map_err(|e| pool_error(control, e))?;
                    if progress == HtmlValuePoolProgress::CandidateReleased {
                        candidate.release(control);
                        *candidate = Usage::default();
                    }
                }
                Ok(
                    match $operation::try_finish(owner).map_err(|e| pool_error(control, e))? {
                        $result::Hit(value) => Some(Selected {
                            value: value.into_value(),
                            usage: std::mem::take(candidate),
                        }),
                        $result::Offered(change) => {
                            let _ = change;
                            #[cfg(test)]
                            {
                                self.$insertions += usize::from(change.inserted().is_some());
                            }
                            None
                        }
                        $result::Miss | $result::Bypassed => None,
                    },
                )
            })();
            // Operation owns any provisional allocation and strong candidate.
            drop(pending);
            self.$usage
                .adjust(Usage::pool(self.$pool.heap_usage()), c)
                .expect("preadmitted pool extent");
            result
        }
    };
}
struct StringOperation<'a> {
    owner: Option<HtmlStringPoolOperation<'a>>,
    candidate: Usage,
}
impl Temporary for StringOperation<'_> {
    fn release(self, c: &mut ParseControl) {
        drop(self.owner);
        self.candidate.release(c);
    }
}
struct AttributeOperation<'a> {
    owner: Option<HtmlAttributePoolOperation<'a>>,
    candidate: Usage,
}
impl Temporary for AttributeOperation<'_> {
    fn release(self, c: &mut ParseControl) {
        drop(self.owner);
        self.candidate.release(c);
    }
}
impl Pools {
    fn new(
        c: &mut ParseControl,
        #[cfg(test)] mode: ValueSharingMode,
    ) -> Result<Self, SessionError> {
        // This fixed producer envelope covers final destruction even if a
        // later admission refuses. No terminal cleanup callback is required.
        work(c, HtmlStringPool::disposal_work())?;
        work(c, HtmlAttributePool::disposal_work())?;
        Usage {
            bytes: mem::size_of::<Self>() as u64,
            units: 0,
        }
        .retain(c)?;
        Ok(Self {
            strings: HtmlStringPool::new(),
            attributes: HtmlAttributePool::new(),
            strings_usage: Usage::default(),
            attributes_usage: Usage::default(),
            strings_root: Authority::Closed,
            attributes_root: Authority::Closed,
            #[cfg(test)]
            mode,
            #[cfg(test)]
            string_insertions: 0,
            #[cfg(test)]
            attribute_insertions: 0,
        })
    }
    drive!(
        drive_string,
        StringOperation,
        strings,
        strings_usage,
        string_insertions,
        HtmlStringPoolOperation,
        HtmlStringPoolResult,
        DocumentString
    );
    drive!(
        drive_attributes,
        AttributeOperation,
        attributes,
        attributes_usage,
        attribute_insertions,
        HtmlAttributePoolOperation,
        HtmlAttributePoolResult,
        OrderedAttributes
    );
    fn string(
        &mut self,
        input: &DocumentString,
        c: &mut ParseControl,
    ) -> Result<Option<Selected<DocumentString>>, SessionError> {
        if !self.strings_enabled() {
            return Ok(None);
        }
        // Offer returns existing hits before preparing any mutation. A miss
        // keeps its qualified fingerprint and scan through the insertion.
        self.drive_string(input, true, c)
    }
    fn offer_attributes(
        &mut self,
        input: &OrderedAttributes,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        if !self.attributes_enabled() {
            return Ok(());
        }
        if let Some(hit) = self.drive_attributes(input, true, c)? {
            hit.release(c);
        }
        Ok(())
    }
    fn enabled(&self) -> bool {
        matches!(self.strings_root, Authority::Open(_))
            && matches!(self.attributes_root, Authority::Open(_))
    }
    fn strings_enabled(&self) -> bool {
        #[cfg(test)]
        if self.mode != ValueSharingMode::Combined {
            return false;
        }
        self.enabled()
    }
    fn attributes_enabled(&self) -> bool {
        #[cfg(test)]
        if self.mode == ValueSharingMode::Off {
            return false;
        }
        self.enabled()
    }
    fn open(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        work(c, HtmlStringPool::root_work())?;
        work(c, HtmlAttributePool::root_work())?;
        if let Some(root) = self.strings.begin_root() {
            self.strings_root = Authority::Open(root);
        } else {
            return Ok(());
        }
        if let Some(root) = self.attributes.begin_root() {
            self.attributes_root = Authority::Open(root);
        } else {
            self.abort();
        }
        Ok(())
    }
    fn prepare(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        if !self.enabled() {
            return Ok(());
        }
        // Both closing allowances precede either freeze. A later refusal still
        // leaves every already-owned root/plan abortable without admission.
        work(c, HtmlStringPool::seal_work())?;
        work(c, HtmlAttributePool::seal_work())?;
        let Authority::Open(root) = mem::replace(&mut self.strings_root, Authority::Closed) else {
            unreachable!("open string root")
        };
        match self.strings.prepare_seal(root) {
            Ok(seal) => self.strings_root = Authority::Prepared(seal),
            Err((error, root)) => {
                self.strings_root = Authority::Open(root);
                return Err(pool_error(c, error));
            }
        }
        #[cfg(test)]
        self.observe_transfer(
            crate::parser::control::FinalizationTransferStage::FirstPrepared,
            c,
        );
        let Authority::Open(root) = mem::replace(&mut self.attributes_root, Authority::Closed)
        else {
            unreachable!("open attribute root")
        };
        match self.attributes.prepare_seal(root) {
            Ok(seal) => self.attributes_root = Authority::Prepared(seal),
            Err((error, root)) => {
                self.attributes_root = Authority::Open(root);
                return Err(pool_error(c, error));
            }
        }
        if let Authority::Prepared(seal) = &self.strings_root {
            self.strings
                .validate_seal(seal)
                .map_err(|e| pool_error(c, e))?;
        }
        if let Authority::Prepared(seal) = &self.attributes_root {
            self.attributes
                .validate_seal(seal)
                .map_err(|e| pool_error(c, e))?;
        }
        Ok(())
    }
    #[cfg(test)]
    fn observe_transfer(
        &self,
        stage: crate::parser::control::FinalizationTransferStage,
        c: &mut ParseControl,
    ) {
        c.observe_finalization_transfer(crate::parser::control::FinalizationTransferObservation {
            stage,
            strings_prepared: matches!(self.strings_root, Authority::Prepared(_)),
            attributes_prepared: matches!(self.attributes_root, Authority::Prepared(_)),
            pool_bytes: self.strings.heap_usage().bytes() + self.attributes.heap_usage().bytes(),
            retained_bytes: c.usage().actual_capacity_bytes(),
            work: c.usage().resource(HtmlParseResource::WorkCredits).current,
        });
    }
    fn seal(&mut self) {
        if let Authority::Prepared(seal) = mem::replace(&mut self.strings_root, Authority::Closed) {
            self.strings.seal(seal).expect("prevalidated string seal");
        }
        if let Authority::Prepared(seal) =
            mem::replace(&mut self.attributes_root, Authority::Closed)
        {
            self.attributes
                .seal(seal)
                .expect("prevalidated attribute seal");
        }
    }
    fn abort(&mut self) {
        match mem::replace(&mut self.strings_root, Authority::Closed) {
            Authority::Open(root) => {
                self.strings.abort_root(root).expect("owned string root");
            }
            Authority::Prepared(seal) => {
                self.strings.abort_seal(seal).expect("owned string seal");
            }
            Authority::Closed => {}
        }
        match mem::replace(&mut self.attributes_root, Authority::Closed) {
            Authority::Open(root) => {
                self.attributes
                    .abort_root(root)
                    .expect("owned attribute root");
            }
            Authority::Prepared(seal) => {
                self.attributes
                    .abort_seal(seal)
                    .expect("owned attribute seal");
            }
            Authority::Closed => {}
        }
    }
    fn release(self, c: &mut ParseControl) {
        let Self {
            strings,
            attributes,
            strings_usage,
            attributes_usage,
            ..
        } = self;
        drop(strings);
        drop(attributes);
        strings_usage.release(c);
        attributes_usage.release(c);
        Usage {
            bytes: mem::size_of::<Self>() as u64,
            units: 0,
        }
        .release(c);
    }
}

/// Once original alias/history checks pass, this owner can only finalize or
/// release. The mutable tokenizer/tree-builder never receives it again.
pub(super) struct Finalizing {
    owner: ControlIdentity,
    construction: ConstructionOwner,
    payloads: PayloadStorage,
    effects: ConstructionEffects,
    evidence: Option<ParseEvidence>,
    fragment: Option<FragmentContextStorage>,
    root: HtmlBuilderNode,
    joins: Vec<Join>,
    joins_allocation: VectorCapacity<Join>,
}
#[derive(Clone, Copy)]
struct Join {
    head: Option<usize>,
    next: Option<usize>,
    update: Option<usize>,
}
struct Update {
    slot: usize,
    value: DocumentString,
    allocation: DocumentStringAllocation,
}
struct Updates {
    values: Vec<Update>,
    allocation: VectorCapacity<Update>,
}
impl Updates {
    fn new(c: &mut ParseControl) -> Result<Self, SessionError> {
        let (values, allocation) = c.new_vector()?;
        Ok(Self { values, allocation })
    }
    fn push(
        &mut self,
        slot: usize,
        value: &DocumentString,
        c: &mut ParseControl,
    ) -> Result<usize, SessionError> {
        c.reserve_vector_geometric(&mut self.values, &mut self.allocation, 1)?;
        payload_write::<Update>(c, 1)?;
        let allocation = c.final_string_ticket(value)?;
        let index = self.values.len();
        self.values.push(Update {
            slot,
            value: value.clone(),
            allocation,
        });
        Ok(index)
    }
    fn release(mut self, c: &mut ParseControl) {
        while let Some(Update {
            value, allocation, ..
        }) = self.values.pop()
        {
            drop(value);
            c.release_document_string(allocation)
                .expect("owned final string");
        }
        release_vector(c, self.values, self.allocation).expect("owned update vector");
    }
    fn commit(mut self, payloads: &mut PayloadStorage, c: &mut ParseControl) {
        while let Some(Update {
            slot,
            value: new,
            allocation: new_allocation,
        }) = self.values.pop()
        {
            let RetainedPayload::String {
                value, allocation, ..
            } = &mut payloads.values[slot]
            else {
                unreachable!("qualified row")
            };
            let old = mem::replace(value, new);
            let mut ticket = mem::replace(allocation, new_allocation);
            // This terminal transition retires an occurrence, without claiming
            // that a shared target was physically deallocated.
            ticket.promote_final();
            drop(old);
            c.release_document_string(ticket)
                .expect("retired terminal occurrence");
        }
        release_vector(c, self.values, self.allocation).expect("owned update vector");
    }
}
impl Temporary for Updates {
    fn release(self, c: &mut ParseControl) {
        Updates::release(self, c);
    }
}
struct Parts(OwnedVec<[Option<usize>; 4]>);
impl Temporary for Parts {
    fn release(self, c: &mut ParseControl) {
        self.0.release(c).expect("owned component map");
    }
}
// One temporary owner follows vector -> shared backing -> validated bank. Every
// callee receives its actual data by move; on unwind that callee destroys it
// before this caller's tickets are retired.
struct BankOutput {
    vector: Option<Vec<Attribute>>,
    backing: Option<Arc<Vec<Attribute>>>,
    values: Option<OrderedAttributes>,
    vector_ticket: Option<VectorCapacity<Attribute>>,
    shared: Option<SharedAllocation<Vec<Attribute>>>,
    attributes: u64,
}
impl BankOutput {
    fn vector(c: &mut ParseControl) -> Result<Self, SessionError> {
        let (vector, ticket) = c.new_vector()?;
        Ok(Self {
            vector: Some(vector),
            backing: None,
            values: None,
            vector_ticket: Some(ticket),
            shared: None,
            attributes: 0,
        })
    }
    fn selected(
        values: OrderedAttributes,
        tickets: (VectorCapacity<Attribute>, SharedAllocation<Vec<Attribute>>),
    ) -> Self {
        let attributes = values.len() as u64;
        Self {
            vector: None,
            backing: None,
            values: Some(values),
            vector_ticket: Some(tickets.0),
            shared: Some(tickets.1),
            attributes,
        }
    }
}
impl Temporary for BankOutput {
    fn release(self, c: &mut ParseControl) {
        drop(self.values);
        drop(self.backing);
        drop(self.vector);
        if let Some(shared) = self.shared {
            c.release_final_shared(shared);
        }
        if let Some(vector) = self.vector_ticket {
            c.release_final_vector(vector, self.attributes);
        }
    }
}
struct StringOutput {
    selected: Selected<DocumentString>,
    ticket: Option<DocumentStringAllocation>,
}
impl Temporary for StringOutput {
    fn release(self, c: &mut ParseControl) {
        if let Some(ticket) = self.ticket {
            c.release_document_string(ticket)
                .expect("owned selected output ticket");
        }
        self.selected.release(c);
    }
}
struct Content {
    value: Option<bounded::HtmlConstructionContent>,
    usage: Usage,
}
impl Temporary for Content {
    fn release(self, c: &mut ParseControl) {
        drop(self.value);
        self.usage.release(c);
    }
}
impl Finalizing {
    pub(super) fn require_cleanup(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_owner(c, &self.owner)
    }
    pub(super) fn take_stopped_evidence(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<Option<evidence::FailureDiagnosticEvidence>, SessionError> {
        c.cleanup_release(&self.owner, HtmlParseResource::AuxiliaryEntries, 0)?;
        ParseEvidence::partition_failed(&mut self.evidence, c)
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), SessionError> {
        let Self {
            owner,
            construction,
            payloads,
            effects,
            evidence,
            fragment,
            root,
            joins,
            joins_allocation,
        } = self;
        drop(root);
        release_vector(c, joins, joins_allocation)?;
        if let Some(fragment) = fragment {
            fragment.release(c, &owner)?;
        }
        if let Some(evidence) = evidence {
            evidence.release(c)?;
        }
        effects.release(c)?;
        release_parts(
            c,
            &owner,
            ReleasedStorage {
                construction,
                payloads,
            },
        )
    }

    fn make_joins(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        c.reserve_vector(
            &mut self.joins,
            &mut self.joins_allocation,
            self.payloads.values.len(),
        )?;
        for _ in &self.payloads.values {
            payload_write::<Join>(c, 1)?;
            self.joins.push(Join {
                head: None,
                next: None,
                update: None,
            });
        }
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::String { location, .. } = payload
                && let Some((container, _)) = attribute_location(*location)
            {
                payload_write::<Join>(c, 2)?;
                self.joins[slot].next = self.joins[container].head;
                self.joins[container].head = Some(slot);
            }
        }
        Ok(())
    }

    fn prepare_updates(
        payloads: &PayloadStorage,
        joins: &mut [Join],
        container: usize,
        bank: &OrderedAttributes,
        updates: &mut Updates,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let mut next = joins[container].head;
        while let Some(slot) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            next = joins[slot].next;
            if joins[slot].update.is_none() {
                let RetainedPayload::String { location, .. } = payloads.values[slot] else {
                    unreachable!("string join")
                };
                let (_, component) = attribute_location(location).unwrap();
                payload_write::<Option<usize>>(c, 1)?;
                let update = updates.push(slot, attribute_component(bank, component), c)?;
                joins[slot].update = Some(update);
            }
        }
        Ok(())
    }

    fn share_bank(
        &mut self,
        container: usize,
        pools: &mut Pools,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let mut pending = Pending::new(Updates::new(c)?, c);
        let (updates, c) = pending.parts();
        self.share_bank_inner(container, pools, updates, c)?;
        let updates = pending.value.take().expect("prepared terminal rows");
        updates.commit(&mut self.payloads, pending.control);
        Ok(())
    }

    fn share_bank_inner(
        &mut self,
        container: usize,
        pools: &mut Pools,
        updates: &mut Updates,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
            values: original,
            element: Some(element),
            ..
        })) = &self.payloads.values[container]
        else {
            return Ok(());
        };
        let element = *element;
        if let Some(selected) = if pools.attributes_enabled() {
            pools.drive_attributes(original, false, c)?
        } else {
            None
        } {
            let mut pending = Pending::new(selected, c);
            let (selected, c) = pending.parts();
            if Arc::ptr_eq(
                original.shared_attributes(),
                selected.value.shared_attributes(),
            ) {
                return self.promote_bank(container, c);
            }
            return (|| {
                Self::prepare_updates(
                    &self.payloads,
                    &mut self.joins,
                    container,
                    &selected.value,
                    updates,
                    c,
                )?;
                let tickets = c.final_attribute_tickets(&selected.value)?;
                self.install_bank(container, element, selected.value.clone(), tickets, c)
            })();
        }
        // The complete bank missed. Select component backings first; allocate
        // no replacement bank unless at least one actually changes.
        let mut next = self.joins[container].head;
        while let Some(slot) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            next = self.joins[slot].next;
            let RetainedPayload::String { value, .. } = &self.payloads.values[slot] else {
                unreachable!("string join")
            };
            if let Some(selected) = pools.string(value, c)? {
                let mut pending = Pending::new(selected, c);
                let (selected, c) = pending.parts();
                (|| {
                    if !Arc::ptr_eq(
                        value.shared_code_units(),
                        selected.value.shared_code_units(),
                    ) {
                        payload_write::<Option<usize>>(c, 1)?;
                        updates
                            .push(slot, &selected.value, c)
                            .map(|index| self.joins[slot].update = Some(index))
                    } else {
                        Ok(())
                    }
                })()?;
            }
        }
        if updates.values.is_empty() {
            let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { values, .. })) =
                &self.payloads.values[container]
            else {
                unreachable!()
            };
            pools.offer_attributes(values, c)?;
            return self.promote_bank(container, c);
        }
        let RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
            values: original, ..
        })) = &self.payloads.values[container]
        else {
            unreachable!()
        };
        Self::prepare_updates(
            &self.payloads,
            &mut self.joins,
            container,
            original,
            updates,
            c,
        )?;
        // Every component clone below is already held by its matching final
        // row ticket. The old bank and original tickets remain live.
        let mut parts = Pending::new(
            Parts(OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?),
            c,
        );
        let (parts_value, c) = parts.parts();
        let parts_value = &mut parts_value.0;
        for _ in 0..original.len() {
            parts_value.try_push_geometric(c, [None; 4])?;
        }
        let mut next = self.joins[container].head;
        while let Some(slot) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            next = self.joins[slot].next;
            let RetainedPayload::String { location, .. } = self.payloads.values[slot] else {
                unreachable!("string join")
            };
            let (_, (ordinal, part)) = attribute_location(location).unwrap();
            payload_write::<Option<usize>>(c, 1)?;
            parts_value.get_mut(ordinal, c)?.unwrap()[part] = self.joins[slot].update;
        }
        let mut pending = Pending::new(BankOutput::vector(c)?, c);
        let (output, c) = pending.parts();
        c.reserve_vector(
            output.vector.as_mut().unwrap(),
            output.vector_ticket.as_mut().unwrap(),
            original.len(),
        )?;
        c.acquire(HtmlParseResource::ManagedAttributes, original.len() as u64)?;
        output.attributes = original.len() as u64;
        for ordinal in 0..original.len() {
            payload_write::<Attribute>(c, 1)?;
            let old = &original.as_slice()[ordinal];
            let component = |part: usize,
                             c: &mut ParseControl|
             -> Result<DocumentString, SessionError> {
                c.charge(WorkCategory::RecordVisit, 2)?;
                let index = parts_value.as_slice()[ordinal][part].expect("actual component row");
                Ok(updates.values[index].value.clone())
            };
            let namespace = if old.name().namespace().uri().is_some() {
                Some(component(0, c)?)
            } else {
                None
            };
            let prefix = if old.name().prefix().is_some() {
                Some(component(1, c)?)
            } else {
                None
            };
            output.vector.as_mut().unwrap().push(Attribute::new(
                QualifiedName::new(
                    namespace.map_or_else(Namespace::none, Namespace::from_uri),
                    prefix,
                    component(2, c)?,
                ),
                component(3, c)?,
            ));
        }
        match c.share_owned(output.vector.take().unwrap()) {
            Ok((backing, mut shared)) => {
                shared.promote_final();
                output.backing = Some(backing);
                output.shared = Some(shared);
            }
            Err((stop, vector)) => {
                output.vector = Some(vector);
                return Err(stop.into());
            }
        }
        match validate_parser_attributes(output.backing.take().unwrap(), c) {
            Ok(values) => output.values = Some(values),
            Err((error, backing)) => {
                output.backing = Some(backing);
                return Err(error);
            }
        }
        pools.offer_attributes(output.values.as_ref().unwrap(), c)?;
        #[cfg(test)]
        if let Some(earlier) = c.finalization_cut_earlier(element) {
            let vector_ticket = output.vector_ticket.as_ref().unwrap();
            let shared = output.shared.as_ref().unwrap();
            // Observe the actual provisional output and prior committed owner;
            // then let the next real install admission encounter the test cut.
            let earlier_promoted = self.payloads.values.iter().any(|payload| matches!(payload,
                RetainedPayload::Attributes(Some(AttributeStorage::Frozen { element: Some(index), shared, .. }))
                    if *index == earlier && shared.finalized()));
            c.cut_finalization(crate::parser::control::FinalizationObservation {
                string_pool_bytes: pools.strings.heap_usage().bytes(),
                attribute_pool_bytes: pools.attributes.heap_usage().bytes(),
                string_insertions: pools.string_insertions,
                attribute_insertions: pools.attribute_insertions,
                earlier_promoted,
                selected_bank_bytes: vector_ticket.capacity() * mem::size_of::<Attribute>()
                    + shared.payload_bytes() as usize,
                selected_components: updates.values.len(),
                selected_tickets_finalized: shared.finalized()
                    && updates
                        .values
                        .iter()
                        .all(|update| update.allocation.finalized()),
                work: c.usage().resource(HtmlParseResource::WorkCredits).current,
            });
        }
        let bank = output.values.take().unwrap();
        let tickets = (
            output.vector_ticket.take().unwrap(),
            output.shared.take().unwrap(),
        );
        self.install_bank(container, element, bank, tickets, c)
    }

    fn promote_bank(&mut self, container: usize, c: &mut ParseControl) -> Result<(), SessionError> {
        payload_write::<SharedAllocation<Vec<Attribute>>>(c, 1)?;
        let mut next = self.joins[container].head;
        while let Some(slot) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            payload_write::<DocumentStringAllocation>(c, 1)?;
            next = self.joins[slot].next;
        }
        let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { shared, .. })) =
            &mut self.payloads.values[container]
        else {
            unreachable!()
        };
        shared.promote_final();
        let mut next = self.joins[container].head;
        while let Some(slot) = next {
            next = self.joins[slot].next;
            let RetainedPayload::String { allocation, .. } = &mut self.payloads.values[slot] else {
                unreachable!()
            };
            allocation.promote_final();
        }
        Ok(())
    }

    fn install_bank(
        &mut self,
        container: usize,
        element: usize,
        bank: OrderedAttributes,
        tickets: (VectorCapacity<Attribute>, SharedAllocation<Vec<Attribute>>),
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let mut pending = Pending::new(BankOutput::selected(bank, tickets), c);
        let (output, c) = pending.parts();
        let bank = output.values.as_ref().unwrap();
        // All row moves are prepaid before the one construction commit.
        (|| {
            let mut next = self.joins[container].head;
            while let Some(slot) = next {
                c.charge(WorkCategory::RecordVisit, 1)?;
                payload_write::<RetainedPayload>(c, 2)?;
                payload_write::<Update>(c, 1)?;
                next = self.joins[slot].next;
            }
            let handle = self.construction.node_at(element).unwrap().handle();
            self.construction.apply(
                c,
                bounded::HtmlConstructionCommand::ReplaceAttributes {
                    element: handle,
                    attributes: bank.clone().into(),
                },
                &[],
            )?;
            Ok::<(), SessionError>(())
        })()?;
        let bank = output.values.take().unwrap();
        let tickets = (
            output.vector_ticket.take().unwrap(),
            output.shared.take().unwrap(),
        );
        let RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            ..
        })) = &mut self.payloads.values[container]
        else {
            unreachable!()
        };
        let old = mem::replace(values, bank);
        let old_vector = mem::replace(allocation, tickets.0);
        let old_shared = mem::replace(shared, tickets.1);
        if old_shared.finalized() {
            c.release_final_attributes(old, old_vector, old_shared);
        } else {
            let vector = c
                .release_shared(old.into_shared_attributes(), old_shared)
                .unwrap_or_else(|_| unreachable!("checked original bank"));
            let count = vector.len() as u64;
            drop(vector);
            c.release_final_vector(old_vector, count);
        }
        Ok(())
    }

    fn share_string(
        &mut self,
        slot: usize,
        pools: &mut Pools,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let RetainedPayload::String {
            value, location, ..
        } = &self.payloads.values[slot]
        else {
            return Ok(());
        };
        let location = *location;
        if !matches!(
            location,
            StringLocation::Text(_) | StringLocation::ElementName(_) | StringLocation::Namespace(_)
        ) {
            return Ok(());
        }
        let selected = pools.string(value, c)?;
        let Some(selected) = selected else {
            payload_write::<DocumentStringAllocation>(c, 1)?;
            let RetainedPayload::String { allocation, .. } = &mut self.payloads.values[slot] else {
                unreachable!()
            };
            allocation.promote_final();
            return Ok(());
        };
        let mut pending = Pending::new(
            StringOutput {
                selected,
                ticket: None,
            },
            c,
        );
        let (output, c) = pending.parts();
        if Arc::ptr_eq(
            value.shared_code_units(),
            output.selected.value.shared_code_units(),
        ) {
            payload_write::<DocumentStringAllocation>(c, 1)?;
            let RetainedPayload::String { allocation, .. } = &mut self.payloads.values[slot] else {
                unreachable!()
            };
            allocation.promote_final();
            return Ok(());
        }
        output.ticket = Some(c.final_string_ticket(&output.selected.value)?);
        self.install_string(location, &output.selected.value, c)?;
        // No admission, provider call or other fallible step separates the
        // native commit from its exact occurrence-ticket transfer.
        let RetainedPayload::String {
            value, allocation, ..
        } = &mut self.payloads.values[slot]
        else {
            unreachable!()
        };
        let old = mem::replace(value, output.selected.value.clone());
        let mut old_ticket = mem::replace(allocation, output.ticket.take().unwrap());
        old_ticket.promote_final();
        drop(old);
        c.release_document_string(old_ticket)
            .expect("terminal old occurrence");
        Ok(())
    }

    fn install_string(
        &mut self,
        location: StringLocation,
        value: &DocumentString,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        payload_write::<RetainedPayload>(c, 2)?;
        let node = match location {
            StringLocation::Text(node)
            | StringLocation::ElementName(node)
            | StringLocation::Namespace(node) => node,
            _ => unreachable!(),
        };
        let stored = self.construction.node_at(node).unwrap();
        let handle = stored.handle();
        let mut temporary = Usage::default();
        let content = if matches!(location, StringLocation::Text(_)) {
            bounded::HtmlConstructionContent::Text(value.clone())
        } else {
            let HtmlNodeKind::Element(element) = stored.kind() else {
                unreachable!("qualified name")
            };
            let name = element.name();
            let namespace = if matches!(location, StringLocation::Namespace(_)) {
                Some(value)
            } else {
                name.namespace().uri()
            };
            let local = if matches!(location, StringLocation::ElementName(_)) {
                value
            } else {
                name.local_name()
            };
            for component in [namespace, name.prefix(), Some(local)]
                .into_iter()
                .flatten()
            {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let bytes = component
                    .shared_code_units()
                    .capacity()
                    .checked_mul(mem::size_of::<u16>())
                    .and_then(|n| n.checked_add(mem::size_of::<Vec<u16>>()))
                    .ok_or_else(|| {
                        c.representation::<()>(HtmlParseResource::ManagedCapacityBytes)
                            .unwrap_err()
                    })? as u64;
                temporary.bytes = temporary.bytes.checked_add(bytes).ok_or_else(|| {
                    c.representation::<()>(HtmlParseResource::ManagedCapacityBytes)
                        .unwrap_err()
                })?;
                temporary.units = temporary
                    .units
                    .checked_add(component.len() as u64)
                    .ok_or_else(|| {
                        c.representation::<()>(HtmlParseResource::ManagedUtf16Units)
                            .unwrap_err()
                    })?;
            }
            temporary.retain(c)?;
            bounded::HtmlConstructionContent::ElementName(QualifiedName::new(
                if matches!(location, StringLocation::Namespace(_)) {
                    Namespace::from_uri(value.clone())
                } else {
                    name.namespace().clone()
                },
                name.prefix().cloned(),
                if matches!(location, StringLocation::ElementName(_)) {
                    value.clone()
                } else {
                    name.local_name().clone()
                },
            ))
        };
        let mut pending = Pending::new(
            Content {
                value: Some(content),
                usage: temporary,
            },
            c,
        );
        let (content, c) = pending.parts();
        self.construction
            .apply(
                c,
                bounded::HtmlConstructionCommand::ReplaceContent {
                    node: handle,
                    content: content.value.take().unwrap(),
                },
                &[],
            )
            .map(|_| ())
    }

    fn run(&mut self, pools: &mut Pools, c: &mut ParseControl) -> Result<(), SessionError> {
        self.make_joins(c)?;
        for slot in 0..self.payloads.values.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(self.payloads.values[slot], RetainedPayload::Attributes(_)) {
                self.share_bank(slot, pools, c)?;
            }
        }
        for slot in 0..self.payloads.values.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            self.share_string(slot, pools, c)?;
        }
        Ok(())
    }
}

fn attribute_location(location: StringLocation) -> Option<(usize, (usize, usize))> {
    match location {
        StringLocation::AttributeNamespace(container, ordinal) => Some((container, (ordinal, 0))),
        StringLocation::AttributePrefix(container, ordinal) => Some((container, (ordinal, 1))),
        StringLocation::AttributeName(container, ordinal) => Some((container, (ordinal, 2))),
        StringLocation::AttributeValue(container, ordinal) => Some((container, (ordinal, 3))),
        _ => None,
    }
}
fn attribute_component(
    bank: &OrderedAttributes,
    (ordinal, part): (usize, usize),
) -> &DocumentString {
    let attribute = &bank.as_slice()[ordinal];
    match part {
        0 => attribute.name().namespace().uri().unwrap(),
        1 => attribute.name().prefix().unwrap(),
        2 => attribute.name().local_name(),
        3 => attribute.value(),
        _ => unreachable!(),
    }
}

impl FrozenArena {
    pub(crate) fn require_original_transfer(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        c.checkpoint()?;
        if self.sharing_started {
            return Err(SessionError::Consumed);
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn arm_finalization_cut(
        &self,
        earlier: &HtmlNodeId,
        target: &HtmlNodeId,
        kind: crate::parser::control::FinalizationCut,
        c: &mut ParseControl,
    ) {
        let view = &self
            .storage
            .as_ref()
            .expect("original frozen construction")
            .view;
        let earlier = view.node_by_result_id(earlier).unwrap().position();
        let target = view.node_by_result_id(target).unwrap().position();
        c.arm_finalization_cut(target, earlier, kind);
    }
    pub(super) fn share_values<'c>(
        &mut self,
        c: &'c mut ParseControl,
        #[cfg(test)] mode: ValueSharingMode,
    ) -> Result<Sharing<'c>, SessionError> {
        if self.sharing_started || self.finalizing.is_some() {
            return Err(SessionError::Consumed);
        }
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        storage
            .construction
            .require_recovery(c, storage.lease.as_ref().unwrap())?;
        payload_write::<Finalizing>(c, 1)?;
        let root_id = match &storage.view {
            HtmlConstructionResult::Document(value) => value.root().id(),
            HtmlConstructionResult::Fragment(value) => value.root().id(),
        };
        let root = storage
            .view
            .node_by_result_id(&root_id)
            .map_err(|_| SessionError::ForeignNode)?
            .handle();
        let (joins, joins_allocation) = c.new_vector()?;
        let FrozenStorage {
            view,
            control,
            construction,
            mut lease,
            payloads,
            effects,
            evidence,
            fragment,
        } = self.storage.take().unwrap();
        let mut result = Some(view);
        let recovery = match bounded::HtmlConstructionLease::try_recover(&mut result, &mut lease) {
            Ok(value) => value,
            Err(error) => {
                release_vector(c, joins, joins_allocation)?;
                self.storage = Some(FrozenStorage {
                    view: result.take().unwrap(),
                    control,
                    construction,
                    lease,
                    payloads,
                    effects,
                    evidence,
                    fragment,
                });
                return Err(construction_owner::construction_error(c, error));
            }
        };
        self.sharing_started = true;
        self.finalizing = Some(Finalizing {
            owner: control,
            construction,
            payloads,
            effects,
            evidence,
            fragment,
            root,
            joins,
            joins_allocation,
        });
        self.finalizing
            .as_mut()
            .unwrap()
            .construction
            .recovered(c, recovery)?;
        let mut sharing = Sharing::new(
            c,
            #[cfg(test)]
            mode,
        )?;
        self.finalizing
            .as_mut()
            .unwrap()
            .run(sharing.pools.as_mut().unwrap(), sharing.control)?;
        sharing.pools.as_mut().unwrap().prepare(sharing.control)?;
        let c = &mut *sharing.control;
        let state = self.finalizing.as_mut().unwrap();
        let (empty, empty_allocation) = c.new_vector()?;
        let joins = mem::replace(&mut state.joins, empty);
        let joins_allocation = mem::replace(&mut state.joins_allocation, empty_allocation);
        release_vector(c, joins, joins_allocation).expect("owned join vector");
        let (view, lease) = state.construction.publish(c, state.root.clone())?;
        let Finalizing {
            owner,
            construction,
            payloads,
            effects,
            evidence,
            fragment,
            root,
            joins,
            joins_allocation,
        } = self.finalizing.take().unwrap();
        drop(root);
        release_vector(c, joins, joins_allocation).expect("empty owned join vector");
        self.storage = Some(FrozenStorage {
            view,
            control: owner,
            construction,
            lease: Some(lease),
            payloads,
            effects,
            evidence,
            fragment,
        });
        Ok(sharing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_cold_string_inspection(repetitions: usize, expected_units: usize) {
        let input = DocumentString::from_utf16([65, 0xd800, 0, 0xdc00].repeat(repetitions));
        let equal = DocumentString::from_utf16([65, 0xd800, 0, 0xdc00].repeat(repetitions));
        assert_eq!(input.len(), expected_units);
        assert!(!Arc::ptr_eq(
            input.shared_code_units(),
            equal.shared_code_units()
        ));
        let capacity = input.shared_code_units().capacity();
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        let inspected = {
            let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
            let pools = sharing.pools.as_mut().unwrap();
            assert_eq!(pools.strings.heap_usage().bytes(), 0);
            let before = sharing.control.usage().work(WorkCategory::InspectByte);
            assert!(pools.string(&input, sharing.control).unwrap().is_none());
            let inspected = sharing.control.usage().work(WorkCategory::InspectByte) - before;
            assert!(sharing.control.stop().is_none());
            assert!(pools.strings.heap_usage().bytes() > 0);
            assert_eq!(pools.attributes.heap_usage().bytes(), 0);

            // A subsequent independently backed equal operand must really hit
            // the offered original, so an early bypass cannot satisfy the cut.
            let selected = pools
                .string(&equal, sharing.control)
                .unwrap()
                .expect("cold miss retained an eligible string");
            assert!(Arc::ptr_eq(
                selected.value.shared_code_units(),
                input.shared_code_units()
            ));
            assert_eq!(selected.value.len(), expected_units);
            for chunk in selected.value.code_units().as_chunks::<4>().0 {
                assert_eq!(chunk, &[65, 0xd800, 0, 0xdc00]);
            }
            selected.release(sharing.control);
            inspected
        };
        // This is the same original controller and real paired-root Drop,
        // including pool data-before-credit cleanup; work is never refunded.
        super::super::tests::storage_zero(&c);
        assert!(c.stop().is_none());
        assert_eq!(input.shared_code_units().capacity(), capacity);
        assert_eq!(input.len(), expected_units);
        assert_eq!(equal.len(), expected_units);
        for value in [&input, &equal] {
            for chunk in value.code_units().as_chunks::<4>().0 {
                assert_eq!(chunk, &[65, 0xd800, 0, 0xdc00]);
            }
        }
        // On an empty pool there are no candidate payload comparisons. The
        // successful input hash admits two InspectByte credits per raw UTF-16
        // unit. Promoting its qualified immutable miss must not hash it again.
        // This observes original-controller work, not CPU or allocator reads.
        assert_eq!(inspected, (expected_units * 2) as u64);
    }

    #[test]
    fn cold_string_miss_charges_one_inspection_of_nonempty_raw_utf16() {
        assert_cold_string_inspection(1, 4);
    }

    #[test]
    fn cold_string_miss_charges_one_inspection_of_longer_raw_utf16() {
        assert_cold_string_inspection(17, 68);
    }

    #[test]
    fn second_pool_prepare_refusal_retains_both_abort_authorities_and_releases_custody() {
        // This is the concrete consumer guard over the producers' real public
        // mark/freeze protocol, rather than a synthetic prepare error.
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        let text = DocumentString::from("value");
        let bank = OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "class"),
            "value",
        )])
        .unwrap();
        {
            let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
            let pools = sharing.pools.as_mut().unwrap();
            assert!(pools.string(&text, sharing.control).unwrap().is_none());
            pools.offer_attributes(&bank, sharing.control).unwrap();
            assert!(pools.strings.heap_usage().bytes() > 0);
            assert!(pools.attributes.heap_usage().bytes() > 0);
            work(sharing.control, HtmlAttributePool::mark_work()).unwrap();
            let Authority::Open(root) = &pools.attributes_root else {
                panic!("open pair")
            };
            let _outstanding = pools
                .attributes
                .mark(root)
                .unwrap()
                .expect("real nested authority");
            let failure = pools.prepare(sharing.control).unwrap_err();
            assert!(matches!(
                failure,
                SessionError::Stop(ParseStop::InternalInvariant(
                    crate::parser::control::Invariant::CapacityMismatch
                ))
            ));
            assert!(matches!(pools.strings_root, Authority::Prepared(_)));
            assert!(matches!(pools.attributes_root, Authority::Open(_)));
            // Guard Drop aborts the prepared first root and the still-open
            // second root, including its outstanding nested mark.
        }
        super::super::tests::storage_zero(&c);
        assert!(matches!(
            c.stop(),
            Some(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::CapacityMismatch
            ))
        ));
        assert_eq!(text.code_units(), &[118, 97, 108, 117, 101]);
        assert_eq!(bank.as_slice()[0].value(), &text);
    }
}

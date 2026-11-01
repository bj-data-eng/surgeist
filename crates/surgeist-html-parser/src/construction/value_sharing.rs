//! Terminal report promotion. No mutable parser session can acquire these pools.
use super::*;
use crate::parser::control::{Invariant, SharedReleaseError};
#[cfg(test)]
use crate::request::ValueSharingMode;
use surgeist_html::bounded::{
    HtmlAttributePool, HtmlAttributePoolOperation, HtmlAttributePoolResult, HtmlStringPool,
    HtmlStringPoolOperation, HtmlStringPoolResult, HtmlValuePoolActionKind, HtmlValuePoolError,
    HtmlValuePoolProgress, HtmlValuePoolRoot, HtmlValuePoolSeal, HtmlValuePoolUsage,
    HtmlValuePoolWork,
};

/// The stop for a pool that refused a protocol step or ran out of
/// representation, latched as the controller's first stop.
fn pool_stop(c: &mut ParseControl, error: HtmlValuePoolError) -> ParseStop {
    if matches!(error, HtmlValuePoolError::RepresentationExhausted) {
        c.stop_representation(HtmlParseResource::ManagedCapacityBytes)
    } else {
        c.fail_invariant(Invariant::CapacityMismatch)
    }
}
fn pool_error(c: &mut ParseControl, error: HtmlValuePoolError) -> SessionError {
    pool_stop(c, error).into()
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
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        c.release_retained_provider(&c.identity(), self.bytes, self.units, 0)
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
        removed.release(c)?;
        *self = target;
        Ok(())
    }
}
/// A scoped terminal value never outlives its exact original-controller credit.
/// The disjoint borrow lets useful work observe the value while Drop retains
/// exclusive cleanup authority on both refusal and unwind.
trait Temporary {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop>;
}
impl Temporary for Usage {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        Usage::release(self, c)
    }
}
/// Teardown that runs on `Drop`, which has no result. A refused credit is
/// already latched on the controller, whose next checkpoint reports it, and the
/// refused ticket is dropped with its value.
fn latched(result: Result<(), ParseStop>) {
    let _ = result;
}
/// Owns one temporary until it is handed on. Only [`Self::take`] empties the
/// guard, and nothing reads the guard after it, so the accessors' refusal of an
/// emptied guard reports a state no caller can reach.
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
    fn parts(&mut self) -> Result<(&mut T, &mut ParseControl), SessionError> {
        match self.value.as_mut() {
            Some(value) => Ok((value, &mut *self.control)),
            None => Err(latch_registry_row(self.control)),
        }
    }
    /// Moves the temporary through a transition that takes it by value. The
    /// guard holds it again afterwards, advanced or, on a refusal, unchanged.
    fn advance(
        &mut self,
        step: impl FnOnce(T, &mut ParseControl) -> Result<T, (SessionError, T)>,
    ) -> Result<(), SessionError> {
        let value = self.take()?;
        match step(value, &mut *self.control) {
            Ok(value) => {
                self.value = Some(value);
                Ok(())
            }
            Err((error, value)) => {
                self.value = Some(value);
                Err(error)
            }
        }
    }
    /// Hands the temporary to its next owner; the guard is then empty.
    fn take(&mut self) -> Result<T, SessionError> {
        match self.value.take() {
            Some(value) => Ok(value),
            None => Err(latch_registry_row(self.control)),
        }
    }
}
impl<T: Temporary> Drop for Pending<'_, T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            latched(value.release(self.control));
        }
    }
}
struct Selected<T> {
    value: T,
    usage: Usage,
}
impl<T> Temporary for Selected<T> {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let Self { value, usage } = self;
        drop(value);
        usage.release(c)
    }
}
// Authority is affine and remains inside the same controller-borrowing guard
// until the receiver has admitted its final transfer and polled cancellation.
enum Authority {
    Closed,
    Open(HtmlValuePoolRoot),
    Prepared(HtmlValuePoolSeal),
}
impl Authority {
    /// The open root, leaving any other authority where it was.
    fn take_open(&mut self) -> Option<HtmlValuePoolRoot> {
        match mem::replace(self, Self::Closed) {
            Self::Open(root) => Some(root),
            other => {
                *self = other;
                None
            }
        }
    }
}
pub(super) struct Sharing<'c> {
    /// Owned until [`Self::commit`] consumes the guard, so only a guard that is
    /// already gone ever holds `None`.
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
        let (pools, control) = result.parts()?;
        pools.open(control)?;
        Ok(result)
    }
    /// The pools and the controller they are charged to.
    fn parts(&mut self) -> Result<(&mut Pools, &mut ParseControl), SessionError> {
        match self.pools.as_mut() {
            Some(pools) => Ok((pools, &mut *self.control)),
            None => Err(latch_registry_row(self.control)),
        }
    }
    pub(super) fn control(&mut self) -> &mut ParseControl {
        #[cfg(test)]
        if let Some(pools) = &self.pools {
            pools.observe_transfer(
                crate::parser::control::FinalizationTransferStage::Receiver,
                self.control,
            );
        }
        self.control
    }
    pub(super) fn commit(mut self) -> Result<(), ParseStop> {
        let Some(mut pools) = self.pools.take() else {
            return Err(self.control.fail_invariant(Invariant::RegistryRow));
        };
        // Both credits are released even when a pool refuses its seal; the
        // first refusal is reported.
        let sealed = pools.seal().map_err(|error| pool_stop(self.control, error));
        let released = pools.release(self.control);
        sealed.and(released)
    }
}
impl Drop for Sharing<'_> {
    fn drop(&mut self) {
        if let Some(mut pools) = self.pools.take() {
            latched(
                pools
                    .abort()
                    .map_err(|error| pool_stop(self.control, error)),
            );
            // Pools release their actual data before each matching credit.
            // Disposal/root/action closing work was admitted before retention.
            latched(pools.release(self.control));
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
/// The operation a driver still owns. Only `try_finish` takes it, and it ends
/// the driver's loop, so an empty slot is a pool protocol violation.
fn live<'o, O>(owner: &'o mut Option<O>, c: &mut ParseControl) -> Result<&'o mut O, SessionError> {
    match owner.as_mut() {
        Some(operation) => Ok(operation),
        None => Err(pool_error(c, HtmlValuePoolError::NotReady)),
    }
}
/// Settles a pool's final extent after its operation. A refusal keeps the
/// operation's own stop when it has one; otherwise a selected value is handed
/// back with its candidate credit.
fn settle<T: Temporary>(
    result: Result<Option<T>, SessionError>,
    adjusted: Result<(), SessionError>,
    c: &mut ParseControl,
) -> Result<Option<T>, SessionError> {
    match (result, adjusted) {
        (Ok(selected), Ok(())) => Ok(selected),
        (Ok(selected), Err(error)) => {
            if let Some(selected) = selected {
                latched(selected.release(c));
            }
            Err(error)
        }
        (Err(error), _) => Err(error),
    }
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
            let (state, control) = pending.parts()?;
            let owner = &mut state.owner;
            let candidate = &mut state.candidate;
            let result = (|| {
                while let Some(action) = self
                    .$pool
                    .next_action(live(owner, control)?)
                    .map_err(|e| pool_error(control, e))?
                {
                    let layout = action.layout();
                    work(control, layout.work())?;
                    work(control, layout.closing_work())?;
                    self.$usage.grow(Usage::pool(layout.pool()), control)?;
                    candidate.grow(Usage::pool(layout.candidate()), control)?;
                    let progress = match action.kind() {
                        HtmlValuePoolActionKind::Read => live(owner, control)?.read(action),
                        HtmlValuePoolActionKind::Pool => {
                            self.$pool.perform(live(owner, control)?, action)
                        }
                    }
                    .map_err(|e| pool_error(control, e))?;
                    if progress == HtmlValuePoolProgress::CandidateReleased {
                        candidate.release(control)?;
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
            let adjusted = self.$usage.adjust(Usage::pool(self.$pool.heap_usage()), c);
            settle(result, adjusted, c)
        }
    };
}
struct StringOperation<'a> {
    owner: Option<HtmlStringPoolOperation<'a>>,
    candidate: Usage,
}
impl Temporary for StringOperation<'_> {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        drop(self.owner);
        self.candidate.release(c)
    }
}
struct AttributeOperation<'a> {
    owner: Option<HtmlAttributePoolOperation<'a>>,
    candidate: Usage,
}
impl Temporary for AttributeOperation<'_> {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        drop(self.owner);
        self.candidate.release(c)
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
            hit.release(c)?;
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
            self.abort().map_err(|error| pool_error(c, error))?;
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
        // Both roots were open when this began and nothing since has moved
        // them, so a root that is not open breaks the pool protocol.
        let Some(root) = self.strings_root.take_open() else {
            return Err(pool_error(c, HtmlValuePoolError::WrongAction));
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
        let Some(root) = self.attributes_root.take_open() else {
            return Err(pool_error(c, HtmlValuePoolError::WrongAction));
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
    /// Seals both prepared plans. Each is attempted; the first refusal is
    /// returned.
    fn seal(&mut self) -> Result<(), HtmlValuePoolError> {
        let mut refused = Ok(());
        if let Authority::Prepared(seal) = mem::replace(&mut self.strings_root, Authority::Closed) {
            refused = self.strings.seal(seal).map(|_| ());
        }
        if let Authority::Prepared(seal) =
            mem::replace(&mut self.attributes_root, Authority::Closed)
        {
            let sealed = self.attributes.seal(seal).map(|_| ());
            refused = refused.and(sealed);
        }
        refused
    }
    /// Aborts whatever authority each pool still holds. Each is attempted; the
    /// first refusal is returned.
    fn abort(&mut self) -> Result<(), HtmlValuePoolError> {
        let strings = match mem::replace(&mut self.strings_root, Authority::Closed) {
            Authority::Open(root) => self.strings.abort_root(root).map(|_| ()),
            Authority::Prepared(seal) => self.strings.abort_seal(seal).map(|_| ()),
            Authority::Closed => Ok(()),
        };
        let attributes = match mem::replace(&mut self.attributes_root, Authority::Closed) {
            Authority::Open(root) => self.attributes.abort_root(root).map(|_| ()),
            Authority::Prepared(seal) => self.attributes.abort_seal(seal).map(|_| ()),
            Authority::Closed => Ok(()),
        };
        strings.and(attributes)
    }
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let Self {
            strings,
            attributes,
            strings_usage,
            attributes_usage,
            ..
        } = self;
        drop(strings);
        drop(attributes);
        // Every credit is attempted; the first refusal is reported.
        let strings = strings_usage.release(c);
        let attributes = attributes_usage.release(c);
        let envelope = Usage {
            bytes: mem::size_of::<Self>() as u64,
            units: 0,
        }
        .release(c);
        strings.and(attributes).and(envelope)
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
/// The join of the registry row at `slot`. The table is built with one join per
/// row, so a slot past it is an impossible registry state.
fn join_at<'a>(
    joins: &'a [Join],
    c: &mut ParseControl,
    slot: usize,
) -> Result<&'a Join, SessionError> {
    match joins.get(slot) {
        Some(join) => Ok(join),
        None => Err(latch_registry_row(c)),
    }
}
/// As [`join_at`], for editing.
fn join_at_mut<'a>(
    joins: &'a mut [Join],
    c: &mut ParseControl,
    slot: usize,
) -> Result<&'a mut Join, SessionError> {
    match joins.get_mut(slot) {
        Some(join) => Ok(join),
        None => Err(latch_registry_row(c)),
    }
}
/// The string value and location of the row at `slot`.
fn string_row<'a>(
    payloads: &'a PayloadStorage,
    c: &mut ParseControl,
    slot: usize,
) -> Result<(&'a DocumentString, StringLocation), SessionError> {
    match payloads.row(c, slot)? {
        RetainedPayload::String {
            value, location, ..
        } => Ok((value, *location)),
        _ => Err(latch_registry_row(c)),
    }
}
/// The string value and occurrence ticket of the row at `slot`, for editing.
fn string_row_mut<'a>(
    payloads: &'a mut PayloadStorage,
    c: &mut ParseControl,
    slot: usize,
) -> Result<(&'a mut DocumentString, &'a mut DocumentStringAllocation), SessionError> {
    match payloads.row_mut(c, slot)? {
        RetainedPayload::String {
            value, allocation, ..
        } => Ok((value, allocation)),
        _ => Err(latch_registry_row(c)),
    }
}
/// Promotes the occurrence ticket of the string row at `slot` to final.
fn promote_string(
    payloads: &mut PayloadStorage,
    c: &mut ParseControl,
    slot: usize,
) -> Result<(), SessionError> {
    string_row_mut(payloads, c, slot)?.1.promote_final();
    Ok(())
}
type BankRow<'a> = (
    &'a mut OrderedAttributes,
    &'a mut VectorCapacity<Attribute>,
    &'a mut SharedAllocation<Vec<Attribute>>,
);
/// The frozen bank and both of its tickets in the row at `slot`.
fn frozen_bank_mut<'a>(
    payloads: &'a mut PayloadStorage,
    c: &mut ParseControl,
    slot: usize,
) -> Result<BankRow<'a>, SessionError> {
    match payloads.row_mut(c, slot)? {
        RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            ..
        })) => Ok((values, allocation, shared)),
        _ => Err(latch_registry_row(c)),
    }
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
    /// Moves every prepared value into its row. A row that is not a string
    /// releases its prepared value instead; every credit is attempted and the
    /// first refusal is returned.
    fn commit(
        mut self,
        payloads: &mut PayloadStorage,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let mut refused = None;
        while let Some(Update {
            slot,
            value: new,
            allocation: new_allocation,
        }) = self.values.pop()
        {
            match string_row_mut(payloads, c, slot) {
                Ok((value, allocation)) => {
                    let old = mem::replace(value, new);
                    let mut ticket = mem::replace(allocation, new_allocation);
                    // This terminal transition retires an occurrence, without claiming
                    // that a shared target was physically deallocated.
                    ticket.promote_final();
                    drop(old);
                    keep_first(&mut refused, c.release_document_string(ticket));
                }
                Err(error) => {
                    refused.get_or_insert(error);
                    drop(new);
                    keep_first(&mut refused, c.release_document_string(new_allocation));
                }
            }
        }
        keep_first(
            &mut refused,
            release_vector(c, self.values, self.allocation),
        );
        refused.map_or(Ok(()), Err)
    }
}
impl Temporary for Updates {
    fn release(mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let mut refused = Ok(());
        while let Some(Update {
            value, allocation, ..
        }) = self.values.pop()
        {
            drop(value);
            refused = refused.and(c.release_document_string(allocation));
        }
        let vector = c
            .release_vector(self.values, self.allocation)
            .map_err(|error| error.into_parts().0);
        refused.and(vector)
    }
}
/// The attribute fields that hold shared strings.
#[derive(Clone, Copy)]
enum Component {
    Namespace,
    Prefix,
    Name,
    Value,
}
/// The update row of each shared string of one attribute.
#[derive(Clone, Copy, Default)]
struct ComponentRows {
    namespace: Option<usize>,
    prefix: Option<usize>,
    name: Option<usize>,
    value: Option<usize>,
}
// Charges and layouts follow the record size, which is unchanged.
const _: () = assert!(mem::size_of::<ComponentRows>() == mem::size_of::<[Option<usize>; 4]>());
impl ComponentRows {
    fn get(self, component: Component) -> Option<usize> {
        match component {
            Component::Namespace => self.namespace,
            Component::Prefix => self.prefix,
            Component::Name => self.name,
            Component::Value => self.value,
        }
    }
    fn set(&mut self, component: Component, row: Option<usize>) {
        match component {
            Component::Namespace => self.namespace = row,
            Component::Prefix => self.prefix = row,
            Component::Name => self.name = row,
            Component::Value => self.value = row,
        }
    }
}
struct Parts(OwnedVec<ComponentRows>);
impl Temporary for Parts {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.0.release(c)
    }
}
/// The staging of one output bank: vector -> shared backing -> validated bank.
/// Every callee receives its actual data by move; on unwind that callee destroys
/// it before this caller's tickets are retired.
enum BankStage {
    Building {
        vector: Vec<Attribute>,
        ticket: VectorCapacity<Attribute>,
    },
    Shared {
        backing: Arc<Vec<Attribute>>,
        ticket: VectorCapacity<Attribute>,
        shared: SharedAllocation<Vec<Attribute>>,
    },
    Validated {
        values: OrderedAttributes,
        ticket: VectorCapacity<Attribute>,
        shared: SharedAllocation<Vec<Attribute>>,
    },
}
/// The tickets that account for one attribute bank.
type BankTickets = (VectorCapacity<Attribute>, SharedAllocation<Vec<Attribute>>);
struct BankOutput {
    stage: BankStage,
    attributes: u64,
}
impl BankOutput {
    fn vector(c: &mut ParseControl) -> Result<Self, SessionError> {
        let (vector, ticket) = c.new_vector()?;
        Ok(Self {
            stage: BankStage::Building { vector, ticket },
            attributes: 0,
        })
    }
    fn selected(values: OrderedAttributes, tickets: BankTickets) -> Self {
        let attributes = values.len() as u64;
        Self {
            stage: BankStage::Validated {
                values,
                ticket: tickets.0,
                shared: tickets.1,
            },
            attributes,
        }
    }
    fn reserve(&mut self, c: &mut ParseControl, additional: usize) -> Result<(), SessionError> {
        match &mut self.stage {
            BankStage::Building { vector, ticket } => {
                c.reserve_vector(vector, ticket, additional)?;
                Ok(())
            }
            _ => Err(latch_registry_row(c)),
        }
    }
    fn push(&mut self, attribute: Attribute, c: &mut ParseControl) -> Result<(), SessionError> {
        match &mut self.stage {
            BankStage::Building { vector, .. } => {
                vector.push(attribute);
                Ok(())
            }
            _ => Err(latch_registry_row(c)),
        }
    }
    /// Moves the staged vector behind its shared allocation. A refusal hands the
    /// output back with the vector still in it.
    fn share(self, c: &mut ParseControl) -> Result<Self, (SessionError, Self)> {
        let Self { stage, attributes } = self;
        match stage {
            BankStage::Building { vector, ticket } => match c.share_owned(vector) {
                Ok((backing, mut shared)) => {
                    shared.promote_final();
                    Ok(Self {
                        stage: BankStage::Shared {
                            backing,
                            ticket,
                            shared,
                        },
                        attributes,
                    })
                }
                Err((stop, vector)) => Err((
                    stop.into(),
                    Self {
                        stage: BankStage::Building { vector, ticket },
                        attributes,
                    },
                )),
            },
            stage => Err((latch_registry_row(c), Self { stage, attributes })),
        }
    }
    /// Validates the shared backing as a bank. A refusal hands the output back
    /// with the backing still in it.
    fn validate(self, c: &mut ParseControl) -> Result<Self, (SessionError, Self)> {
        let Self { stage, attributes } = self;
        match stage {
            BankStage::Shared {
                backing,
                ticket,
                shared,
            } => match validate_parser_attributes(backing, c) {
                Ok(values) => Ok(Self {
                    stage: BankStage::Validated {
                        values,
                        ticket,
                        shared,
                    },
                    attributes,
                }),
                Err((error, backing)) => Err((
                    error,
                    Self {
                        stage: BankStage::Shared {
                            backing,
                            ticket,
                            shared,
                        },
                        attributes,
                    },
                )),
            },
            stage => Err((latch_registry_row(c), Self { stage, attributes })),
        }
    }
    fn values(&self, c: &mut ParseControl) -> Result<&OrderedAttributes, SessionError> {
        match &self.stage {
            BankStage::Validated { values, .. } => Ok(values),
            _ => Err(latch_registry_row(c)),
        }
    }
    /// The validated bank and its tickets. Any other stage is released.
    fn into_validated(
        self,
        c: &mut ParseControl,
    ) -> Result<(OrderedAttributes, BankTickets), SessionError> {
        match self.stage {
            BankStage::Validated {
                values,
                ticket,
                shared,
            } => Ok((values, (ticket, shared))),
            stage => {
                let error = latch_registry_row(c);
                latched(
                    Self {
                        stage,
                        attributes: self.attributes,
                    }
                    .release(c),
                );
                Err(error)
            }
        }
    }
}
impl Temporary for BankOutput {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let (shared, ticket) = match self.stage {
            BankStage::Building { vector, ticket } => {
                drop(vector);
                (None, Some(ticket))
            }
            BankStage::Shared {
                backing,
                ticket,
                shared,
            } => {
                drop(backing);
                (Some(shared), Some(ticket))
            }
            BankStage::Validated {
                values,
                ticket,
                shared,
            } => {
                drop(values);
                (Some(shared), Some(ticket))
            }
        };
        // Both credits are attempted; the first refusal is reported.
        let shared = match shared {
            Some(shared) => c
                .release_final_shared(shared)
                .map_err(FinalReleaseError::discard),
            None => Ok(()),
        };
        let vector = match ticket {
            Some(ticket) => c
                .release_final_vector(ticket, self.attributes)
                .map_err(FinalReleaseError::discard),
            None => Ok(()),
        };
        shared.and(vector)
    }
}
/// Drops a refused shared release's payload and ticket, for a caller with
/// nowhere to keep them; the stop stays latched and reports the skipped credit.
fn discard_shared<T>(error: SharedReleaseError<T>) -> ParseStop {
    let (stop, shared, allocation) = error.into_parts();
    drop(shared);
    drop(allocation);
    stop
}
struct StringOutput {
    selected: Selected<DocumentString>,
    ticket: Option<DocumentStringAllocation>,
}
impl Temporary for StringOutput {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let ticket = match self.ticket {
            Some(ticket) => c.release_document_string(ticket),
            None => Ok(()),
        };
        ticket.and(self.selected.release(c))
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
                let head = join_at(&self.joins, c, container)?.head;
                join_at_mut(&mut self.joins, c, slot)?.next = head;
                join_at_mut(&mut self.joins, c, container)?.head = Some(slot);
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
        let mut next = join_at(joins, c, container)?.head;
        while let Some(slot) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let join = join_at(joins, c, slot)?;
            next = join.next;
            if join.update.is_none() {
                let (_, location) = string_row(payloads, c, slot)?;
                let Some((_, component)) = attribute_location(location) else {
                    return Err(latch_registry_row(c));
                };
                payload_write::<Option<usize>>(c, 1)?;
                let value = attribute_component(bank, component, c)?;
                let update = updates.push(slot, value, c)?;
                join_at_mut(joins, c, slot)?.update = Some(update);
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
        let (updates, c) = pending.parts()?;
        self.share_bank_inner(container, pools, updates, c)?;
        let updates = pending.take()?;
        updates.commit(&mut self.payloads, &mut *pending.control)
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
        })) = self.payloads.row(c, container)?
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
            let (selected, c) = pending.parts()?;
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
        let mut next = join_at(&self.joins, c, container)?.head;
        while let Some(slot) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            next = join_at(&self.joins, c, slot)?.next;
            let (value, _) = string_row(&self.payloads, c, slot)?;
            if let Some(selected) = pools.string(value, c)? {
                let mut pending = Pending::new(selected, c);
                let (selected, c) = pending.parts()?;
                if !Arc::ptr_eq(
                    value.shared_code_units(),
                    selected.value.shared_code_units(),
                ) {
                    payload_write::<Option<usize>>(c, 1)?;
                    let index = updates.push(slot, &selected.value, c)?;
                    join_at_mut(&mut self.joins, c, slot)?.update = Some(index);
                }
            }
        }
        if updates.values.is_empty() {
            pools.offer_attributes(original, c)?;
            return self.promote_bank(container, c);
        }
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
        let (parts_value, c) = parts.parts()?;
        let parts_value = &mut parts_value.0;
        // The count is known, but a single reserve would precede the logical
        // admission of the first row; each push keeps its own order.
        for _ in 0..original.len() {
            parts_value.try_push_geometric_toward(c, ComponentRows::default(), original.len())?;
        }
        let mut next = join_at(&self.joins, c, container)?.head;
        while let Some(slot) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let join = join_at(&self.joins, c, slot)?;
            next = join.next;
            let update = join.update;
            let (_, location) = string_row(&self.payloads, c, slot)?;
            let Some((_, (ordinal, component))) = attribute_location(location) else {
                return Err(latch_registry_row(c));
            };
            payload_write::<Option<usize>>(c, 1)?;
            match parts_value.get_mut(ordinal, c)? {
                Some(rows) => rows.set(component, update),
                None => return Err(latch_registry_row(c)),
            }
        }
        let mut pending = Pending::new(BankOutput::vector(c)?, c);
        let (output, c) = pending.parts()?;
        output.reserve(c, original.len())?;
        c.acquire(HtmlParseResource::ManagedAttributes, original.len() as u64)?;
        output.attributes = original.len() as u64;
        for (ordinal, old) in original.as_slice().iter().enumerate() {
            payload_write::<Attribute>(c, 1)?;
            let component = |component: Component,
                             c: &mut ParseControl|
             -> Result<DocumentString, SessionError> {
                c.charge(WorkCategory::RecordVisit, 2)?;
                let Some(index) = parts_value
                    .as_slice()
                    .get(ordinal)
                    .and_then(|rows| rows.get(component))
                else {
                    return Err(latch_registry_row(c));
                };
                match updates.values.get(index) {
                    Some(update) => Ok(update.value.clone()),
                    None => Err(latch_registry_row(c)),
                }
            };
            let namespace = if old.name().namespace().uri().is_some() {
                Some(component(Component::Namespace, c)?)
            } else {
                None
            };
            let prefix = if old.name().prefix().is_some() {
                Some(component(Component::Prefix, c)?)
            } else {
                None
            };
            let attribute = Attribute::new(
                QualifiedName::new(
                    namespace.map_or_else(Namespace::none, Namespace::from_uri),
                    prefix,
                    component(Component::Name, c)?,
                ),
                component(Component::Value, c)?,
            );
            output.push(attribute, c)?;
        }
        pending.advance(BankOutput::share)?;
        pending.advance(BankOutput::validate)?;
        let (output, c) = pending.parts()?;
        pools.offer_attributes(output.values(c)?, c)?;
        #[cfg(test)]
        if let Some(earlier) = c.finalization_cut_earlier(element)
            && let BankStage::Validated { ticket, shared, .. } = &output.stage
        {
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
                selected_bank_bytes: ticket.capacity() * mem::size_of::<Attribute>()
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
        let output = pending.take()?;
        let (bank, tickets) = output.into_validated(&mut *pending.control)?;
        self.install_bank(container, element, bank, tickets, &mut *pending.control)
    }

    fn promote_bank(&mut self, container: usize, c: &mut ParseControl) -> Result<(), SessionError> {
        payload_write::<SharedAllocation<Vec<Attribute>>>(c, 1)?;
        let mut next = join_at(&self.joins, c, container)?.head;
        while let Some(slot) = next {
            c.charge(WorkCategory::RecordVisit, 1)?;
            payload_write::<DocumentStringAllocation>(c, 1)?;
            next = join_at(&self.joins, c, slot)?.next;
        }
        let (_, _, shared) = frozen_bank_mut(&mut self.payloads, c, container)?;
        shared.promote_final();
        let mut next = join_at(&self.joins, c, container)?.head;
        while let Some(slot) = next {
            next = join_at(&self.joins, c, slot)?.next;
            promote_string(&mut self.payloads, c, slot)?;
        }
        Ok(())
    }

    fn install_bank(
        &mut self,
        container: usize,
        element: usize,
        bank: OrderedAttributes,
        tickets: BankTickets,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let mut pending = Pending::new(BankOutput::selected(bank, tickets), c);
        let (output, c) = pending.parts()?;
        let bank = output.values(c)?;
        // The row is found before anything is committed, so a missing one
        // leaves the output with its guard.
        let (values, allocation, shared) = frozen_bank_mut(&mut self.payloads, c, container)?;
        // All row moves are prepaid before the one construction commit.
        (|| {
            let mut next = join_at(&self.joins, c, container)?.head;
            while let Some(slot) = next {
                c.charge(WorkCategory::RecordVisit, 1)?;
                payload_write::<RetainedPayload>(c, 2)?;
                payload_write::<Update>(c, 1)?;
                next = join_at(&self.joins, c, slot)?.next;
            }
            let handle = match self.construction.node_at(element) {
                Some(record) => record.handle(),
                None => return Err(latch_registry_row(c)),
            };
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
        let output = pending.take()?;
        let (bank, tickets) = output.into_validated(&mut *pending.control)?;
        let c = &mut *pending.control;
        let old = mem::replace(values, bank);
        let old_vector = mem::replace(allocation, tickets.0);
        let old_shared = mem::replace(shared, tickets.1);
        if old_shared.finalized() {
            c.release_final_attributes(old, old_vector, old_shared)
                .map_err(FinalReleaseError::discard)?;
        } else {
            let vector = c
                .release_shared(old.into_shared_attributes(), old_shared)
                .map_err(discard_shared)?;
            let count = vector.len() as u64;
            drop(vector);
            c.release_final_vector(old_vector, count)
                .map_err(FinalReleaseError::discard)?;
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
        } = self.payloads.row(c, slot)?
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
            return promote_string(&mut self.payloads, c, slot);
        };
        let mut pending = Pending::new(
            StringOutput {
                selected,
                ticket: None,
            },
            c,
        );
        let (output, c) = pending.parts()?;
        if Arc::ptr_eq(
            value.shared_code_units(),
            output.selected.value.shared_code_units(),
        ) {
            payload_write::<DocumentStringAllocation>(c, 1)?;
            return promote_string(&mut self.payloads, c, slot);
        }
        output.ticket = Some(c.final_string_ticket(&output.selected.value)?);
        self.install_string(location, &output.selected.value, c)?;
        // No admission, provider call or other fallible step separates the
        // native commit from its exact occurrence-ticket transfer.
        let (value, allocation) = string_row_mut(&mut self.payloads, c, slot)?;
        let Some(new_ticket) = output.ticket.take() else {
            return Err(latch_registry_row(c));
        };
        let old = mem::replace(value, output.selected.value.clone());
        let mut old_ticket = mem::replace(allocation, new_ticket);
        old_ticket.promote_final();
        drop(old);
        c.release_document_string(old_ticket)?;
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
            _ => return Err(latch_registry_row(c)),
        };
        let Some(stored) = self.construction.node_at(node) else {
            return Err(latch_registry_row(c));
        };
        let handle = stored.handle();
        let mut temporary = Usage::default();
        let content = if matches!(location, StringLocation::Text(_)) {
            bounded::HtmlConstructionContent::Text(value.clone())
        } else {
            let HtmlNodeKind::Element(element) = stored.kind() else {
                return Err(latch_registry_row(c));
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
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::ManagedCapacityBytes))?
                    as u64;
                temporary.bytes = temporary.bytes.checked_add(bytes).ok_or_else(|| {
                    c.stop_representation(HtmlParseResource::ManagedCapacityBytes)
                })?;
                temporary.units = temporary
                    .units
                    .checked_add(component.len() as u64)
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
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
        // The content moves into the command, which owns it on every outcome;
        // the guard keeps only its temporary credit.
        let mut pending = Pending::new(temporary, c);
        let (_, c) = pending.parts()?;
        self.construction
            .apply(
                c,
                bounded::HtmlConstructionCommand::ReplaceContent {
                    node: handle,
                    content,
                },
                &[],
            )
            .map(|_| ())
    }

    fn run(&mut self, pools: &mut Pools, c: &mut ParseControl) -> Result<(), SessionError> {
        self.make_joins(c)?;
        for slot in 0..self.payloads.values.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(self.payloads.row(c, slot)?, RetainedPayload::Attributes(_)) {
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

/// The container row and the attribute component a string row is bound to.
fn attribute_location(location: StringLocation) -> Option<(usize, (usize, Component))> {
    match location {
        StringLocation::AttributeNamespace(container, ordinal) => {
            Some((container, (ordinal, Component::Namespace)))
        }
        StringLocation::AttributePrefix(container, ordinal) => {
            Some((container, (ordinal, Component::Prefix)))
        }
        StringLocation::AttributeName(container, ordinal) => {
            Some((container, (ordinal, Component::Name)))
        }
        StringLocation::AttributeValue(container, ordinal) => {
            Some((container, (ordinal, Component::Value)))
        }
        _ => None,
    }
}
/// The shared string of one attribute of `bank`. An attribute or component the
/// bank lacks is an impossible registry state.
fn attribute_component<'a>(
    bank: &'a OrderedAttributes,
    (ordinal, component): (usize, Component),
    c: &mut ParseControl,
) -> Result<&'a DocumentString, SessionError> {
    let Some(attribute) = bank.as_slice().get(ordinal) else {
        return Err(latch_registry_row(c));
    };
    let value = match component {
        Component::Namespace => attribute.name().namespace().uri(),
        Component::Prefix => attribute.name().prefix(),
        Component::Name => Some(attribute.name().local_name()),
        Component::Value => Some(attribute.value()),
    };
    match value {
        Some(value) => Ok(value),
        None => Err(latch_registry_row(c)),
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
    /// Moves the original storage into its terminal finalizing owner. A refusal
    /// leaves the storage, or the owner that already holds it, for release.
    fn begin_finalizing(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        if self.sharing_started || self.finalizing.is_some() {
            return Err(SessionError::Consumed);
        }
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        // A missing lease is refused before anything is taken, so the owner
        // stays where it was.
        let Some(publication) = storage.lease.as_ref() else {
            return Err(latch_registry_row(c));
        };
        storage.construction.require_recovery(c, publication)?;
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
        let Some(FrozenStorage {
            view,
            control,
            construction,
            mut lease,
            payloads,
            effects,
            evidence,
            fragment,
        }) = self.storage.take()
        else {
            release_vector(c, joins, joins_allocation)?;
            return Err(SessionError::Consumed);
        };
        let mut result = Some(view);
        let recovery = match bounded::HtmlConstructionLease::try_recover(&mut result, &mut lease) {
            Ok(value) => value,
            Err(error) => {
                let released = release_vector(c, joins, joins_allocation);
                // `try_recover` refuses before it takes the result, so the slot
                // filled above is still occupied.
                let Some(view) = result.take() else {
                    return Err(latch_registry_row(c));
                };
                self.storage = Some(FrozenStorage {
                    view,
                    control,
                    construction,
                    lease,
                    payloads,
                    effects,
                    evidence,
                    fragment,
                });
                released?;
                return Err(construction_owner::construction_error(c, error));
            }
        };
        self.sharing_started = true;
        let state = self.finalizing.insert(Finalizing {
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
        state.construction.recovered(c, recovery)?;
        Ok(())
    }
    /// Retires the join table and publishes the shared storage.
    fn publish_terminal(
        state: &mut Finalizing,
        c: &mut ParseControl,
    ) -> Result<(HtmlConstructionResult, bounded::HtmlConstructionLease), SessionError> {
        let (empty, empty_allocation) = c.new_vector()?;
        let joins = mem::replace(&mut state.joins, empty);
        let joins_allocation = mem::replace(&mut state.joins_allocation, empty_allocation);
        release_vector(c, joins, joins_allocation)?;
        state.construction.publish(c, state.root.clone())
    }
    pub(super) fn share_values<'c>(
        &mut self,
        c: &'c mut ParseControl,
        #[cfg(test)] mode: ValueSharingMode,
    ) -> Result<Sharing<'c>, SessionError> {
        self.begin_finalizing(c)?;
        let mut sharing = Sharing::new(
            c,
            #[cfg(test)]
            mode,
        )?;
        let (pools, control) = sharing.parts()?;
        let Some(state) = self.finalizing.as_mut() else {
            return Err(latch_registry_row(control));
        };
        state.run(pools, control)?;
        pools.prepare(control)?;
        let c = &mut *sharing.control;
        let Some(mut state) = self.finalizing.take() else {
            return Err(latch_registry_row(c));
        };
        let (view, lease) = match Self::publish_terminal(&mut state, c) {
            Ok(published) => published,
            Err(error) => {
                self.finalizing = Some(state);
                return Err(error);
            }
        };
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
        } = state;
        drop(root);
        let released = release_vector(c, joins, joins_allocation);
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
        released?;
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
            selected.release(sharing.control).unwrap();
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

    // Characterization of existing behaviour after B07 (a refused credit on
    // Drop is latched, not a panic); not RED against an earlier commit.
    const UNDERFLOW: ParseStop =
        ParseStop::InternalInvariant(crate::parser::control::Invariant::ReleaseUnderflow);

    #[test]
    fn characterization_dropping_a_pending_over_unadmitted_storage_latches_the_refusal() {
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        {
            let _pending = Pending::new(
                Selected {
                    value: (),
                    usage: Usage { bytes: 8, units: 2 },
                },
                &mut c,
            );
        }
        assert_eq!(c.stop(), Some(UNDERFLOW));
        assert_eq!(c.checkpoint(), Err(UNDERFLOW));
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn characterization_dropping_a_sharing_over_unadmitted_storage_latches_the_refusal() {
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        {
            let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
            // More than the controller ever admitted for these pools.
            sharing.pools.as_mut().unwrap().strings_usage = Usage {
                bytes: u64::MAX,
                units: 0,
            };
        }
        assert_eq!(c.stop(), Some(UNDERFLOW));
        assert_eq!(c.checkpoint(), Err(UNDERFLOW));
        // Every other credit was still attempted, so the real admission drained.
        super::super::tests::storage_zero(&c);
    }

    // Impossible registry, protocol and custody states are reported as the
    // controller's first stop with every owner handed back, never a panic. Each
    // test forces the state through crate-private access, asserts the exact
    // typed error, that it is the first stop, and that release drains.
    fn registry_row() -> ParseStop {
        ParseStop::InternalInvariant(crate::parser::control::Invariant::RegistryRow)
    }
    fn capacity_mismatch() -> ParseStop {
        ParseStop::InternalInvariant(crate::parser::control::Invariant::CapacityMismatch)
    }
    fn control() -> ParseControl {
        ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        )
    }
    /// A parsed document with two paragraphs of equal one-attribute banks,
    /// frozen and ready for value sharing.
    fn parsed() -> (ParseControl, FrozenArena) {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<body><p class=k>A</p><p class=k>A</p>"),
            crate::parser::tree_builder::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        loop {
            match driver.step().unwrap() {
                crate::parser::tree_builder::head::HeadStep::Consumed => {}
                crate::parser::tree_builder::head::HeadStep::Complete => break,
                step => panic!("fixture stopped at {step:?}"),
            }
        }
        let mut input = Some(driver);
        let (mut c, mut session, _) =
            crate::parser::tree_builder::StandaloneDocumentDriver::take_standalone_fixture(
                &mut input,
            )
            .unwrap();
        let arena = session.freeze(&mut c).unwrap();
        (c, arena)
    }
    /// The finalizing owner of a fresh pair.
    fn finalizing() -> (ParseControl, FrozenArena) {
        let (mut c, mut arena) = parsed();
        arena.begin_finalizing(&mut c).unwrap();
        (c, arena)
    }
    fn banks(state: &Finalizing) -> Vec<usize> {
        (0..state.payloads.values.len())
            .filter(|&slot| {
                matches!(
                    &state.payloads.values[slot],
                    RetainedPayload::Attributes(Some(AttributeStorage::Frozen { values, .. }))
                        if !values.is_empty()
                )
            })
            .collect()
    }
    fn strings(state: &Finalizing) -> Vec<usize> {
        (0..state.payloads.values.len())
            .filter(|&slot| matches!(state.payloads.values[slot], RetainedPayload::String { .. }))
            .collect()
    }
    fn element_node(state: &Finalizing) -> usize {
        (0..state.payloads.values.len() + 8)
            .find(|&node| {
                matches!(
                    state.construction.node_at(node).map(|record| record.kind()),
                    Some(HtmlNodeKind::Element(_))
                )
            })
            .expect("an element record")
    }
    fn attribute_string(state: &Finalizing, bank: usize) -> usize {
        strings(state)
            .into_iter()
            .find(|&slot| {
                matches!(
                    state.payloads.values[slot],
                    RetainedPayload::String { location, .. }
                        if attribute_location(location).is_some_and(|(owner, _)| owner == bank)
                )
            })
            .expect("an attribute string of the bank")
    }
    fn release_and_check(mut arena: FrozenArena, c: &mut ParseControl) {
        arena.release(c).unwrap();
        super::super::tests::storage_zero(c);
    }

    #[test]
    fn a_join_table_shorter_than_the_registry_reports_the_registry_row() {
        let (mut c, mut arena) = finalizing();
        let state = arena.finalizing.as_mut().unwrap();
        state.make_joins(&mut c).unwrap();
        let bank = banks(state)[0];
        state.joins.clear();
        {
            let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
            let pools = sharing.pools.as_mut().unwrap();
            assert_eq!(
                state.share_bank(bank, pools, sharing.control),
                Err(SessionError::Stop(registry_row()))
            );
            assert_eq!(sharing.control.stop(), Some(registry_row()));
        }
        release_and_check(arena, &mut c);
    }

    #[test]
    fn a_string_row_naming_a_container_past_the_registry_reports_the_registry_row() {
        let (mut c, mut arena) = finalizing();
        let state = arena.finalizing.as_mut().unwrap();
        let bank = banks(state)[0];
        let slot = attribute_string(state, bank);
        let RetainedPayload::String { location, .. } = &mut state.payloads.values[slot] else {
            panic!("string row")
        };
        let original = *location;
        *location = StringLocation::AttributeValue(usize::MAX, 0);
        assert_eq!(
            state.make_joins(&mut c),
            Err(SessionError::Stop(registry_row()))
        );
        assert_eq!(c.stop(), Some(registry_row()));
        let RetainedPayload::String { location, .. } = &mut state.payloads.values[slot] else {
            panic!("string row")
        };
        *location = original;
        release_and_check(arena, &mut c);
    }

    #[test]
    fn a_component_ordinal_past_the_selected_bank_reports_the_registry_row() {
        let (mut c, mut arena) = finalizing();
        let state = arena.finalizing.as_mut().unwrap();
        let banks = banks(state);
        // The second bank equals the first, so its components are updated from
        // the selected bank and one names an attribute that bank lacks.
        let slot = attribute_string(state, banks[1]);
        let RetainedPayload::String { location, .. } = &mut state.payloads.values[slot] else {
            panic!("string row")
        };
        let original = *location;
        *location = StringLocation::AttributeValue(banks[1], 99);
        {
            let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
            let pools = sharing.pools.as_mut().unwrap();
            assert_eq!(
                state.run(pools, sharing.control),
                Err(SessionError::Stop(registry_row()))
            );
            assert_eq!(sharing.control.stop(), Some(registry_row()));
        }
        let RetainedPayload::String { location, .. } = &mut state.payloads.values[slot] else {
            panic!("string row")
        };
        *location = original;
        release_and_check(arena, &mut c);
    }

    #[test]
    fn a_joined_slot_that_is_not_a_string_reports_the_registry_row() {
        let (mut c, mut arena) = finalizing();
        let state = arena.finalizing.as_mut().unwrap();
        state.make_joins(&mut c).unwrap();
        let bank = banks(state)[0];
        let slot = attribute_string(state, bank);
        let saved = mem::replace(
            &mut state.payloads.values[slot],
            RetainedPayload::Attributes(None),
        );
        {
            let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
            let pools = sharing.pools.as_mut().unwrap();
            assert_eq!(
                state.share_bank(bank, pools, sharing.control),
                Err(SessionError::Stop(registry_row()))
            );
            assert_eq!(sharing.control.stop(), Some(registry_row()));
        }
        state.payloads.values[slot] = saved;
        release_and_check(arena, &mut c);
    }

    #[test]
    fn promoting_a_row_that_is_not_a_bank_reports_the_registry_row() {
        let (mut c, mut arena) = finalizing();
        let state = arena.finalizing.as_mut().unwrap();
        state.make_joins(&mut c).unwrap();
        let slot = strings(state)[0];
        assert_eq!(
            state.promote_bank(slot, &mut c),
            Err(SessionError::Stop(registry_row()))
        );
        assert_eq!(c.stop(), Some(registry_row()));
        release_and_check(arena, &mut c);
    }

    #[test]
    fn installing_a_string_at_an_impossible_location_reports_the_registry_row() {
        let value = DocumentString::from("v");
        for location in [
            StringLocation::Detached,
            StringLocation::Text(usize::MAX),
            StringLocation::ElementName(0),
        ] {
            let (mut c, mut arena) = finalizing();
            let state = arena.finalizing.as_mut().unwrap();
            assert_eq!(
                state.install_string(location, &value, &mut c),
                Err(SessionError::Stop(registry_row()))
            );
            assert_eq!(c.stop(), Some(registry_row()));
            release_and_check(arena, &mut c);
        }
    }

    #[test]
    fn installing_a_bank_that_names_no_row_or_element_reports_and_releases_its_tickets() {
        for impossible in ["container", "element"] {
            let (mut c, mut arena) = finalizing();
            let state = arena.finalizing.as_mut().unwrap();
            state.make_joins(&mut c).unwrap();
            let bank = banks(state)[0];
            let element = element_node(state);
            let (container, element) = if impossible == "container" {
                (strings(state)[0], element)
            } else {
                (bank, usize::MAX)
            };
            let values = OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "class"),
                "k",
            )])
            .unwrap();
            let tickets = c.final_attribute_tickets(&values).unwrap();
            assert_eq!(
                state.install_bank(container, element, values, tickets, &mut c),
                Err(SessionError::Stop(registry_row()))
            );
            assert_eq!(c.stop(), Some(registry_row()));
            release_and_check(arena, &mut c);
        }
    }

    #[test]
    fn committing_an_update_over_a_row_that_is_not_a_string_reports_and_releases_it() {
        let (mut c, mut arena) = finalizing();
        let state = arena.finalizing.as_mut().unwrap();
        let slot = strings(state)[0];
        let mut updates = Updates::new(&mut c).unwrap();
        updates
            .push(slot, &DocumentString::from("new"), &mut c)
            .unwrap();
        let saved = mem::replace(
            &mut state.payloads.values[slot],
            RetainedPayload::Attributes(None),
        );
        let _ = updates.commit(&mut state.payloads, &mut c);
        assert_eq!(c.stop(), Some(registry_row()));
        state.payloads.values[slot] = saved;
        release_and_check(arena, &mut c);
    }

    #[test]
    fn releasing_an_update_the_controller_refuses_latches_instead_of_panicking() {
        let mut c = control();
        let mut other = control();
        let value = DocumentString::from("foreign");
        let ticket = other.final_string_ticket(&value).unwrap();
        let mut updates = Updates::new(&mut c).unwrap();
        c.reserve_vector_geometric(&mut updates.values, &mut updates.allocation, 1)
            .unwrap();
        updates.values.push(Update {
            slot: 0,
            value,
            allocation: ticket,
        });
        {
            let _pending = Pending::new(updates, &mut c);
        }
        assert_eq!(
            c.stop(),
            Some(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            ))
        );
        super::super::tests::storage_zero(&c);
    }

    fn foreign_string_seal() -> HtmlValuePoolSeal {
        let mut other = HtmlStringPool::new();
        let root = other.begin_root().expect("fresh pool root");
        match other.prepare_seal(root) {
            Ok(seal) => seal,
            Err((error, _)) => panic!("fresh pool seal: {error:?}"),
        }
    }

    #[test]
    fn committing_a_pair_whose_seal_the_pool_refuses_latches_and_releases() {
        let mut c = control();
        let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
        let pools = sharing.pools.as_mut().unwrap();
        // The pool's own open root is simply set aside.
        let _open = mem::replace(
            &mut pools.strings_root,
            Authority::Prepared(foreign_string_seal()),
        );
        assert_eq!(sharing.commit(), Err(capacity_mismatch()));
        assert_eq!(c.stop(), Some(capacity_mismatch()));
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn dropping_a_pair_whose_root_the_pool_refuses_latches_and_releases() {
        let mut c = control();
        {
            let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
            let pools = sharing.pools.as_mut().unwrap();
            let mut other = HtmlStringPool::new();
            let foreign = other.begin_root().expect("fresh pool root");
            let _own = mem::replace(&mut pools.strings_root, Authority::Open(foreign));
        }
        assert_eq!(c.stop(), Some(capacity_mismatch()));
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_pool_extent_the_controller_cannot_credit_reports_instead_of_panicking() {
        // A string miss, and a string hit whose selected value must be handed
        // back with its candidate credit.
        for hit in [false, true] {
            let mut c = control();
            let text = DocumentString::from("value");
            let equal = DocumentString::from("value");
            {
                let mut sharing = Sharing::new(&mut c, ValueSharingMode::Combined).unwrap();
                let pools = sharing.pools.as_mut().unwrap();
                if hit {
                    assert!(pools.string(&text, sharing.control).unwrap().is_none());
                    let real = pools.strings_usage;
                    pools.strings_usage = Usage {
                        bytes: u64::MAX,
                        units: 0,
                    };
                    let refused = pools.string(&equal, sharing.control);
                    assert!(matches!(
                        refused,
                        Err(SessionError::Stop(ParseStop::InternalInvariant(
                            crate::parser::control::Invariant::ReleaseUnderflow
                        )))
                    ));
                    pools.strings_usage = real;
                } else {
                    pools.strings_usage = Usage {
                        bytes: u64::MAX,
                        units: 0,
                    };
                    assert!(matches!(
                        pools.string(&text, sharing.control),
                        Err(SessionError::Stop(ParseStop::InternalInvariant(
                            crate::parser::control::Invariant::ReleaseUnderflow
                        )))
                    ));
                    pools.strings_usage = Usage::default();
                }
            }
            assert_eq!(c.stop(), Some(UNDERFLOW));
            super::super::tests::storage_zero(&c);
        }
    }
}

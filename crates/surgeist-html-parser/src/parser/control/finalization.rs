//! Explicit terminal shared-output tickets. Mutable allocation checks stay exact.
use super::*;
use crate::{Attribute, DocumentString, OrderedAttributes};

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum FinalizationCut {
    Work,
    Cancellation,
    Panic,
}
#[cfg(test)]
pub(super) struct CutRequest {
    kind: FinalizationCut,
    target: usize,
    earlier: usize,
}
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct FinalizationObservation {
    pub(crate) string_pool_bytes: usize,
    pub(crate) attribute_pool_bytes: usize,
    pub(crate) string_insertions: usize,
    pub(crate) attribute_insertions: usize,
    pub(crate) earlier_promoted: bool,
    pub(crate) selected_bank_bytes: usize,
    pub(crate) selected_components: usize,
    pub(crate) selected_tickets_finalized: bool,
    pub(crate) work: u64,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FinalizationTransferStage {
    FirstPrepared,
    Receiver,
}
#[cfg(test)]
pub(super) struct TransferCut {
    stage: FinalizationTransferStage,
    kind: FinalizationCut,
}
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct FinalizationTransferObservation {
    pub(crate) stage: FinalizationTransferStage,
    pub(crate) strings_prepared: bool,
    pub(crate) attributes_prepared: bool,
    pub(crate) pool_bytes: usize,
    pub(crate) retained_bytes: u64,
    pub(crate) work: u64,
}

impl DocumentStringAllocation {
    pub(crate) fn finalized(&self) -> bool {
        self.finalized
    }
    pub(crate) fn promote_final(&mut self) {
        self.finalized = true;
    }
}

impl<T> SharedAllocation<T> {
    pub(crate) fn finalized(&self) -> bool {
        self.finalized
    }
    pub(crate) fn promote_final(&mut self) {
        self.finalized = true;
    }
}
impl ParseControl {
    #[cfg(test)]
    pub(crate) fn arm_finalization_cut(
        &mut self,
        target: usize,
        earlier: usize,
        kind: FinalizationCut,
    ) {
        assert!(self.finalization_cut.is_none());
        self.finalization_cut = Some(CutRequest {
            kind,
            target,
            earlier,
        });
    }
    #[cfg(test)]
    pub(crate) fn finalization_cut_earlier(&self, target: usize) -> Option<usize> {
        self.finalization_cut
            .as_ref()
            .filter(|cut| cut.target == target)
            .map(|cut| cut.earlier)
    }
    #[cfg(test)]
    pub(crate) fn cut_finalization(&mut self, observation: FinalizationObservation) {
        let cut = self
            .finalization_cut
            .take()
            .expect("armed actual replacement");
        assert_eq!(
            observation.work,
            self.usage()
                .resource(HtmlParseResource::WorkCredits)
                .current
        );
        self.finalization_observation = Some(observation);
        match cut.kind {
            FinalizationCut::Work => {
                self.test_limit(HtmlParseResource::WorkCredits, observation.work)
            }
            FinalizationCut::Cancellation => self.cancellation.cancel(),
            FinalizationCut::Panic => panic!("terminal value-sharing replacement cut"),
        }
    }
    #[cfg(test)]
    pub(crate) fn arm_finalization_transfer(
        &mut self,
        stage: FinalizationTransferStage,
        kind: FinalizationCut,
    ) {
        assert!(self.finalization_transfer_cut.is_none());
        self.finalization_transfer_cut = Some(TransferCut { stage, kind });
    }
    #[cfg(test)]
    pub(crate) fn observe_finalization_transfer(
        &mut self,
        observation: FinalizationTransferObservation,
    ) {
        if !self
            .finalization_transfer_cut
            .as_ref()
            .is_some_and(|cut| cut.stage == observation.stage)
        {
            return;
        }
        let cut = self.finalization_transfer_cut.take().unwrap();
        self.finalization_transfer_observation = Some(observation);
        match cut.kind {
            FinalizationCut::Work => {
                self.test_limit(HtmlParseResource::WorkCredits, observation.work)
            }
            FinalizationCut::Cancellation => self.cancellation.cancel(),
            FinalizationCut::Panic => panic!("terminal value-sharing transfer cut"),
        }
    }
    #[cfg(test)]
    pub(crate) fn finalization_transfer_observation(
        &self,
    ) -> Option<FinalizationTransferObservation> {
        self.finalization_transfer_observation
    }
    #[cfg(test)]
    pub(crate) fn finalization_observation(&self) -> Option<FinalizationObservation> {
        self.finalization_observation
    }
    pub(crate) fn admit_pool_work(
        &mut self,
        work: surgeist_html::bounded::HtmlValuePoolWork,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::RecordVisit, work.record_visits() as u64),
            (WorkCategory::InspectByte, work.inspected_bytes() as u64),
            (WorkCategory::CopyByte, work.copied_bytes() as u64),
            (WorkCategory::ProviderCall, work.provider_calls() as u64),
            (WorkCategory::ReserveByte, work.reserve_bytes() as u64),
        ])
    }
    /// Credits a finalized vector ticket. A ticket of another controller or a
    /// capacity that does not fit the ledger is refused with a latched stop,
    /// credits nothing and is handed back.
    pub(crate) fn release_final_vector<T>(
        &mut self,
        vector: VectorCapacity<T>,
        attributes: u64,
    ) -> Result<(), FinalReleaseError<VectorCapacity<T>>> {
        let released = (|| {
            if !Arc::ptr_eq(&vector.owner, &self.identity) {
                return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
            }
            let Some(bytes) = vector
                .capacity
                .checked_mul(size_of::<T>())
                .and_then(|bytes| u64::try_from(bytes).ok())
            else {
                return self.representation(HtmlParseResource::ManagedCapacityBytes);
            };
            self.commit_prepared_release(&[
                (HtmlParseResource::ManagedCapacityBytes, bytes),
                (HtmlParseResource::PeakTransientBytes, bytes),
                (HtmlParseResource::ManagedAttributes, attributes),
            ])
        })();
        match released {
            Ok(()) => {
                drop(vector);
                Ok(())
            }
            Err(stop) => Err(FinalReleaseError {
                stop,
                tickets: vector,
            }),
        }
    }
    fn acquire_final(&mut self, bytes: u64, units: u64, attributes: u64) -> Result<(), ParseStop> {
        let values = self.plan_storage(&[
            (HtmlParseResource::ManagedUtf16Units, units),
            (HtmlParseResource::ManagedAttributes, attributes),
        ])?;
        let total = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current
            .checked_add(bytes)
            .ok_or_else(|| self.stop_representation(HtmlParseResource::ManagedCapacityBytes))?;
        self.check_value(HtmlParseResource::ManagedCapacityBytes, total, bytes)?;
        self.check_value(HtmlParseResource::PeakTransientBytes, total, bytes)?;
        for resource in RESOURCES {
            if resource.storage() {
                self.set(resource, values.copy(resource));
            }
        }
        self.set(HtmlParseResource::ManagedCapacityBytes, total);
        self.set(HtmlParseResource::PeakTransientBytes, total);
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        Ok(())
    }
    pub(crate) fn final_string_ticket(
        &mut self,
        value: &DocumentString,
    ) -> Result<DocumentStringAllocation, ParseStop> {
        self.charge_many(&[
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordWrite, 1),
        ])?;
        let capacity = value.shared_code_units().capacity();
        let bytes = capacity
            .checked_mul(size_of::<u16>())
            .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| self.stop_representation(HtmlParseResource::ManagedCapacityBytes))?;
        self.acquire_final(bytes, value.len() as u64, 0)?;
        Ok(DocumentStringAllocation {
            owner: self.identity.clone(),
            units: value.len() as u64,
            vector_capacity: capacity,
            wrapper_bytes: size_of::<Vec<u16>>() as u64,
            resource: HtmlParseResource::ManagedUtf16Units,
            data_pointer: value.code_units().as_ptr() as usize,
            backing: Some(Arc::downgrade(value.shared_code_units())),
            finalized: true,
        })
    }
    pub(crate) fn final_attribute_tickets(
        &mut self,
        value: &OrderedAttributes,
    ) -> Result<(VectorCapacity<Attribute>, SharedAllocation<Vec<Attribute>>), ParseStop> {
        self.charge_many(&[
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordWrite, 2),
        ])?;
        let capacity = value.shared_attributes().capacity();
        let bytes = capacity
            .checked_mul(size_of::<Attribute>())
            .and_then(|n| n.checked_add(size_of::<Vec<Attribute>>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| self.stop_representation(HtmlParseResource::ManagedCapacityBytes))?;
        self.acquire_final(bytes, 0, value.len() as u64)?;
        Ok((
            VectorCapacity {
                owner: self.identity.clone(),
                capacity,
                element: PhantomData,
            },
            SharedAllocation {
                owner: self.identity.clone(),
                shared: Arc::downgrade(value.shared_attributes()),
                bytes: size_of::<Vec<Attribute>>() as u64,
                element: PhantomData,
                finalized: true,
            },
        ))
    }
    /// Credits a finalized shared ticket. A ticket of another controller or a
    /// ticket that was not finalized is refused with a latched stop, credits
    /// nothing and is handed back.
    pub(crate) fn release_final_shared<T>(
        &mut self,
        shared: SharedAllocation<T>,
    ) -> Result<(), FinalReleaseError<SharedAllocation<T>>> {
        let released = if !Arc::ptr_eq(&shared.owner, &self.identity) {
            self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl))
        } else if !shared.finalized {
            self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
        } else {
            self.commit_prepared_release(&[
                (HtmlParseResource::ManagedCapacityBytes, shared.bytes),
                (HtmlParseResource::PeakTransientBytes, shared.bytes),
            ])
        };
        match released {
            Ok(()) => {
                drop(shared);
                Ok(())
            }
            Err(stop) => Err(FinalReleaseError {
                stop,
                tickets: shared,
            }),
        }
    }
    /// Credits a finalized attribute bank and both of its tickets. Tickets of
    /// another controller, an unfinalized ticket, or tickets that do not
    /// describe `value` are refused with a latched stop; nothing is credited
    /// and the bank and tickets are handed back.
    pub(crate) fn release_final_attributes(
        &mut self,
        value: OrderedAttributes,
        vector: VectorCapacity<Attribute>,
        shared: SharedAllocation<Vec<Attribute>>,
    ) -> Result<(), FinalReleaseError<FinalAttributes>> {
        let released = (|| {
            if !Arc::ptr_eq(&vector.owner, &self.identity)
                || !Arc::ptr_eq(&shared.owner, &self.identity)
            {
                return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
            }
            if !shared.finalized
                || shared.shared.as_ptr() != Arc::as_ptr(value.shared_attributes())
                || vector.capacity != value.shared_attributes().capacity()
            {
                return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
            }
            let Some(bytes) = vector
                .capacity
                .checked_mul(size_of::<Attribute>())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .and_then(|bytes| bytes.checked_add(shared.bytes))
            else {
                return self.representation(HtmlParseResource::ManagedCapacityBytes);
            };
            self.commit_prepared_release(&[
                (HtmlParseResource::ManagedCapacityBytes, bytes),
                (HtmlParseResource::PeakTransientBytes, bytes),
                (HtmlParseResource::ManagedAttributes, value.len() as u64),
            ])
        })();
        match released {
            Ok(()) => {
                drop(value);
                drop(vector);
                drop(shared);
                Ok(())
            }
            Err(stop) => Err(FinalReleaseError {
                stop,
                tickets: (value, vector, shared),
            }),
        }
    }
}

/// An attribute bank with the tickets that account for it.
pub(crate) type FinalAttributes = (
    OrderedAttributes,
    VectorCapacity<Attribute>,
    SharedAllocation<Vec<Attribute>>,
);

/// A final release the controller refused: the stop it latched and the
/// ticket(s) it did not take.
#[derive(Debug)]
pub(crate) struct FinalReleaseError<T> {
    stop: ParseStop,
    tickets: T,
}
impl<T> FinalReleaseError<T> {
    pub(crate) fn into_parts(self) -> (ParseStop, T) {
        (self.stop, self.tickets)
    }
    /// For a caller with nowhere to keep the refused tickets, such as a
    /// teardown of the storage that held them: drops them and returns the stop,
    /// which stays latched and reports the skipped credit.
    pub(crate) fn discard(self) -> ParseStop {
        let (stop, tickets) = self.into_parts();
        drop(tickets);
        stop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control() -> ParseControl {
        ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        )
    }

    #[test]
    fn selected_string_ticket_admits_actual_spare_capacity_exactly_before_weak_retention() {
        for one_short in [true, false] {
            let mut vector = Vec::with_capacity(37);
            vector.extend([0xd800, 65]);
            let value = DocumentString::from_utf16(vector);
            let bytes = (value.shared_code_units().capacity() * size_of::<u16>()
                + size_of::<Vec<u16>>()) as u64;
            let mut c = control();
            c.test_limit(
                HtmlParseResource::ManagedCapacityBytes,
                bytes - u64::from(one_short),
            );
            let result = c.final_string_ticket(&value);
            if one_short {
                assert!(
                    matches!(result, Err(ParseStop::LimitExceeded { resource: HtmlParseResource::ManagedCapacityBytes, current: 0, requested_delta, .. }) if requested_delta == bytes)
                );
                assert_eq!(Arc::weak_count(value.shared_code_units()), 0);
                assert_eq!(c.usage().actual_capacity_bytes(), 0);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    0
                );
            } else {
                let ticket = result.unwrap();
                assert_eq!(c.usage().actual_capacity_bytes(), bytes);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    2
                );
                c.require_document_string(&ticket, &value).unwrap();
                let held = value.clone();
                drop(value);
                c.release_document_string(ticket).unwrap();
                assert_eq!(held.code_units(), &[0xd800, 65]);
                assert_eq!(c.usage().actual_capacity_bytes(), 0);
                assert_eq!(Arc::weak_count(held.shared_code_units()), 0);
            }
        }
    }

    #[test]
    fn selected_attribute_ticket_keeps_capacity_and_cannot_reenter_mutable_editing() {
        let values = OrderedAttributes::new([Attribute::new(
            crate::QualifiedName::new(crate::Namespace::none(), None, "class"),
            "k",
        )])
        .unwrap();
        let mut c = control();
        let (vector, shared) = c.final_attribute_tickets(&values).unwrap();
        let bytes = (values.shared_attributes().capacity() * size_of::<Attribute>()
            + size_of::<Vec<Attribute>>()) as u64;
        assert_eq!(c.usage().actual_capacity_bytes(), bytes);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            1
        );
        assert!(matches!(
            c.prepare_attribute_edit(values.shared_attributes(), &shared, &vector),
            Err(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
        ));
        let held = values.clone();
        c.release_final_attributes(values, vector, shared).unwrap();
        assert_eq!(held.as_slice()[0].value().code_units(), &[107]);
        assert_eq!(Arc::weak_count(held.shared_attributes()), 0);
        assert_eq!(c.usage().actual_capacity_bytes(), 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            0
        );
    }
}

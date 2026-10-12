//! Consuming private ordinary construction ownership. This seal proves the
//! admitted immutable native/source transfer, not complete lexical D1 or public
//! parse success. Public promotion must consume the real complete associations
//! proof supplied by the sole evidence owner.

use super::{StandaloneError, end::FrozenStandaloneParts};
use crate::{HtmlDocumentParseRequest, request::DocumentRequestParts};

/// Exact physical transfer layouts are prepaid by the initialization owner.
/// Four cleared owning options, five detached fields, four request fields and
/// two joined fields: every terminal move is fixed and needs no new admission.
pub(super) fn terminal_transfer_layout() -> (usize, usize) {
    use std::mem::size_of;
    (
        size_of::<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>>()
            + size_of::<Option<super::end::FailedStandaloneDocument>>()
            + size_of::<super::end::DetachedStandaloneFailure>()
            + size_of::<Option<HtmlDocumentParseRequest>>()
            + size_of::<Option<super::end::DetachedStandaloneFailure>>()
            + size_of::<DocumentRequestParts>()
            + size_of::<OwnedOrdinaryFailure>(),
        4 + 5 + 4 + 2,
    )
}
/// The terminal join reads two actual Arc identities and coordinate tags, with
/// two fixed offline-context predicates. Admission precedes the first pull.
pub(super) fn terminal_qualification_bytes() -> usize {
    2 * std::mem::size_of::<crate::HtmlParseCancellation>()
        + 2 * std::mem::size_of::<crate::HtmlSourceId>()
        + 2 * std::mem::size_of::<crate::HtmlSourceCoordinates>()
}

pub(crate) struct SealedOrdinaryConstruction {
    request: DocumentRequestParts,
    parts: FrozenStandaloneParts,
}
impl SealedOrdinaryConstruction {
    /// Both original owning slots survive every admission/qualification error.
    /// No fallible operation follows the two already-admitted moves.
    pub(crate) fn take(
        request: &mut Option<HtmlDocumentParseRequest>,
        parts: &mut Option<FrozenStandaloneParts>,
    ) -> Result<Self, StandaloneError> {
        let original = request.as_ref().ok_or(StandaloneError::Consumed)?;
        parts
            .as_mut()
            .ok_or(StandaloneError::Consumed)?
            .prepare_seal(original)?;
        Ok(Self {
            request: request
                .take()
                .expect("qualified original request")
                .into_parts(),
            parts: parts.take().expect("admitted immutable construction"),
        })
    }
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.request.source()
    }
    pub(crate) fn limits(&self) -> &crate::HtmlParseLimits {
        self.request.limits()
    }
    pub(crate) fn is_offline(&self) -> bool {
        self.request.is_offline()
    }
    pub(crate) fn document(&self) -> Option<&crate::HtmlDocument> {
        self.parts.immutable_document()
    }
    /// Same fixed committed store, with no controller borrow or new admission.
    /// Its native-only publication basis does not certify complete lexical D1.
    pub(crate) fn evidence(
        &self,
    ) -> Result<&crate::construction::parser_session::evidence::ParseEvidence, StandaloneError>
    {
        self.parts.immutable_evidence()
    }
    pub(crate) fn final_usage(&self) -> &crate::parser::control::ParseUsage {
        self.parts.final_usage()
    }
    pub(crate) fn retained_usage(&self) -> &crate::parser::control::ParseUsage {
        self.parts.retained_usage()
    }
    /// Actual fallible disposal remains explicit. A live borrowed/owned native
    /// alias can refuse disposal; that error never discards the retained owner.
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        self.parts.release_sealed()
    }
}

/// Cleaned terminal parse evidence, joined to the original owned request. There
/// is deliberately no tree/effect/native lender on this stopped result.
pub(crate) struct OwnedOrdinaryFailure {
    request: DocumentRequestParts,
    failure: super::end::DetachedStandaloneFailure,
}
impl OwnedOrdinaryFailure {
    pub(crate) fn take(
        request: &mut Option<HtmlDocumentParseRequest>,
        failure: &mut Option<super::end::DetachedStandaloneFailure>,
    ) -> Result<Self, StandaloneError> {
        let original = request.as_ref().ok_or(StandaloneError::Consumed)?;
        let failed = failure.as_ref().ok_or(StandaloneError::Consumed)?;
        if original.capture().is_some()
            || original.document().is_some()
            || !failed.owns_request(original.cancellation())
            || &failed.binding.0 != original.source().identity()
            || failed.binding.1 != original.source().coordinates()
        {
            return Err(StandaloneError::Consumed);
        }
        // Fixed envelope moves were admitted before the first source pull.
        Ok(Self {
            request: request
                .take()
                .expect("actual original request")
                .into_parts(),
            failure: failure.take().expect("qualified cleaned failure"),
        })
    }
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.request.source()
    }
    pub(crate) fn stop(&self) -> crate::parser::control::ParseStop {
        self.failure.stop()
    }
    pub(crate) fn at_stop(&self) -> &crate::parser::control::ParseUsage {
        self.failure.at_stop()
    }
    pub(crate) fn after_cleanup(&self) -> &crate::parser::control::ParseUsage {
        self.failure.after_cleanup()
    }
    pub(crate) fn retained_usage(&self) -> &crate::parser::control::ParseUsage {
        self.failure.retained_usage()
    }
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::parser_session::evidence::DiagnosticEvidenceView<'_>> {
        self.failure.diagnostics()
    }
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        self.failure.release()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tree_builder::{StandaloneDocumentDriver, head::HeadStep};
    use crate::{HtmlParseResource, HtmlSource};

    fn frozen(request: &HtmlDocumentParseRequest) -> FrozenStandaloneParts {
        let mut driver = Some(super::super::entry::initialized_driver(request));
        for _ in 0..4096 {
            if driver.as_mut().unwrap().step().unwrap() == HeadStep::Complete {
                let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
                completed.prepare_native_associations().unwrap();
                return completed.freeze_parts().unwrap();
            }
        }
        panic!("actual ordinary EOF was not reached");
    }
    fn request() -> HtmlDocumentParseRequest {
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>A&amp;B"),
            super::super::tests::limits(100),
        )
    }
    fn work_limits(work: u64) -> crate::HtmlParseLimits {
        crate::HtmlParseLimits::builder()
            .source_units(100_000)
            .buffered_utf16_units(100_000)
            .token_attributes(10_000)
            .auxiliary_entries(1_000_000)
            .managed_nodes(100_000)
            .managed_edges(100_000)
            .managed_attributes(100_000)
            .managed_utf16_units(100_000)
            .source_map_entries(100_000)
            .source_map_payload_units(100_000)
            .diagnostics(100_000)
            .diagnostic_payload_units(100_000)
            .context_effects(100_000)
            .open_elements(100)
            .tree_depth(100_000)
            .managed_capacity_bytes(100_000_000)
            .peak_transient_bytes(100_000_000)
            .atomic_layout_bytes(100_000_000)
            .work_credits(work)
            .finish()
            .unwrap()
    }
    #[test]
    fn actual_seal_write_admission_cut_preserves_both_slots_and_terminal_cleanup() {
        let original = request();
        let mut measured = frozen(&original);
        let before = measured
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let bytes = std::mem::size_of::<SealedOrdinaryConstruction>()
            + std::mem::size_of::<Option<FrozenStandaloneParts>>()
            + std::mem::size_of::<Option<HtmlDocumentParseRequest>>()
            + std::mem::size_of::<DocumentRequestParts>()
            + std::mem::size_of::<crate::parser::control::ParseUsage>();
        measured.release().unwrap();
        let mut request = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>A&amp;B"),
            work_limits(before + bytes as u64 + 1 + 5 + 9 - 1),
        ));
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        assert_eq!(
            parts
                .as_ref()
                .unwrap()
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before
        );
        assert!(matches!(
            SealedOrdinaryConstruction::take(&mut request, &mut parts),
            Err(StandaloneError::Stop(
                crate::parser::control::ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    requested_delta: 9,
                    ..
                }
            ))
        ));
        assert!(request.is_some() && parts.is_some());
        let at_stop = parts.as_ref().unwrap().at_stop().unwrap().clone();
        parts.as_mut().unwrap().release().unwrap();
        assert_eq!(parts.as_ref().unwrap().at_stop(), Some(&at_stop));
        assert_eq!(
            parts
                .as_ref()
                .unwrap()
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }
    #[test]
    fn seal_moves_original_source_and_same_frozen_tree_without_readback_work() {
        let mut request = Some(request());
        let identity = request.as_ref().unwrap().source().identity().clone();
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        let document = parts.as_mut().unwrap().document().unwrap().clone();
        let root = document.root().id();
        let mut sealed = SealedOrdinaryConstruction::take(&mut request, &mut parts).unwrap();
        assert!(request.is_none() && parts.is_none());
        assert_eq!(sealed.source().identity(), &identity);
        assert_eq!(sealed.document().unwrap().root().id(), root);
        assert!(sealed.is_offline());
        assert_eq!(sealed.limits(), &super::super::tests::limits(100));
        let tree = sealed.document().unwrap();
        let element = |parent: crate::HtmlNode<'_>, name: &str| {
            parent.children().find_map(|id| {
                let node = tree.node(&id).unwrap();
                matches!(node.kind(), crate::HtmlNodeKind::Element(e) if e.name().local_name().code_units() == name.encode_utf16().collect::<Vec<_>>()).then_some(node)
            }).expect("literal native element")
        };
        let html = element(tree.root(), "html");
        let body = element(html, "body");
        let p = element(body, "p");
        let text = tree.node(&p.children().next().unwrap()).unwrap();
        assert!(
            matches!(text.kind(), crate::HtmlNodeKind::Text(value) if value.code_units() == [65, 38, 66])
        );
        let before = sealed.retained_usage().clone();
        let evidence = sealed.evidence().unwrap() as *const _;
        for _ in 0..3 {
            assert_eq!(sealed.evidence().unwrap() as *const _, evidence);
            assert_eq!(sealed.document().unwrap().root().id(), root);
        }
        assert_eq!(sealed.retained_usage(), &before);
        drop(document);
        sealed.release().unwrap();
        assert!(sealed.document().is_none());
        assert!(sealed.evidence().is_err());
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        sealed.release().unwrap();
    }
    #[test]
    fn later_cancellation_does_not_invalidate_immutable_native_or_evidence_lenders() {
        let mut request = Some(request());
        let cancellation = request.as_ref().unwrap().cancellation().clone();
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        let mut sealed = SealedOrdinaryConstruction::take(&mut request, &mut parts).unwrap();
        let final_usage = sealed.final_usage().clone();
        let retained = sealed.retained_usage().clone();
        let evidence = sealed.evidence().unwrap() as *const _;
        cancellation.cancel();
        assert!(sealed.request.cancellation().is_cancelled());
        assert!(sealed.document().is_some());
        assert_eq!(sealed.evidence().unwrap() as *const _, evidence);
        assert_eq!(sealed.final_usage(), &final_usage);
        assert_eq!(sealed.retained_usage(), &retained);
        sealed.release().unwrap();
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }
    #[test]
    fn real_alias_refusal_retains_sealed_owner_and_retry_releases_same_storage() {
        let mut request = Some(request());
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        let mut sealed = SealedOrdinaryConstruction::take(&mut request, &mut parts).unwrap();
        let alias = sealed.document().unwrap().clone();
        let identity = alias.root().id();
        let evidence = sealed.evidence().unwrap() as *const _;
        let final_usage = sealed.final_usage().clone();
        assert!(sealed.release().is_err());
        assert_eq!(sealed.document().unwrap().root().id(), identity);
        assert_eq!(sealed.evidence().unwrap() as *const _, evidence);
        assert_eq!(sealed.final_usage(), &final_usage);
        assert!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                > 0
        );
        drop(alias);
        sealed.release().unwrap();
        assert!(sealed.document().is_none());
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }
    #[test]
    fn foreign_equal_text_request_and_final_cancellation_keep_both_original_slots() {
        let original = request();
        let mut parts = Some(frozen(&original));
        let mut foreign = Some(request());
        assert!(SealedOrdinaryConstruction::take(&mut foreign, &mut parts).is_err());
        assert!(foreign.is_some() && parts.is_some());
        parts.as_mut().unwrap().release().unwrap();
        let mut actual = Some(request());
        let mut parts = Some(frozen(actual.as_ref().unwrap()));
        actual.as_ref().unwrap().cancellation().cancel();
        assert!(SealedOrdinaryConstruction::take(&mut actual, &mut parts).is_err());
        assert!(actual.is_some() && parts.is_some());
        parts.as_mut().unwrap().release().unwrap();
    }
    #[test]
    fn stopped_frozen_alias_refusal_preserves_binding_and_retry_joins_original_owned_source() {
        let mut request = Some(request());
        let source = request.as_ref().unwrap().source().identity().clone();
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        let alias = parts.as_mut().unwrap().document().unwrap().clone();
        request.as_ref().unwrap().cancellation().cancel();
        assert!(SealedOrdinaryConstruction::take(&mut request, &mut parts).is_err());
        let first = parts.as_ref().unwrap().at_stop().unwrap().clone();
        assert!(FrozenStandaloneParts::detach_failure(&mut parts).is_err());
        assert!(parts.is_some() && request.is_some());
        assert_eq!(parts.as_ref().unwrap().source_binding().0, &source);
        assert_eq!(parts.as_ref().unwrap().at_stop(), Some(&first));
        drop(alias);
        let mut failure = Some(FrozenStandaloneParts::detach_failure(&mut parts).unwrap());
        assert!(parts.is_none());
        let mut owned = OwnedOrdinaryFailure::take(&mut request, &mut failure).unwrap();
        assert_eq!(owned.source().identity(), &source);
        assert_eq!(owned.at_stop(), &first);
        assert_eq!(
            owned
                .retained_usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            first.resource(HtmlParseResource::ManagedNodes).current
        );
        owned.release().unwrap();
        assert_eq!(owned.at_stop(), &first);
        assert_eq!(
            owned
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    fn stopped(request: &HtmlDocumentParseRequest) -> super::super::end::DetachedStandaloneFailure {
        let mut driver = Some(super::super::entry::initialized_driver(request));
        for _ in 0..256 {
            if driver
                .as_ref()
                .unwrap()
                .control
                .usage()
                .resource(HtmlParseResource::Diagnostics)
                .current
                > 0
            {
                break;
            }
            driver.as_mut().unwrap().step().unwrap();
        }
        assert!(
            driver
                .as_ref()
                .unwrap()
                .control
                .usage()
                .resource(HtmlParseResource::Diagnostics)
                .current
                > 0
        );
        request.cancellation().cancel();
        assert!(driver.as_mut().unwrap().step().is_err());
        let mut failed = Some(StandaloneDocumentDriver::take_failed(&mut driver).unwrap());
        assert!(driver.is_none());
        let detached = super::super::end::FailedStandaloneDocument::detach(&mut failed).unwrap();
        assert!(failed.is_none());
        assert!(super::super::end::FailedStandaloneDocument::detach(&mut failed).is_err());
        detached
    }
    #[test]
    fn cleaned_terminal_transfer_owns_original_source_and_keeps_exact_first_stop_and_retention() {
        let mut request = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        ));
        let identity = request.as_ref().unwrap().source().identity().clone();
        let mut failure = Some(stopped(request.as_ref().unwrap()));
        let at_stop = failure.as_ref().unwrap().at_stop().clone();
        let cleaned = failure.as_ref().unwrap().after_cleanup().clone();
        let retained = failure.as_ref().unwrap().retained_usage().clone();
        let records = failure.as_ref().unwrap().diagnostics().unwrap().len();
        assert!(records > 0);
        let mut owned = OwnedOrdinaryFailure::take(&mut request, &mut failure).unwrap();
        assert!(request.is_none() && failure.is_none());
        assert_eq!(owned.source().identity(), &identity);
        assert_eq!(owned.stop(), crate::parser::control::ParseStop::Cancelled);
        assert_eq!(owned.at_stop(), &at_stop);
        assert_eq!(owned.after_cleanup(), &cleaned);
        assert_eq!(owned.retained_usage(), &retained);
        assert_eq!(owned.diagnostics().unwrap().len(), records);
        owned.release().unwrap();
        assert_eq!(owned.at_stop(), &at_stop);
        assert_eq!(owned.after_cleanup(), &cleaned);
        assert_eq!(
            owned
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        owned.release().unwrap();
    }
    #[test]
    fn foreign_terminal_join_refuses_without_losing_original_request_or_failure_and_retry_succeeds()
    {
        let mut request = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        ));
        let mut failure = Some(stopped(request.as_ref().unwrap()));
        let mut foreign = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        ));
        let before = failure.as_ref().unwrap().retained_usage().clone();
        assert!(OwnedOrdinaryFailure::take(&mut foreign, &mut failure).is_err());
        assert!(foreign.is_some() && failure.is_some());
        assert_eq!(failure.as_ref().unwrap().retained_usage(), &before);
        let mut owned = OwnedOrdinaryFailure::take(&mut request, &mut failure).unwrap();
        owned.release().unwrap();
    }
    #[test]
    fn source_binding_move_is_cleanup_only_and_once_only_without_changing_first_stop() {
        let request = request();
        let mut control = crate::parser::control::ParseControl::new(
            *request.limits(),
            request.cancellation().clone(),
        );
        control.admit_source(request.source()).unwrap();
        assert!(control.take_stopped_source_binding().is_err());
        assert!(control.stop().is_none());
        control.require_source(request.source()).unwrap();
        request.cancellation().cancel();
        assert!(control.checkpoint().is_err());
        let before = control.usage().clone();
        let first = control.first_stop_usage().unwrap().clone();
        let binding = control.take_stopped_source_binding().unwrap();
        assert_eq!(&binding.0, request.source().identity());
        assert_eq!(binding.1, request.source().coordinates());
        assert!(control.take_stopped_source_binding().is_err());
        assert_eq!(control.usage(), &before);
        assert_eq!(control.first_stop_usage(), Some(&first));
    }
    #[test]
    fn same_source_fresh_request_cannot_join_sealed_runtime_or_frozen_owners() {
        let mut original = Some(request());
        let mut fresh = Some(HtmlDocumentParseRequest::offline(
            original.as_ref().unwrap().source().clone(),
            work_limits(99_999_999),
        ));
        let mut parts = Some(frozen(original.as_ref().unwrap()));
        assert!(SealedOrdinaryConstruction::take(&mut fresh, &mut parts).is_err());
        assert!(fresh.is_some() && parts.is_some());
        let mut sealed = SealedOrdinaryConstruction::take(&mut original, &mut parts).unwrap();
        sealed.release().unwrap();

        let mut original = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        ));
        let mut fresh = Some(HtmlDocumentParseRequest::offline(
            original.as_ref().unwrap().source().clone(),
            work_limits(99_999_999),
        ));
        let mut failure = Some(stopped(original.as_ref().unwrap()));
        assert!(OwnedOrdinaryFailure::take(&mut fresh, &mut failure).is_err());
        assert!(fresh.is_some() && failure.is_some());
        let mut owned = OwnedOrdinaryFailure::take(&mut original, &mut failure).unwrap();
        owned.release().unwrap();

        let mut original = Some(request());
        let mut fresh = Some(HtmlDocumentParseRequest::offline(
            original.as_ref().unwrap().source().clone(),
            work_limits(99_999_999),
        ));
        let mut parts = Some(frozen(original.as_ref().unwrap()));
        original.as_ref().unwrap().cancellation().cancel();
        assert!(SealedOrdinaryConstruction::take(&mut original, &mut parts).is_err());
        let mut failure = Some(FrozenStandaloneParts::detach_failure(&mut parts).unwrap());
        assert!(OwnedOrdinaryFailure::take(&mut fresh, &mut failure).is_err());
        assert!(fresh.is_some() && failure.is_some());
        let mut owned = OwnedOrdinaryFailure::take(&mut original, &mut failure).unwrap();
        owned.release().unwrap();
    }
}

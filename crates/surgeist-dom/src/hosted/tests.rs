//! Actual issuer identities and admission, without an external event-loop fixture.
use super::*;
use std::mem::size_of;

impl DomHostedSource {
    /// Raw physical next issuer-path witness for sibling constructor tests.
    /// This creates only empty storage, without an identity, revision or Work.
    pub(crate) fn next_operation_reservation_scratch_bytes_for_fixture(&self) -> usize {
        let state = self.lock().unwrap();
        let bank = &state.banks[Role::Operation.index()];
        let index = bank.lowest_vacant().unwrap_or_else(|| bank.len());
        if bank.get(index).is_some() {
            return 0;
        }
        let path = bank
            .reserve(index, |_| Ok::<Option<Retained>, DomHostedError>(None))
            .unwrap();
        size_of::<crate::store::EmptyPath<IdentitySlot>>() + path.inspect_storage_bytes()
    }
    /// Canonical-owner fixtures need a genuinely metered, live Global token.
    /// This issues only that identity: no Window, constructor phase or external
    /// execution effect is asserted, and no canonical revision is reserved.
    pub(crate) fn issue_global_for_canonical_source_fixture(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<DomHostedGlobalId, DomHostedError> {
        let work = self.work(cancel)?;
        let (slot, generation) = self.issue(Role::Global, &work, false, None, 0, cancel)?;
        Ok(DomHostedGlobalId {
            source: self.id(),
            slot,
            generation,
        })
    }
}

fn cancellation() -> AtomicBool {
    AtomicBool::new(false)
}

fn record_limits(hosts: usize, contexts: usize) -> DomHostedRecordLimits {
    DomHostedRecordLimits::new(hosts, contexts, 32, 32, 32, 32, 32)
}

fn limits(records: DomHostedRecordLimits) -> DomHostedLimits {
    DomHostedLimits::new(records, 1_000_000, 1_000_000, 1_000_000)
}

fn source(limits: DomHostedLimits) -> DomHostedSource {
    DomHostedSource::new(
        DomHostedSourceId {
            owner: DomOwnerId::allocate().unwrap(),
            serial: 0,
        },
        limits,
        &cancellation(),
    )
    .unwrap()
}

fn retained(source: &DomHostedSource) -> usize {
    source.shared.bytes.load(Ordering::Relaxed)
}

#[test]
fn hosted_operation_issuance_keeps_the_callers_live_candidate_in_scratch() {
    let c = cancellation();
    let configured = limits(record_limits(1, 1));
    // The reservation owns only empty path allocations. Independently enumerate
    // its actual page backing and Slot body; no resident row is copied.
    let reservation_bytes = {
        let bank = crate::store::OwnedBank::<IdentitySlot>::default();
        let reservation = bank
            .reserve(0, |_| Ok::<Option<Retained>, DomHostedError>(None))
            .unwrap();
        let raw = reservation.inspect_storage_bytes();
        assert_eq!(
            raw,
            crate::store::OwnedBank::<IdentitySlot>::inspect_first_reservation_bytes()
        );
        assert_eq!(reservation.charge().bytes, raw);
        raw
    };
    let caller = vec![0u8; 17];
    let caller_bytes = size_of::<Vec<u8>>() + caller.capacity();
    let replacement = size_of::<crate::store::EmptyPath<IdentitySlot>>() + reservation_bytes;
    let exact = caller_bytes + replacement;
    for allowance in [exact - 1, exact] {
        let mut issuer = source(configured);
        let host = issuer.issue_host(&c).unwrap();
        let prefix = issuer.prepare_top_level(host, &c).unwrap();
        let context = prefix.context;
        let work = prefix.work;
        let before = retained(&issuer);
        let revision = issuer.revision().unwrap();
        let spent = work.spent.load(Ordering::Relaxed);
        issuer
            .set_limits(
                DomHostedLimits::new(
                    configured.records(),
                    configured.retained_bytes(),
                    allowance,
                    configured.work(),
                ),
                &c,
            )
            .unwrap();
        let result = issuer.begin_operation(&work, 2, caller_bytes, &c);
        if allowance < exact {
            match result {
                Err(DomHostedError::Limit {
                    resource: DomHostedResource::ScratchBytes,
                    limit,
                }) if limit == allowance => {}
                Ok(id) => panic!(
                    "one-short scratch admitted issuer replacement: caller={caller_bytes}, replacement={replacement}, exact={exact}, operation={id:?}"
                ),
                Err(error) => {
                    panic!("one-short issuer scratch returned an unrelated error: {error:?}")
                }
            }
            assert_eq!(issuer.revision().unwrap(), revision);
            assert_eq!(retained(&issuer), before);
            let state = issuer.lock().unwrap();
            assert!(state.banks[Role::Operation.index()].is_empty());
            assert_eq!(state.reserved_revisions, 0);
        } else {
            let operation = result.unwrap();
            assert_eq!(
                issuer.lock().unwrap().banks[Role::Operation.index()].inspect_storage_bytes(),
                reservation_bytes
            );
            let request = Request {
                key: RequestKey {
                    operation,
                    phase: 0,
                },
                host,
                context,
                work: work.clone(),
                parent: None,
            };
            issuer
                .permit(&request, issuer.revision().unwrap(), true, &c)
                .unwrap()
                .commit();
            assert_eq!(issuer.lock().unwrap().reserved_revisions, 0);
        }
        assert!(work.spent.load(Ordering::Relaxed) > spent);
        assert_eq!(caller, [0; 17]);
    }
}

#[test]
fn hosted_retirement_reuses_only_the_next_generation_and_never_wraps() {
    let cancel = cancellation();
    let mut source = source(limits(record_limits(1, 1)));
    let first = source.issue_host(&cancel).unwrap();
    let observation = source.observe();
    source.retire_host(first, &cancel).unwrap();
    let second = source.issue_host(&cancel).unwrap();
    assert_eq!(second.slot, first.slot);
    assert_eq!(second.generation, first.generation + 1);
    assert_eq!(
        observation.validate(first.source, Role::Host, first.slot, first.generation),
        Err(DomHostedError::StaleIdentity)
    );
    assert!(
        observation
            .validate(second.source, Role::Host, second.slot, second.generation)
            .is_ok()
    );

    // Reach the last representable generation of this actual issued slot.
    // Retirement may release it, but may never make the same identity live again.
    source.lock().unwrap().banks[Role::Host.index()][second.slot].generation = u64::MAX;
    let last = DomHostedHostId {
        generation: u64::MAX,
        ..second
    };
    source.retire_host(last, &cancel).unwrap();
    let next = source.issue_host(&cancel).unwrap();
    assert_ne!(next.slot, last.slot);
    assert_eq!(next.generation, 1);
    let state = source.lock().unwrap();
    let retired = &state.banks[Role::Host.index()][last.slot];
    assert!(retired.retired);
    assert!(!retired.live);
    assert_eq!(retired.generation, u64::MAX);
    drop(state);
    assert_eq!(
        observation.validate(last.source, Role::Host, last.slot, last.generation),
        Err(DomHostedError::StaleIdentity)
    );
    let revision = source.revision().unwrap();
    assert_eq!(
        source.retire_host(second, &cancel),
        Err(DomHostedError::StaleIdentity)
    );
    assert_eq!(source.revision().unwrap(), revision);
}

#[test]
fn hosted_observations_do_not_keep_issuer_permission_alive() {
    let mut source = source(limits(record_limits(1, 1)));
    let host = source.issue_host(&cancellation()).unwrap();
    let observation = source.observe();
    let history_backing = source.shared.clone();
    assert!(
        observation
            .validate(host.source, Role::Host, host.slot, host.generation)
            .is_ok()
    );
    drop(source);
    // Even an independently retained historical allocation cannot revive the
    // dropped unique issuer. Releasing it also expires the actual Weak pointer.
    assert!(observation.shared.upgrade().is_some());
    assert_eq!(
        observation.validate(host.source, Role::Host, host.slot, host.generation),
        Err(DomHostedError::SourceExpired)
    );
    drop(history_backing);
    assert!(observation.shared.upgrade().is_none());
    assert_eq!(
        observation.validate(host.source, Role::Host, host.slot, host.generation),
        Err(DomHostedError::SourceExpired)
    );
}

#[test]
fn hosted_foreign_source_and_retired_generation_are_distinct_refusals() {
    let cancel = cancellation();
    let mut first = source(limits(record_limits(1, 1)));
    let mut second = source(limits(record_limits(1, 1)));
    let host = first.issue_host(&cancel).unwrap();
    let foreign = second.issue_host(&cancel).unwrap();
    let revision = first.revision().unwrap();
    assert_eq!(
        first.retire_host(foreign, &cancel),
        Err(DomHostedError::ForeignSource)
    );
    assert_eq!(first.revision().unwrap(), revision);
    first.retire_host(host, &cancel).unwrap();
    assert_eq!(
        first.retire_host(host, &cancel),
        Err(DomHostedError::StaleIdentity)
    );
}

#[test]
fn hosted_cancelled_seal_preserves_the_exact_prepared_envelope_and_work() {
    let cancel = cancellation();
    let mut source = source(limits(record_limits(1, 1)));
    let host = source.issue_host(&cancel).unwrap();
    let prepared = source.prepare_top_level(host, &cancel).unwrap();
    let context = prepared.context;
    let work_pointer = Arc::as_ptr(&prepared.work);
    let spent = prepared.work.spent.load(Ordering::Relaxed);
    let revision = source.revision().unwrap();
    let bytes = retained(&source);
    let mut prepared = Some(prepared);
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        source.seal_top_level(&mut prepared, &cancel),
        Err(DomHostedError::Dom(DomError::Cancelled))
    ));
    let preserved = prepared.as_ref().unwrap();
    assert_eq!(preserved.host(), host);
    assert_eq!(preserved.browsing_context(), context);
    assert_eq!(Arc::as_ptr(&preserved.work), work_pointer);
    assert_eq!(preserved.work.spent.load(Ordering::Relaxed), spent);
    assert_eq!(source.revision().unwrap(), revision);
    assert_eq!(retained(&source), bytes);
    cancel.store(false, Ordering::Relaxed);
    let sealed = source.seal_top_level(&mut prepared, &cancel).unwrap();
    assert!(prepared.is_none());
    assert_eq!(sealed.preparation.context, context);
    assert_eq!(Arc::as_ptr(&sealed.preparation.work), work_pointer);
    assert_eq!(
        sealed.preparation.work.spent.load(Ordering::Relaxed),
        spent + 1
    );
    assert_eq!(
        source.seal_top_level(&mut prepared, &cancel).unwrap_err(),
        DomHostedError::MissingPublication
    );
    let state = source.lock().unwrap();
    assert_eq!(
        state.banks[Role::BrowsingContext.index()]
            .values()
            .filter(|slot| slot.live)
            .count(),
        1
    );
}

#[test]
fn hosted_record_bounds_reject_before_issuing_or_growing_another_row() {
    let cancel = cancellation();
    let mut source = source(limits(record_limits(1, 1)));
    let host = source.issue_host(&cancel).unwrap();
    let revision = source.revision().unwrap();
    let bytes = retained(&source);
    let storage = source.lock().unwrap().banks[Role::Host.index()].inspect_storage_bytes();
    assert_eq!(
        source.issue_host(&cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::Hosts,
            limit: 1,
        })
    );
    assert_eq!(source.revision().unwrap(), revision);
    assert_eq!(retained(&source), bytes);
    assert_eq!(
        source.lock().unwrap().banks[Role::Host.index()].inspect_storage_bytes(),
        storage
    );
    let prepared = source.prepare_top_level(host, &cancel).unwrap();
    assert!(matches!(
        source.prepare_top_level(host, &cancel),
        Err(DomHostedError::IncompleteOperation)
    ));
    let context = prepared.context;
    drop(prepared);
    let revision = source.revision().unwrap();
    let bytes = retained(&source);
    assert!(matches!(
        source.prepare_top_level(host, &cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::BrowsingContexts,
            limit: 1,
        })
    ));
    assert_eq!(source.revision().unwrap(), revision);
    assert_eq!(retained(&source), bytes);
    assert!(
        source
            .observe()
            .validate(
                context.source,
                Role::BrowsingContext,
                context.slot,
                context.generation
            )
            .is_ok()
    );
}

#[test]
fn hosted_fixed_owner_and_work_payload_have_exact_byte_boundaries() {
    let cancel = cancellation();
    let records = record_limits(1, 1);
    let fixed = size_of::<Shared>();
    let work = size_of::<Work>() + 2 * size_of::<AtomicU64>();
    let id = DomHostedSourceId {
        owner: DomOwnerId::allocate().unwrap(),
        serial: 0,
    };
    assert!(matches!(DomHostedSource::new(id,
        DomHostedLimits::new(records, fixed - 1, 0, 10), &cancel),
        Err(DomHostedError::Limit { resource: DomHostedResource::RetainedBytes, limit })
            if limit == fixed - 1));
    let exact =
        DomHostedSource::new(id, DomHostedLimits::new(records, fixed, 0, 10), &cancel).unwrap();
    assert_eq!(retained(&exact), fixed);
    assert!(matches!(exact.work(&cancel), Err(DomHostedError::Limit {
        resource: DomHostedResource::RetainedBytes, limit,
    }) if limit == fixed));
    assert_eq!(retained(&exact), fixed);
    let exact = source(DomHostedLimits::new(records, fixed + work, 0, 10));
    let cursor = exact.work(&cancel).unwrap();
    assert_eq!(retained(&exact), fixed + work);
    assert_eq!(exact.remaining_bytes().unwrap(), 0);
    drop(cursor);
    assert_eq!(retained(&exact), fixed);
}

#[test]
fn hosted_growth_counts_actual_pages_and_empty_path_overlap_without_copying_old_rows() {
    let cancel = cancellation();
    let mut source = source(limits(record_limits(usize::MAX, 1)));
    source.issue_host(&cancel).unwrap();
    let width = crate::store::OwnedBank::<IdentitySlot>::inspect_leaf_width();
    // Fill the actual first leaf, including retained inactive/retired row slots.
    for _ in 1..width {
        source.issue_host(&cancel).unwrap();
    }
    let (old_bytes, requested) = {
        let state = source.lock().unwrap();
        let bank = &state.banks[Role::Host.index()];
        let reservation = bank
            .reserve(width, |_| Ok::<Option<Retained>, DomHostedError>(None))
            .unwrap();
        let requested = reservation.inspect_storage_bytes();
        assert_eq!(reservation.charge().bytes, requested);
        (bank.inspect_storage_bytes(), requested)
    };
    let fixed = size_of::<Shared>();
    let work = size_of::<Work>() + 2 * size_of::<AtomicU64>();
    assert_eq!(retained(&source), fixed + old_bytes);
    // Existing pages stay resident while the genuine new empty path is admitted.
    // There is no replacement backing or copied-old-row quota in OwnedBank.
    let refusal_limit = fixed + work + old_bytes + requested - 1;
    let old_limits = source.limits();
    source
        .set_limits(
            DomHostedLimits::new(
                old_limits.records(),
                refusal_limit,
                old_limits.scratch_bytes(),
                old_limits.work(),
            ),
            &cancel,
        )
        .unwrap();
    let revision = source.revision().unwrap();
    assert_eq!(
        source.issue_host(&cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::RetainedBytes,
            limit: refusal_limit,
        })
    );
    let state = source.lock().unwrap();
    assert_eq!(state.banks[Role::Host.index()].len(), width);
    assert_eq!(
        state.banks[Role::Host.index()].inspect_storage_bytes(),
        old_bytes
    );
    drop(state);
    assert_eq!(source.revision().unwrap(), revision);
    assert_eq!(retained(&source), fixed + old_bytes);
    source.set_limits(old_limits, &cancel).unwrap();
    let added = source.issue_host(&cancel).unwrap();
    assert_eq!(added.slot, width);
    let state = source.lock().unwrap();
    let actual = state.banks[Role::Host.index()].inspect_storage_bytes();
    assert_eq!(state.banks[Role::Host.index()].len(), width + 1);
    assert_eq!(actual, old_bytes + requested);
    drop(state);
    assert_eq!(retained(&source), fixed + actual);
}

#[test]
fn hosted_work_refusal_and_counter_exhaustion_never_refund_spent_work() {
    let cancel = cancellation();
    let records = record_limits(1, 1);
    let small_source = source(DomHostedLimits::new(records, 1_000_000, 0, 2));
    let work = small_source.work(&cancel).unwrap();
    work.spend(2, &cancel).unwrap();
    assert_eq!(
        work.step(&cancel),
        Err(DomHostedError::WorkLimit { limit: 2 })
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), 2);
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(
        work.spend(0, &cancel),
        Err(DomHostedError::Dom(DomError::Cancelled))
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), 2);
    cancel.store(false, Ordering::Relaxed);
    let large_source = source(DomHostedLimits::new(records, 1_000_000, 0, u64::MAX));
    let work = large_source.work(&cancel).unwrap();
    work.spent.store(u64::MAX - 1, Ordering::Relaxed);
    work.step(&cancel).unwrap();
    assert_eq!(work.spent.load(Ordering::Relaxed), u64::MAX);
    assert_eq!(
        work.step(&cancel),
        Err(DomHostedError::RepresentationExhausted)
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), u64::MAX);
}

// These fixtures exercise the issuer's private phase/role predicate only. The
// checked native Document supplies a real ID and origin; it is not a completed
// hosted entry, nor evidence that a Runtime agent, loop or Global was constructed.
struct IssuerFixture {
    _dom: crate::Dom,
    document: crate::DocumentId,
    origin: crate::DomOpaqueOrigin,
    source: DomHostedSource,
    host: DomHostedHostId,
    context: DomHostedBrowsingContextId,
    work: Option<Arc<Work>>,
}

fn issuer_fixture() -> IssuerFixture {
    let cancel = cancellation();
    let mut dom =
        crate::Dom::new(crate::DomLimits::new(32, 1024, 32, 0, 0, 8, 1_000_000).unwrap()).unwrap();
    let mut batch = crate::DomBatch::new();
    let ticket = batch.create_document();
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&ticket).unwrap();
    let origin = dom
        .snapshot()
        .document(document)
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let mut source = dom
        .register_hosted_source(limits(record_limits(32, 32)), &cancel)
        .unwrap();
    let host = source.issue_host(&cancel).unwrap();
    let prefix = source.prepare_top_level(host, &cancel).unwrap();
    IssuerFixture {
        _dom: dom,
        document,
        origin,
        source,
        host,
        context: prefix.context,
        work: Some(prefix.work),
    }
}

impl IssuerFixture {
    fn work(&self) -> &Arc<Work> {
        self.work
            .as_ref()
            .expect("actual original preparation work")
    }
}

fn begin_request(fixture: &mut IssuerFixture, revisions: u64) -> Request {
    if fixture.work.is_none() {
        let prefix = fixture
            .source
            .prepare_top_level(fixture.host, &cancellation())
            .unwrap();
        fixture.context = prefix.context;
        fixture.work = Some(prefix.work);
    }
    let work = fixture.work().clone();
    let operation = fixture
        .source
        .begin_operation(&work, revisions, 0, &cancellation())
        .unwrap();
    Request {
        key: RequestKey {
            operation,
            phase: 0,
        },
        host: fixture.host,
        context: fixture.context,
        work,
        parent: None,
    }
}

fn advance_issuer(source: &DomHostedSource, request: &mut Request) {
    source
        .permit(request, source.revision().unwrap(), false, &cancellation())
        .unwrap()
        .commit();
    request.key.phase += 1;
}

fn execution_request(fixture: &mut IssuerFixture) -> DomHostedExecutionRequest {
    let mut request = begin_request(fixture, 10);
    advance_issuer(&fixture.source, &mut request);
    let inputs = crate::document::HostedDocumentInputs::form(
        &fixture.source,
        &request.work,
        &fixture.origin,
        &cancellation(),
    )
    .unwrap();
    DomHostedExecutionRequest {
        request,
        origin: fixture.origin.clone(),
        inputs,
    }
}

fn entry_request(fixture: &mut IssuerFixture) -> DomHostedEntryRequest {
    let execution = execution_request(fixture);
    let global = fixture
        .source
        .issue_global(&execution, &cancellation())
        .unwrap();
    let mut request = execution.request;
    // Original issuer acceptance moves Execution -> Active -> Loaded -> Entry.
    // No canonical publication or external service completion is asserted here.
    for _ in 0..3 {
        advance_issuer(&fixture.source, &mut request);
    }
    assert_eq!(request.key.phase, 4);
    DomHostedEntryRequest {
        request,
        document: fixture.document,
        global,
    }
}

fn completed_issuer_roles(
    fixture: &mut IssuerFixture,
) -> (DomHostedGlobalId, DomHostedEntryId, DomHostedNavigableId) {
    let entry_request = entry_request(fixture);
    let entry = fixture
        .source
        .issue_entry(&entry_request, &cancellation())
        .unwrap();
    let navigable = fixture
        .source
        .issue_navigable(&entry_request, &cancellation())
        .unwrap();
    let mut request = entry_request.request;
    advance_issuer(&fixture.source, &mut request);
    fixture
        .source
        .permit(
            &request,
            fixture.source.revision().unwrap(),
            true,
            &cancellation(),
        )
        .unwrap()
        .commit();
    assert_eq!(fixture.source.lock().unwrap().reserved_revisions, 0);
    fixture.work = None;
    (entry_request.global, entry, navigable)
}

#[test]
fn hosted_global_sealing_rejects_a_live_role_from_the_prior_creating_request() {
    let cancel = cancellation();
    let mut fixture = issuer_fixture();
    let (prior, _, _) = completed_issuer_roles(&mut fixture);
    let request = execution_request(&mut fixture);
    let revision = fixture.source.revision().unwrap();
    let bytes = retained(&fixture.source);
    let spent = fixture.work().spent.load(Ordering::Relaxed);
    assert!(
        fixture
            .source
            .observe()
            .validate(prior.source, Role::Global, prior.slot, prior.generation)
            .is_ok()
    );
    assert!(matches!(
        fixture
            .source
            .seal_native_execution(&request, prior, 100, &cancel),
        Err(DomHostedError::StaleIdentity)
    ));
    assert_eq!(fixture.source.revision().unwrap(), revision);
    assert_eq!(retained(&fixture.source), bytes);
    assert_eq!(fixture.work().spent.load(Ordering::Relaxed), spent + 1);
    let current = fixture.source.issue_global(&request, &cancel).unwrap();
    assert_ne!(current, prior);
    let revision = fixture.source.revision().unwrap();
    assert_eq!(
        fixture.source.issue_global(&request, &cancel),
        Err(DomHostedError::WrongPhase)
    );
    assert_eq!(fixture.source.revision().unwrap(), revision);
    let publication = fixture
        .source
        .seal_native_execution(&request, current, 100, &cancel)
        .unwrap();
    assert_eq!(publication.key, request.request.key);
    assert_eq!(publication.revision, revision);
    assert!(
        matches!(publication.value, PublicationValue::NativeExecution { global, navigation_start }
        if global == current && navigation_start.source == fixture.source.id()
            && navigation_start.microseconds() == 100)
    );
}

#[test]
fn hosted_entry_sealing_checks_both_actual_creating_request_qualifications() {
    let cancel = cancellation();
    let mut fixture = issuer_fixture();
    let (_, prior_entry, prior_navigable) = completed_issuer_roles(&mut fixture);
    let request = entry_request(&mut fixture);
    let navigation_key = (*b"00000000-0000-4000-8000-000000000001").map(u16::from);
    let navigation_id = (*b"00000000-0000-4000-8000-000000000002").map(u16::from);
    let mut initialization = Some(DomHostedEntryInitialization::new(
        navigation_key,
        navigation_id,
    ));
    let revision = fixture.source.revision().unwrap();
    assert!(matches!(
        fixture.source.seal_initial_entry(
            &request,
            prior_entry,
            prior_navigable,
            &mut initialization,
            &cancel,
        ),
        Err(DomHostedError::StaleIdentity)
    ));
    assert_eq!(fixture.source.revision().unwrap(), revision);
    assert_eq!(
        initialization.as_ref().unwrap().navigation_key,
        navigation_key
    );
    assert_eq!(
        initialization.as_ref().unwrap().navigation_id,
        navigation_id
    );
    let entry = fixture.source.issue_entry(&request, &cancel).unwrap();
    let navigable = fixture.source.issue_navigable(&request, &cancel).unwrap();
    let revision = fixture.source.revision().unwrap();
    let bytes = retained(&fixture.source);
    for (entry, navigable) in [(prior_entry, navigable), (entry, prior_navigable)] {
        assert!(matches!(
            fixture.source.seal_initial_entry(
                &request,
                entry,
                navigable,
                &mut initialization,
                &cancel
            ),
            Err(DomHostedError::StaleIdentity)
        ));
        assert_eq!(fixture.source.revision().unwrap(), revision);
        assert_eq!(retained(&fixture.source), bytes);
        assert_eq!(
            initialization.as_ref().unwrap().navigation_key,
            navigation_key
        );
        assert_eq!(
            initialization.as_ref().unwrap().navigation_id,
            navigation_id
        );
    }
    assert_eq!(
        fixture.source.issue_entry(&request, &cancel),
        Err(DomHostedError::WrongPhase)
    );
    assert_eq!(
        fixture.source.issue_navigable(&request, &cancel),
        Err(DomHostedError::WrongPhase)
    );
    let publication = fixture
        .source
        .seal_initial_entry(&request, entry, navigable, &mut initialization, &cancel)
        .unwrap();
    assert!(
        initialization.is_none(),
        "successful sealing consumes the original input once"
    );
    assert_eq!(publication.key, request.request.key);
    let PublicationValue::InitialEntry {
        entry: actual_entry,
        navigable: actual_nav,
        initialization: actual_initialization,
        ..
    } = publication.value
    else {
        panic!("actual entry publication");
    };
    assert_eq!(actual_entry, entry);
    assert_eq!(actual_nav, navigable);
    assert_eq!(actual_initialization.navigation_key, navigation_key);
    assert_eq!(actual_initialization.navigation_id, navigation_id);
}

#[test]
fn hosted_phase_permit_drop_and_replay_preserve_the_exact_source_phase() {
    let cancel = cancellation();
    let mut fixture = issuer_fixture();
    let mut request = begin_request(&mut fixture, 3);
    let revision = fixture.source.revision().unwrap();
    let reserved = fixture.source.lock().unwrap().reserved_revisions;
    let spent = fixture.work().spent.load(Ordering::Relaxed);
    drop(
        fixture
            .source
            .permit(&request, revision, false, &cancel)
            .unwrap(),
    );
    assert_eq!(fixture.source.revision().unwrap(), revision);
    assert_eq!(fixture.source.lock().unwrap().reserved_revisions, reserved);
    assert_eq!(fixture.work().spent.load(Ordering::Relaxed), spent + 1);
    fixture
        .source
        .permit(&request, revision, false, &cancel)
        .unwrap()
        .commit();
    assert_eq!(fixture.source.revision().unwrap().get(), revision.get() + 1);
    assert!(matches!(
        fixture
            .source
            .permit(&request, fixture.source.revision().unwrap(), false, &cancel),
        Err(DomHostedError::WrongPhase)
    ));
    request.key.phase = 1;
    assert!(matches!(
        fixture.source.permit(&request, revision, false, &cancel),
        Err(DomHostedError::StaleBasis)
    ));
    fixture
        .source
        .permit(&request, fixture.source.revision().unwrap(), true, &cancel)
        .unwrap()
        .commit();
    assert_eq!(fixture.source.lock().unwrap().reserved_revisions, 0);
    assert!(matches!(
        fixture
            .source
            .permit(&request, fixture.source.revision().unwrap(), true, &cancel),
        Err(DomHostedError::WrongPhase)
    ));
}

#[test]
fn hosted_mandatory_revision_headroom_survives_unrelated_issue_and_retirement() {
    let cancel = cancellation();
    let mut fixture = issuer_fixture();
    fixture.source.lock().unwrap().revision = DomHostedSourceRevision(u64::MAX - 2);
    let request = begin_request(&mut fixture, 2);
    assert_eq!(fixture.source.revision().unwrap().get(), u64::MAX - 1);
    assert_eq!(fixture.source.lock().unwrap().reserved_revisions, 1);
    let bytes = retained(&fixture.source);
    assert_eq!(
        fixture.source.issue_host(&cancel),
        Err(DomHostedError::RepresentationExhausted)
    );
    assert_eq!(
        fixture.source.retire_host(fixture.host, &cancel),
        Err(DomHostedError::RepresentationExhausted)
    );
    assert_eq!(fixture.source.revision().unwrap().get(), u64::MAX - 1);
    assert_eq!(fixture.source.lock().unwrap().reserved_revisions, 1);
    assert_eq!(retained(&fixture.source), bytes);
    fixture
        .source
        .permit(&request, fixture.source.revision().unwrap(), true, &cancel)
        .unwrap()
        .commit();
    assert_eq!(fixture.source.revision().unwrap().get(), u64::MAX);
    assert_eq!(fixture.source.lock().unwrap().reserved_revisions, 0);
    assert!(
        fixture
            .source
            .observe()
            .validate(
                fixture.host.source,
                Role::Host,
                fixture.host.slot,
                fixture.host.generation
            )
            .is_ok()
    );
    let spent = fixture.work().spent.load(Ordering::Relaxed);
    let work = fixture.work().clone();
    assert_eq!(
        fixture.source.begin_operation(&work, 1, 0, &cancel),
        Err(DomHostedError::RepresentationExhausted)
    );
    assert_eq!(fixture.work().spent.load(Ordering::Relaxed), spent);
}

#[test]
fn hosted_failed_operation_reservation_keeps_first_stop_and_spent_work() {
    let cancel = cancellation();
    let mut fixture = issuer_fixture();
    fixture.source.lock().unwrap().revision = DomHostedSourceRevision(u64::MAX - 1);
    let spent = fixture.work().spent.load(Ordering::Relaxed);
    let bytes = retained(&fixture.source);
    let work = fixture.work().clone();
    assert_eq!(
        fixture.source.begin_operation(&work, 3, 0, &cancel),
        Err(DomHostedError::RepresentationExhausted)
    );
    assert_eq!(fixture.source.lock().unwrap().reserved_revisions, 0);
    assert_eq!(
        fixture.source.lock().unwrap().banks[Role::Operation.index()].len(),
        0
    );
    assert_eq!(fixture.work().spent.load(Ordering::Relaxed), spent);
    assert_eq!(retained(&fixture.source), bytes);
}

#[test]
fn hosted_record_refusal_rolls_back_reservation_but_not_original_work() {
    let cancel = cancellation();
    let mut fixture = issuer_fixture();
    let old = fixture.source.limits();
    fixture
        .source
        .set_limits(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 0, 32),
                old.retained_bytes(),
                old.scratch_bytes(),
                old.work(),
            ),
            &cancel,
        )
        .unwrap();
    let revision = fixture.source.revision().unwrap();
    let bytes = retained(&fixture.source);
    let work = fixture.work().clone();
    let spent = work.spent.load(Ordering::Relaxed);
    assert_eq!(
        fixture.source.begin_operation(&work, 3, 0, &cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::Operations,
            limit: 0
        })
    );
    assert_eq!(fixture.source.lock().unwrap().reserved_revisions, 0);
    assert_eq!(
        fixture.source.lock().unwrap().banks[Role::Operation.index()].inspect_storage_bytes(),
        0
    );
    assert_eq!(fixture.source.revision().unwrap(), revision);
    assert_eq!(work.spent.load(Ordering::Relaxed), spent + 1);
    assert_eq!(retained(&fixture.source), bytes);
}

#[test]
fn hosted_limit_lowering_preserves_the_original_cursor_and_exact_live_usage() {
    let cancel = cancellation();
    let mut fixture = issuer_fixture();
    let old = fixture.source.limits();
    let work = fixture.work().clone();
    let spent = work.spent.load(Ordering::Relaxed);
    let bytes = retained(&fixture.source);
    let revision = fixture.source.revision().unwrap();
    assert!(spent > 0);
    for (replacement, expected) in [
        (
            DomHostedLimits::new(
                old.records(),
                old.retained_bytes(),
                old.scratch_bytes(),
                spent - 1,
            ),
            DomHostedError::WorkLimit { limit: spent - 1 },
        ),
        (
            DomHostedLimits::new(old.records(), bytes - 1, old.scratch_bytes(), old.work()),
            DomHostedError::Limit {
                resource: DomHostedResource::RetainedBytes,
                limit: bytes - 1,
            },
        ),
        (
            DomHostedLimits::new(
                DomHostedRecordLimits::new(0, 32, 32, 32, 32, 32, 32),
                old.retained_bytes(),
                old.scratch_bytes(),
                old.work(),
            ),
            DomHostedError::Limit {
                resource: DomHostedResource::Hosts,
                limit: 0,
            },
        ),
        (
            DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 0, 32, 32, 32, 32, 32),
                old.retained_bytes(),
                old.scratch_bytes(),
                old.work(),
            ),
            DomHostedError::Limit {
                resource: DomHostedResource::BrowsingContexts,
                limit: 0,
            },
        ),
    ] {
        assert_eq!(
            fixture.source.set_limits(replacement, &cancel),
            Err(expected)
        );
        assert_eq!(fixture.source.limits(), old);
        assert_eq!(work.limit.load(Ordering::Relaxed), old.work());
        assert_eq!(work.spent.load(Ordering::Relaxed), spent);
        assert_eq!(fixture.source.revision().unwrap(), revision);
        assert_eq!(retained(&fixture.source), bytes);
    }
    let equality = DomHostedLimits::new(
        DomHostedRecordLimits::new(1, 1, 0, 0, 0, 0, 0),
        bytes,
        0,
        spent,
    );
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(
        fixture.source.set_limits(equality, &cancel),
        Err(DomHostedError::Dom(DomError::Cancelled))
    );
    assert_eq!(fixture.source.limits(), old);
    cancel.store(false, Ordering::Relaxed);
    fixture.source.set_limits(equality, &cancel).unwrap();
    assert_eq!(fixture.source.remaining_bytes().unwrap(), 0);
    assert_eq!(
        work.step(&cancel),
        Err(DomHostedError::WorkLimit { limit: spent })
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), spent);
    fixture
        .source
        .set_limits(
            DomHostedLimits::new(equality.records(), bytes, 0, spent + 1),
            &cancel,
        )
        .unwrap();
    work.step(&cancel).unwrap();
    assert_eq!(work.spent.load(Ordering::Relaxed), spent + 1);
    assert_eq!(fixture.source.revision().unwrap(), revision);
    assert!(
        fixture
            .source
            .observe()
            .validate(
                fixture.host.source,
                Role::Host,
                fixture.host.slot,
                fixture.host.generation
            )
            .is_ok()
    );
    assert!(
        fixture
            .source
            .observe()
            .validate(
                fixture.context.source,
                Role::BrowsingContext,
                fixture.context.slot,
                fixture.context.generation
            )
            .is_ok()
    );
}

#[test]
fn hosted_foreign_request_cannot_issue_a_role_or_consume_this_phase() {
    let cancel = cancellation();
    let mut first = issuer_fixture();
    let mut second = issuer_fixture();
    let request = execution_request(&mut first);
    let foreign = execution_request(&mut second);
    let revision = first.source.revision().unwrap();
    let other_revision = second.source.revision().unwrap();
    assert_eq!(
        first.source.issue_global(&foreign, &cancel),
        Err(DomHostedError::ForeignSource)
    );
    assert_eq!(first.source.revision().unwrap(), revision);
    assert_eq!(second.source.revision().unwrap(), other_revision);
    let global = first.source.issue_global(&request, &cancel).unwrap();
    assert_eq!(global.source(), first.source.id());
    let other = second.source.issue_global(&foreign, &cancel).unwrap();
    assert_eq!(other.source(), second.source.id());
}

#[test]
fn hosted_registry_quota_is_live_custody_and_limit_lowering_is_atomic() {
    let cancel = cancellation();
    let mut source = source(DomHostedLimits::new(
        DomHostedRecordLimits::new(0, 0, 0, 0, 0, 0, 1),
        1_000_000,
        0,
        0,
    ));
    let old = source.limits();
    let quota = source.retain_registry(&cancel).unwrap();
    assert_eq!(source.shared.registries.load(Ordering::Relaxed), 1);
    assert!(matches!(
        source.retain_registry(&cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::Registries,
            limit: 1,
        })
    ));
    let zero = DomHostedLimits::new(
        DomHostedRecordLimits::new(0, 0, 0, 0, 0, 0, 0),
        old.retained_bytes(),
        0,
        0,
    );
    assert_eq!(
        source.set_limits(zero, &cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::Registries,
            limit: 0,
        })
    );
    assert_eq!(source.limits(), old);
    drop(quota);
    assert_eq!(source.shared.registries.load(Ordering::Relaxed), 0);
    let replacement = source.retain_registry(&cancel).unwrap();
    assert_eq!(source.shared.registries.load(Ordering::Relaxed), 1);
    drop(replacement);
    source.set_limits(zero, &cancel).unwrap();
    assert!(matches!(
        source.retain_registry(&cancel),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::Registries,
            limit: 0,
        })
    ));
}

#[test]
fn hosted_destruction_preserves_the_original_prepublication_work_lane() {
    let cancel = cancellation();
    let mut fixture = issuer_fixture();
    let (_, _, navigable) = completed_issuer_roles(&mut fixture);
    let prefix = fixture
        .source
        .prepare_top_level(fixture.host, &cancel)
        .unwrap();
    let context = prefix.browsing_context();
    let work = prefix.work.clone();
    let spent = work.spent.load(Ordering::Relaxed);
    let limits = fixture.source.limits();
    let bytes = retained(&fixture.source);
    let revision = fixture.source.revision().unwrap();
    let stamp = fixture
        ._dom
        .snapshot()
        .document(fixture.document)
        .unwrap()
        .stamp();
    assert!(
        fixture
            .source
            .observe()
            .validate(
                navigable.source,
                Role::Navigable,
                navigable.slot,
                navigable.generation
            )
            .is_ok()
    );
    fixture
        .source
        .set_limits(
            DomHostedLimits::new(
                limits.records(),
                limits.retained_bytes(),
                limits.scratch_bytes(),
                spent,
            ),
            &cancel,
        )
        .unwrap();
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        fixture
            ._dom
            .begin_hosted_top_level_destruction(&mut fixture.source, navigable, &cancel),
        Err(DomHostedError::Dom(DomError::Cancelled))
    ));
    cancel.store(false, Ordering::Relaxed);
    // The live source-issued Nav passes qualification. The original preparation
    // still owns the work lane, so the lower front must stop before scanning
    // this native identity payload as if it were a completed hosted Document.
    assert!(matches!(
        fixture
            ._dom
            .begin_hosted_top_level_destruction(&mut fixture.source, navigable, &cancel),
        Err(DomHostedError::IncompleteOperation)
    ));
    assert!(Arc::ptr_eq(
        &fixture.source.active_work.upgrade().unwrap(),
        &work
    ));
    assert!(Arc::ptr_eq(&prefix.work, &work));
    assert_eq!(prefix.browsing_context(), context);
    assert_eq!(fixture.source.revision().unwrap(), revision);
    assert_eq!(
        fixture
            ._dom
            .snapshot()
            .document(fixture.document)
            .unwrap()
            .stamp(),
        stamp
    );
    assert_eq!(retained(&fixture.source), bytes);
    assert_eq!(work.spent.load(Ordering::Relaxed), spent);
    assert_eq!(work.limit.load(Ordering::Relaxed), spent);
    fixture
        .source
        .set_limits(
            DomHostedLimits::new(
                limits.records(),
                limits.retained_bytes(),
                limits.scratch_bytes(),
                spent + 1,
            ),
            &cancel,
        )
        .unwrap();
    work.step(&cancel).unwrap();
    assert_eq!(work.spent.load(Ordering::Relaxed), spent + 1);
    assert!(Arc::ptr_eq(
        &fixture.source.active_work.upgrade().unwrap(),
        &work
    ));
    assert_eq!(fixture.source.revision().unwrap(), revision);
    assert_eq!(retained(&fixture.source), bytes);
}

#[test]
fn hosted_queued_role_work_limits_change_only_after_every_original_work_accepts() {
    let cancel = cancellation();
    let mut source = source(limits(record_limits(1, 1)));
    let first = source.work(&cancel).unwrap();
    let first_operation = source.begin_operation(&first, 2, 0, &cancel).unwrap();
    let second = source.work(&cancel).unwrap();
    let second_operation = source.begin_operation(&second, 2, 0, &cancel).unwrap();
    assert!(source.active_work.upgrade().is_none());
    while second.spent.load(Ordering::Relaxed) <= first.spent.load(Ordering::Relaxed) + 1 {
        second.step(&cancel).unwrap();
    }
    let first_spent = first.spent.load(Ordering::Relaxed);
    let second_spent = second.spent.load(Ordering::Relaxed);
    let original = source.limits();
    let bytes = retained(&source);
    let revision = source.revision().unwrap();
    let with_work = |work| {
        DomHostedLimits::new(
            original.records(),
            original.retained_bytes(),
            original.scratch_bytes(),
            work,
        )
    };

    // The earlier row accepts this cap, but the later original Work refuses.
    // Neither original counter may be updated on that partial validation path.
    assert_eq!(
        source.set_limits(with_work(second_spent - 1), &cancel),
        Err(DomHostedError::WorkLimit {
            limit: second_spent - 1,
        })
    );
    assert_eq!(source.limits(), original);
    assert_eq!(first.limit.load(Ordering::Relaxed), original.work());
    assert_eq!(second.limit.load(Ordering::Relaxed), original.work());
    assert_eq!(first.spent.load(Ordering::Relaxed), first_spent);
    assert_eq!(second.spent.load(Ordering::Relaxed), second_spent);
    assert_eq!(retained(&source), bytes);
    assert_eq!(source.revision().unwrap(), revision);

    source.set_limits(with_work(second_spent), &cancel).unwrap();
    assert_eq!(first.limit.load(Ordering::Relaxed), second_spent);
    assert_eq!(second.limit.load(Ordering::Relaxed), second_spent);
    assert_eq!(
        second.step(&cancel),
        Err(DomHostedError::WorkLimit {
            limit: second_spent
        })
    );
    assert_eq!(second.spent.load(Ordering::Relaxed), second_spent);
    source
        .set_limits(with_work(second_spent + 1), &cancel)
        .unwrap();
    second.step(&cancel).unwrap();
    assert_eq!(second.spent.load(Ordering::Relaxed), second_spent + 1);
    assert_eq!(first.spent.load(Ordering::Relaxed), first_spent);
    for operation in [first_operation, second_operation] {
        source
            .observe()
            .validate(
                operation.source,
                Role::Operation,
                operation.slot,
                operation.generation,
            )
            .unwrap();
    }
    assert_eq!(retained(&source), bytes);
    assert_eq!(source.revision().unwrap(), revision);
}

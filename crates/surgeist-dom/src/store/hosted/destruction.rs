//! One invocation's captured descendants and the one actual per-D cleanup.
//! Both are rows of the original issuer/controller; they share original Work.
use super::*;

pub(in crate::store) struct Invocation {
    pub(super) request: DomHostedDestructionInvocationRequest,
    waiters: Vec<Waiter>,
    history: HistoryDependency,
    retiring_reserved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html_semantics::HtmlAboutBaseUrl;

    fn cancel() -> AtomicBool {
        AtomicBool::new(false)
    }

    struct Fixture {
        dom: Dom,
        source: DomHostedSource,
        host: DomHostedHostId,
        documents: [DocumentId; 2],
        container: NodeId,
    }
    impl Fixture {
        fn new() -> Self {
            let c = cancel();
            let mut dom =
                Dom::new(DomLimits::new(64, 4096, 64, 0, 0, 16, 4_000_000).unwrap()).unwrap();
            let mut batch = DomBatch::new();
            let parent = batch.create_document();
            let child = batch.create_document();
            let container = batch.create_element_ns(&parent, crate::Namespace::html(), "iframe");
            batch.insert_before(&container, &parent, None);
            let commit = dom.apply(batch, &c).unwrap();
            let documents = [
                commit.document(&parent).unwrap(),
                commit.document(&child).unwrap(),
            ];
            let container = commit.created(&container).unwrap().unwrap();
            drop(commit);
            let mut source = dom
                .register_hosted_source(
                    DomHostedLimits::new(
                        DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                        16_000_000,
                        16_000_000,
                        10_000_000,
                    ),
                    &c,
                )
                .unwrap();
            let host = source.issue_host(&c).unwrap();
            Self {
                dom,
                source,
                host,
                documents,
                container,
            }
        }
        fn issue(&mut self, document: usize) -> (DomHostedCreation, Arc<Work>) {
            let c = cancel();
            let prefix = self.source.prepare_top_level(self.host, &c).unwrap();
            let work = prefix.work;
            let operation = self.source.begin_operation(&work, 10, 0, &c).unwrap();
            let origin = self
                .dom
                .state
                .node(self.dom.owner, self.documents[document].node())
                .unwrap();
            let DomNodeKind::Document(metadata) = &origin.kind else {
                panic!("actual native Document payload")
            };
            let origin = metadata.origin().clone();
            let mut request = Request {
                key: RequestKey {
                    operation,
                    phase: 0,
                },
                host: self.host,
                context: prefix.context,
                work: work.clone(),
                parent: None,
            };
            advance(&self.source, &mut request);
            let inputs =
                crate::document::HostedDocumentInputs::form(&self.source, &work, &origin, &c)
                    .unwrap();
            let execution = DomHostedExecutionRequest {
                request,
                origin,
                inputs,
            };
            let global = self.source.issue_global(&execution, &c).unwrap();
            let mut request = execution.request;
            for _ in 0..3 {
                advance(&self.source, &mut request);
            }
            let entry = DomHostedEntryRequest {
                request,
                document: self.documents[document],
                global,
            };
            let entry_id = self.source.issue_entry(&entry, &c).unwrap();
            let navigable = self.source.issue_navigable(&entry, &c).unwrap();
            let mut request = entry.request;
            advance(&self.source, &mut request);
            self.source
                .permit(&request, self.source.revision().unwrap(), true, &c)
                .unwrap()
                .commit();
            (
                DomHostedCreation {
                    document: self.documents[document],
                    host: self.host,
                    context: request.context,
                    global,
                    entry: entry_id,
                    navigable,
                },
                work,
            )
        }
        fn used(&self) -> usize {
            self.source.limits().retained_bytes() - self.source.remaining_bytes().unwrap()
        }
        fn limits(&mut self, bytes: usize, scratch: usize) {
            let limits = self.source.limits();
            self.source
                .set_limits(
                    DomHostedLimits::new(limits.records(), bytes, scratch, limits.work()),
                    &cancel(),
                )
                .unwrap();
        }
    }
    fn advance(source: &DomHostedSource, request: &mut Request) {
        source
            .permit(request, source.revision().unwrap(), false, &cancel())
            .unwrap()
            .commit();
        request.key.phase += 1;
    }

    fn fixture_with_child() -> (Fixture, DomHostedCreation, DomHostedCreation, Arc<Work>) {
        // This is a preparation fixture with real issuer identities and native
        // Document payloads. It establishes no hosted activity, Global service,
        // cleanup or successful destruction observation.
        let c = cancel();
        let mut f = Fixture::new();
        let (parent, work) = f.issue(0);
        drop(work);
        let (child, work) = f.issue(1);
        let origin = match &f
            .dom
            .state
            .node(f.dom.owner, parent.document.node())
            .unwrap()
            .kind
        {
            DomNodeKind::Document(metadata) => metadata.origin().clone(),
            _ => panic!("actual Document"),
        };
        let name = DocumentString::from("");
        let about = HtmlAboutBaseUrl::KnownNull;
        let key: [u16; 36] = "00000000-0000-4000-8000-000000000000"
            .encode_utf16()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let input = |creation, parent| history::Initial {
            creation,
            initialization: DomHostedEntryInitialization::new(key, key),
            origin: origin.clone(),
            parent,
            target_name: Some(&name),
            about_base_url: &about,
            initiator_origin: None,
        };
        let context = history::Context::new(&f.source, &work, &c, 0);
        let graph = history::Graph::initial(None, input(parent, None), &context).unwrap();
        let graph = history::Graph::initial(
            Some(&graph),
            input(child, Some((parent.navigable, f.container))),
            &context,
        )
        .unwrap();
        let state = Arc::get_mut(&mut f.dom.state).unwrap();
        state.hosted_history = Some(graph);
        let DomNodeKind::Element(element) =
            &mut state.slots[f.container.slot].node.as_mut().unwrap().kind
        else {
            panic!("native Iframe container")
        };
        element
            .establish_child_navigable(Some(child.navigable))
            .unwrap();
        (f, parent, child, work)
    }

    #[test]
    fn invocation_scratch_keeps_captured_children_and_waiters_until_bank_transfer() {
        let c = cancel();
        let (mut f, parent, child, work) = fixture_with_child();
        let prepared = f
            .dom
            .prepare_hosted_invocation(&f.source, parent, &work, &c)
            .unwrap();
        assert_eq!(prepared.children.values.len(), 1);
        assert_eq!(prepared.children.values[0], child);
        assert_eq!(prepared.waiters.len(), 1);
        // Independently count actual owner headers and represented capacities;
        // no retained/scratch meter supplies this expectation. Old canonical
        // tree, graph and operation backing are not new formation candidates.
        let capture = size_of::<CapturedChildren>()
            + prepared.children.values.capacity() * size_of::<DomHostedCreation>();
        let waiters = prepared.waiters.capacity() * size_of::<Waiter>();
        let bank =
            prepared.bank.candidate_capacity().unwrap() * size_of::<Option<Box<Operation>>>();
        // The Vec and PreparedRows headers are already inline here, including
        // PreparedRows' Option<Operations>. Count each physical header once.
        let header = size_of::<PreparedInvocation>();
        let exact = header + capture + waiters + bank;
        drop(prepared);
        let limits = f.source.limits();
        let used = f.used();
        let revision = f.source.revision().unwrap();
        let canonical = f.dom.revision;
        let graph = f.dom.state.hosted_history.as_ref().unwrap().clone();
        let spent = work.spent.load(Ordering::Relaxed);
        f.limits(limits.retained_bytes(), exact);
        let control = f
            .dom
            .prepare_hosted_invocation(&f.source, parent, &work, &c)
            .unwrap();
        assert_eq!(control.children.values.as_slice(), &[child]);
        drop(control);
        assert_eq!(f.used(), used);
        assert_eq!(f.source.revision().unwrap(), revision);
        assert_eq!(f.dom.revision, canonical);
        eprintln!(
            "exact physical formation control succeeded: header={header}, capture={capture}, waiter_backing={waiters}, bank_backing={bank}, exact={exact}"
        );
        f.limits(limits.retained_bytes(), exact - 1);
        match f
            .dom
            .prepare_hosted_invocation(&f.source, parent, &work, &c)
        {
            Err(DomHostedError::Limit {
                resource: DomHostedResource::ScratchBytes,
                limit,
            }) if limit == exact - 1 => {}
            Ok(prepared) => panic!(
                "one-short scratch admitted the complete candidate: header={header}, capture={capture}, waiter_backing={waiters}, bank_backing={bank}, exact={exact}, child_capacity={}, waiter_capacity={}, bank_capacity={:?}",
                prepared.children.values.capacity(),
                prepared.waiters.capacity(),
                prepared.bank.candidate_capacity(),
            ),
            Err(error) => panic!("one-short scratch returned an unrelated error: {error:?}"),
        }
        assert_eq!(f.used(), used);
        assert_eq!(f.source.revision().unwrap(), revision);
        assert_eq!(f.dom.revision, canonical);
        assert!(Arc::ptr_eq(
            f.dom.state.hosted_history.as_ref().unwrap(),
            &graph
        ));
        assert!(work.spent.load(Ordering::Relaxed) > spent);
        f.limits(limits.retained_bytes(), exact);
        for _ in 0..2 {
            let prepared = f
                .dom
                .prepare_hosted_invocation(&f.source, parent, &work, &c)
                .unwrap();
            assert_eq!(prepared.children.values.as_slice(), &[child]);
            assert!(
                prepared
                    .waiters
                    .iter()
                    .all(|waiter| waiter.invocation.is_none() && !waiter.settled)
            );
            drop(prepared);
            assert_eq!(f.used(), used);
            assert_eq!(f.source.revision().unwrap(), revision);
            assert_eq!(f.dom.revision, canonical);
        }
        let retained_candidate =
            capture + size_of::<Operation>() + waiters + size_of::<operations::Operations>() + bank;
        f.limits(used + retained_candidate - 1, exact);
        assert!(matches!(
            f.dom.prepare_hosted_invocation(&f.source, parent, &work, &c),
            Err(DomHostedError::Limit { resource: DomHostedResource::RetainedBytes, limit })
                if limit == used + retained_candidate - 1
        ));
        assert_eq!(f.used(), used);
        assert_eq!(f.source.revision().unwrap(), revision);
        f.limits(used + retained_candidate, exact);
        let prepared = f
            .dom
            .prepare_hosted_invocation(&f.source, parent, &work, &c)
            .unwrap();
        assert_eq!(f.used(), used + retained_candidate);
        drop(prepared);
        assert_eq!(f.used(), used);
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            f.dom
                .prepare_hosted_invocation(&f.source, parent, &work, &cancelled),
            Err(DomHostedError::Dom(DomError::Cancelled))
        ));
        assert_eq!(f.used(), used);
        assert_eq!(f.source.revision().unwrap(), revision);
        assert_eq!(f.dom.revision, canonical);
    }

    #[test]
    fn invocation_install_admits_the_final_operation_before_issuing_its_role() {
        let c = cancel();
        let (mut f, parent, child, work) = fixture_with_child();
        let prepared = f
            .dom
            .prepare_hosted_invocation(&f.source, parent, &work, &c)
            .unwrap();
        let capture = size_of::<CapturedChildren>()
            + prepared.children.values.capacity() * size_of::<DomHostedCreation>();
        let waiters = prepared.waiters.capacity() * size_of::<Waiter>();
        let bank =
            prepared.bank.candidate_capacity().unwrap() * size_of::<Option<Box<Operation>>>();
        // At this transfer the real Operation replaces PreparedInvocation.
        // Its inline waiter Vec is counted once inside Operation; PreparedRows
        // still owns the candidate controller until insert_prepared consumes it.
        let exact = size_of::<Operation>()
            + size_of::<operations::PreparedRows>()
            + capture
            + waiters
            + bank;
        let preparation = size_of::<PreparedInvocation>() + capture + waiters + bank;
        assert!(exact > preparation);
        drop(prepared);
        let limits = f.source.limits();
        let used = f.used();
        let revision = f.source.revision().unwrap();
        let canonical = f.dom.revision;
        let graph = f.dom.state.hosted_history.as_ref().unwrap().clone();
        // The fixture's completed issuer operation left an actual reusable
        // slot, so this case isolates final payload admission from issuer growth.
        assert_eq!(f.source.next_operation_slot(&work, &c).unwrap(), 0);
        f.limits(limits.retained_bytes(), exact - 1);
        assert!(matches!(
            f.dom.install_hosted_invocation(&mut f.source, parent, parent.navigable, work.clone(), None, &c),
            Err(DomHostedError::Limit { resource: DomHostedResource::ScratchBytes, limit })
                if limit == exact - 1
        ));
        assert_eq!(f.used(), used);
        assert_eq!(f.source.revision().unwrap(), revision);
        assert_eq!(f.dom.revision, canonical);
        assert!(Arc::ptr_eq(
            f.dom.state.hosted_history.as_ref().unwrap(),
            &graph
        ));
        f.limits(limits.retained_bytes(), exact);
        let operation = f
            .dom
            .install_hosted_invocation(
                &mut f.source,
                parent,
                parent.navigable,
                work.clone(),
                None,
                &c,
            )
            .unwrap();
        let row = f.dom.hosted_operations.get(operation).unwrap();
        let CreationPhase::Invocation(invocation) = &row.phase else {
            panic!("actual installed invocation")
        };
        assert_eq!(invocation.request.creation, parent);
        assert_eq!(invocation.request.children.values.as_slice(), &[child]);
        assert_eq!(invocation.request.cleanup, None);
        assert!(invocation.history == HistoryDependency::None);
        assert!(Arc::ptr_eq(&invocation.request.request.work, &work));
        assert_eq!(f.dom.revision, canonical);
        assert!(Arc::ptr_eq(
            f.dom.state.hosted_history.as_ref().unwrap(),
            &graph
        ));
        f.source
            .permit(row.request(), f.source.revision().unwrap(), true, &c)
            .unwrap()
            .commit();
        f.dom.hosted_operations.remove(operation).unwrap();
        // The installed controller backing remains owned and charged after its
        // row is removed; all per-invocation candidate backing has been released.
        assert_eq!(f.used(), used + size_of::<operations::Operations>() + bank);
    }

    #[test]
    fn cleanup_scratch_admits_the_final_operation_before_issuance() {
        // Exercise only the real cleanup installation phase with actual issuer
        // identities and native payloads. No Global service, retiring state or
        // successful lifecycle cleanup is synthesized by this fixture.
        fn fixture() -> (Fixture, DomHostedOperationId, Arc<Work>) {
            let (mut f, parent, _, work) = fixture_with_child();
            let invocation = f
                .dom
                .install_hosted_invocation(
                    &mut f.source,
                    parent,
                    parent.navigable,
                    work.clone(),
                    None,
                    &cancel(),
                )
                .unwrap();
            (f, invocation, work)
        }
        let c = cancel();
        let capacity = {
            let (f, _, work) = fixture();
            let bank = f
                .dom
                .hosted_operations
                .prepare(
                    f.source.next_operation_slot(&work, &c).unwrap(),
                    &work,
                    &f.source,
                    0,
                    None,
                    &c,
                )
                .unwrap();
            bank.candidate_capacity().unwrap()
        };
        let row_backing = capacity * size_of::<Option<Box<Operation>>>();
        let exact = size_of::<Operation>() + size_of::<operations::PreparedRows>() + row_backing;
        for allowance in [exact, exact - 1] {
            let (mut f, invocation, work) = fixture();
            let limits = f.source.limits();
            f.limits(limits.retained_bytes(), allowance);
            let retained = f.used();
            let source_revision = f.source.revision().unwrap();
            let canonical_revision = f.dom.revision;
            let reserved = f.dom.hosted_revisions;
            let spent = work.spent.load(Ordering::Relaxed);
            let result = f.dom.with_hosted_operation(invocation, |dom, operation| {
                let CreationPhase::Invocation(row) = &operation.phase else {
                    panic!("actual original invocation row")
                };
                let cleanup = dom.install_hosted_cleanup(
                    &mut f.source,
                    invocation,
                    row,
                    &operation.origin,
                    &c,
                )?;
                Ok(OperationResult::Retain(cleanup))
            });
            match result {
                Ok(cleanup) if allowance == exact => {
                    assert_eq!(f.dom.hosted_operations.row_capacity(), capacity);
                    let original = f.dom.hosted_operations.get(invocation).unwrap();
                    let CreationPhase::Invocation(original) = &original.phase else {
                        panic!("original invocation remains owned")
                    };
                    let row = f.dom.hosted_operations.get(cleanup).unwrap();
                    let CreationPhase::DestroyEnqueue(request) = &row.phase else {
                        panic!("actual installed cleanup row")
                    };
                    assert_eq!(request.invocation, invocation);
                    assert_eq!(request.creation, original.request.creation);
                    assert!(Arc::ptr_eq(&request.children, &original.request.children));
                    assert!(Arc::ptr_eq(&request.request.work, &work));
                    f.source
                        .observe()
                        .validate(
                            cleanup.source,
                            Role::Operation,
                            cleanup.slot,
                            cleanup.generation,
                        )
                        .unwrap();
                    assert_eq!(f.dom.revision, canonical_revision);
                    assert_eq!(f.dom.hosted_revisions, reserved + 1);
                    eprintln!(
                        "exact cleanup installation succeeded: operation_header={}, prepared_header={}, row_backing={row_backing}, exact={exact}",
                        size_of::<Operation>(),
                        size_of::<operations::PreparedRows>()
                    );
                }
                Err(DomHostedError::Limit {
                    resource: DomHostedResource::ScratchBytes,
                    limit,
                }) if allowance < exact && limit == allowance => {
                    assert_eq!(f.used(), retained);
                    assert_eq!(f.source.revision().unwrap(), source_revision);
                    assert_eq!(f.dom.revision, canonical_revision);
                    assert_eq!(f.dom.hosted_revisions, reserved);
                    let row = f.dom.hosted_operations.get(invocation).unwrap();
                    let CreationPhase::Invocation(row) = &row.phase else {
                        panic!("refusal preserves the original invocation")
                    };
                    assert_eq!(row.request.cleanup, None);
                    assert!(Arc::ptr_eq(&row.request.request.work, &work));
                }
                Ok(cleanup) => panic!(
                    "one-short final cleanup payload admitted: operation={cleanup:?}, operation_header={}, prepared_header={}, row_backing={row_backing}, exact={exact}",
                    size_of::<Operation>(),
                    size_of::<operations::PreparedRows>()
                ),
                Err(error) => panic!(
                    "cleanup installation returned an unrelated error at allowance {allowance}: {error:?}"
                ),
            }
            assert!(work.spent.load(Ordering::Relaxed) > spent);
        }
    }
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum HistoryDependency {
    None,
    Pending(DomHostedOperationId),
    Settled(DomHostedOperationId),
}
#[derive(Clone, Copy)]
struct Waiter {
    invocation: Option<DomHostedOperationId>,
    settled: bool,
}

/// Private preparation has no issued invocation role or canonical publication.
struct PreparedInvocation {
    children: Arc<CapturedChildren>,
    waiters: Vec<Waiter>,
    origin: crate::DomOpaqueOrigin,
    bank: operations::PreparedRows,
    retained: Retained,
}

impl PreparedInvocation {
    fn backing_bytes(&self) -> Result<usize, DomHostedError> {
        let waiters = self
            .waiters
            .capacity()
            .checked_mul(size_of::<Waiter>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let bank = self.bank.backing_bytes()?;
        self.children
            .values
            .capacity()
            .checked_mul(size_of::<DomHostedCreation>())
            .and_then(|bytes| bytes.checked_add(size_of::<CapturedChildren>()))
            .and_then(|bytes| bytes.checked_add(waiters))
            .and_then(|bytes| bytes.checked_add(bank))
            .ok_or(DomHostedError::RepresentationExhausted)
    }
    fn scratch_bytes(&self) -> Result<usize, DomHostedError> {
        size_of::<Self>()
            .checked_add(self.backing_bytes()?)
            .ok_or(DomHostedError::RepresentationExhausted)
    }
    fn operation_scratch_bytes(&self) -> Result<usize, DomHostedError> {
        let backing = self.backing_bytes()?;
        size_of::<Operation>()
            .checked_add(size_of::<operations::PreparedRows>())
            .and_then(|bytes| bytes.checked_add(backing))
            .ok_or(DomHostedError::RepresentationExhausted)
    }
}

impl Dom {
    fn terminal_hosted_proof(
        &self,
        source: &DomHostedSource,
        creation: DomHostedCreation,
    ) -> Result<Option<DomHostedDocumentDestructionProof>, DomHostedError> {
        let DomNodeKind::Document(metadata) =
            &self.state.node(self.owner, creation.document.node())?.kind
        else {
            return Err(DomError::WrongDocument.into());
        };
        let environment = metadata.document_environment();
        let association = environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        if association.global != creation.global || association.source != source.observe() {
            return Err(DomHostedError::ForeignSource);
        }
        Ok(match association.phase {
            DocumentPhase::Retired { operation } => Some(DomHostedDocumentDestructionProof {
                source: source.id(),
                document: creation.document,
                global: creation.global,
                operation,
            }),
            _ => None,
        })
    }
    pub fn begin_hosted_top_level_destruction(
        &mut self,
        source: &mut DomHostedSource,
        navigable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedDestructionInvocation, DomHostedError> {
        check_cancel(cancel)?;
        self.require_source_idle()?;
        self.validate_hosted_source(source)?;
        source.observe().validate(
            navigable.source,
            Role::Navigable,
            navigable.slot,
            navigable.generation,
        )?;
        if source.active_work.upgrade().is_some() {
            return Err(DomHostedError::IncompleteOperation);
        }
        let work = source.work(cancel)?;
        let graph = self
            .state
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        work.step(cancel)?;
        let nav = graph.navigable_with(navigable, || work.step(cancel))?;
        if nav.parent.is_some() {
            return Err(DomHostedError::UnsupportedProfile);
        }
        let document = graph
            .entry_with(nav.active, || work.step(cancel))?
            .document
            .ok_or(DomHostedError::StaleIdentity)?;
        let creation = DomHostedCreation {
            document,
            host: nav.host,
            context: nav.context,
            global: nav.global,
            entry: nav.active,
            navigable,
        };
        self.begin_hosted_invocation(source, creation, nav.traversable, work, None, cancel)
    }
    pub(in crate::store) fn begin_hosted_invocation(
        &mut self,
        source: &mut DomHostedSource,
        creation: DomHostedCreation,
        traversable: DomHostedNavigableId,
        work: Arc<Work>,
        history: Option<DomHostedOperationId>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedDestructionInvocation, DomHostedError> {
        self.validate_hosted_source(source)?;
        work.step(cancel)?;
        // Retry this exact retained invocation instead of replacing its original
        // prefix or spent Work after an admission refusal.
        if let Some(id)=self.hosted_operations.find(&work,cancel,|operation|matches!(&operation.phase,CreationPhase::Invocation(row) if row.request.creation==creation && row.history==history.map_or(HistoryDependency::None,HistoryDependency::Pending) && row.request.cleanup.is_none()))? {
            return self.complete_hosted_invocation(source,id,cancel);
        }
        let operation =
            self.install_hosted_invocation(source, creation, traversable, work, history, cancel)?;
        self.complete_hosted_invocation(source, operation, cancel)
    }
    fn install_hosted_invocation(
        &mut self,
        source: &mut DomHostedSource,
        creation: DomHostedCreation,
        traversable: DomHostedNavigableId,
        work: Arc<Work>,
        history: Option<DomHostedOperationId>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedOperationId, DomHostedError> {
        let prepared = self.prepare_hosted_invocation(source, creation, &work, cancel)?;
        // Both actual phase peaks are admitted before the original issuer can
        // publish a role: its replacement while preparation lives, then the
        // boxed Operation while the prepared controller bank still lives.
        let scratch = prepared.scratch_bytes()?;
        crate::hosted::admit(
            prepared.operation_scratch_bytes()?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let operation = source.begin_operation(&work, 2, scratch, cancel)?;
        let PreparedInvocation {
            children,
            waiters,
            origin,
            bank,
            retained,
        } = prepared;
        let request = Request {
            key: RequestKey {
                operation,
                phase: 0,
            },
            host: creation.host,
            context: creation.context,
            work,
            parent: None,
        };
        self.hosted_operations.insert_prepared(
            bank,
            Operation {
                origin,
                policy: None,
                phase: CreationPhase::Invocation(Invocation {
                    request: DomHostedDestructionInvocationRequest {
                        request,
                        creation,
                        traversable,
                        cleanup: None,
                        children,
                    },
                    waiters,
                    history: history.map_or(HistoryDependency::None, HistoryDependency::Pending),
                    retiring_reserved: false,
                }),
                _retained: retained,
            },
            false,
        );
        Ok(operation)
    }
    fn prepare_hosted_invocation(
        &self,
        source: &DomHostedSource,
        creation: DomHostedCreation,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<PreparedInvocation, DomHostedError> {
        let header = size_of::<PreparedInvocation>();
        let children = self.capture_hosted_children(source, creation, work, header, cancel)?;
        let captured = children
            .values
            .capacity()
            .checked_mul(size_of::<DomHostedCreation>())
            .and_then(|bytes| bytes.checked_add(size_of::<CapturedChildren>()))
            .and_then(|bytes| bytes.checked_add(header))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let requested = children
            .values
            .len()
            .checked_mul(size_of::<Waiter>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            captured
                .checked_add(requested)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let mut retained = source.retain(
            size_of::<Operation>()
                .checked_add(requested)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            cancel,
        )?;
        let mut waiters = Vec::new();
        waiters
            .try_reserve_exact(children.values.len())
            .map_err(|_| DomHostedError::AllocationFailed)?;
        let actual = waiters
            .capacity()
            .checked_mul(size_of::<Waiter>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            captured
                .checked_add(actual)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        retained.append(
            source.retain(
                actual
                    .checked_sub(requested)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                cancel,
            )?,
        )?;
        for _ in &children.values {
            work.step(cancel)?;
            waiters.push(Waiter {
                invocation: None,
                settled: false,
            });
        }
        let DomNodeKind::Document(metadata) =
            &self.state.node(self.owner, creation.document.node())?.kind
        else {
            return Err(DomError::WrongDocument.into());
        };
        let origin = metadata.origin().clone();
        let bank = self.hosted_operations.prepare(
            source.next_operation_slot(work, cancel)?,
            work,
            source,
            captured
                .checked_add(actual)
                .and_then(|bytes| bytes.checked_sub(size_of::<operations::PreparedRows>()))
                .ok_or(DomHostedError::RepresentationExhausted)?,
            None,
            cancel,
        )?;
        Ok(PreparedInvocation {
            children,
            waiters,
            origin,
            bank,
            retained,
        })
    }
    fn install_hosted_cleanup(
        &mut self,
        source: &mut DomHostedSource,
        invocation: DomHostedOperationId,
        row: &Invocation,
        origin: &crate::DomOpaqueOrigin,
        cancel: &AtomicBool,
    ) -> Result<DomHostedOperationId, DomHostedError> {
        let work = &row.request.request.work;
        let creation = row.request.creation;
        let retained = source.retain(size_of::<Operation>(), cancel)?;
        let bank = self.hosted_operations.prepare(
            source.next_operation_slot(work, cancel)?,
            work,
            source,
            0,
            None,
            cancel,
        )?;
        self.revision
            .0
            .checked_add(self.hosted_revisions)
            .and_then(|value| value.checked_add(1))
            .ok_or(DomError::RevisionExhausted)?;
        crate::hosted::admit(
            bank.operation_scratch_bytes()?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let cleanup = source.begin_operation(work, 4, bank.scratch_bytes()?, cancel)?;
        let request = Request {
            key: RequestKey {
                operation: cleanup,
                phase: 0,
            },
            host: creation.host,
            context: creation.context,
            work: work.clone(),
            parent: None,
        };
        self.hosted_operations.insert_prepared(
            bank,
            Operation {
                origin: origin.clone(),
                policy: None,
                phase: CreationPhase::DestroyEnqueue(DomHostedDestructionRequest {
                    request,
                    creation,
                    children: row.request.children.clone(),
                    invocation,
                    traversable: row.request.traversable,
                }),
                _retained: retained,
            },
            false,
        );
        self.hosted_revisions += 1;
        Ok(cleanup)
    }
    fn complete_hosted_invocation(
        &mut self,
        source: &mut DomHostedSource,
        id: DomHostedOperationId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedDestructionInvocation, DomHostedError> {
        self.with_hosted_operation(id, |dom, operation| {
            let CreationPhase::Invocation(row) = &mut operation.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            if row.request.cleanup.is_none() {
                let creation = row.request.creation;
                // The captured tree cannot change during this fixed synchronous
                // prefix. Publish the retiring guard before any external task
                // or child-constructor service can run.
                if dom.terminal_hosted_proof(source, creation)?.is_none() {
                    let retiring = dom
                        .state
                        .hosted_history
                        .as_deref()
                        .ok_or(DomHostedError::UnsupportedProfile)?
                        .navigable_with(creation.navigable, || {
                            row.request.request.work.step(cancel)
                        })?
                        .retiring;
                    if !retiring {
                        if !row.retiring_reserved {
                            // The issuer ensures total remaining credit: this
                            // mark publication and the final wrapper close each
                            // still consume one original Source permit.
                            dom.reserve_hosted_operation(
                                source,
                                &row.request.request,
                                2,
                                1,
                                cancel,
                            )?;
                            row.retiring_reserved = true;
                        }
                        let mut draft =
                            dom.hosted_request_draft(&row.request.request, source, cancel)?;
                        let fresh = source::formation::Formation::new(&draft, cancel)?;
                        let context =
                            history::Context::new(source, &row.request.request.work, cancel, 0);
                        let context = if let Some(fresh) = &fresh {
                            context.with_formation(fresh)
                        } else {
                            context
                        };
                        let graph = draft
                            .hosted_history
                            .as_deref()
                            .ok_or(DomHostedError::UnsupportedProfile)?
                            .mark_retiring(creation.navigable, &context)?;
                        draft.hosted_history = Some(graph);
                        drop(fresh);
                        let prepared = draft.finish(dom, cancel)?;
                        let next = row
                            .request
                            .request
                            .key
                            .phase
                            .checked_add(1)
                            .ok_or(DomHostedError::RepresentationExhausted)?;
                        let permit = source.permit(
                            &row.request.request,
                            source.revision()?,
                            false,
                            cancel,
                        )?;
                        dom.publish_hosted(prepared, cancel)?;
                        permit.commit();
                        row.retiring_reserved = false;
                        row.request.request.key.phase = next;
                    }
                }
                if let Some(proof) = dom.terminal_hosted_proof(source, creation)? {
                    row.request.cleanup = Some(proof.operation);
                } else {
                    let work = &row.request.request.work;
                    let found = dom.hosted_operations.find(work, cancel, |candidate| {
                        match &candidate.phase {
                            CreationPhase::DestroyEnqueue(request)
                            | CreationPhase::DestroyCleanup(request) => {
                                request.creation.document == creation.document
                                    && request.creation.global == creation.global
                            }
                            CreationPhase::Destroyed(_, actual) => {
                                actual.document == creation.document
                                    && actual.global == creation.global
                            }
                            _ => false,
                        }
                    })?;
                    let cleanup = if let Some(found) = found {
                        found
                    } else {
                        dom.install_hosted_cleanup(source, id, row, &operation.origin, cancel)?
                    };
                    row.request.cleanup = Some(cleanup);
                }
            }
            Ok(OperationResult::Retain(DomHostedDestructionInvocation {
                operation: id,
                navigable: row.request.creation.navigable,
            }))
        })
    }
    pub(super) fn require_hosted_invocation_children(
        &self,
        id: DomHostedOperationId,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let CreationPhase::Invocation(row) = &self.hosted_operation(id)?.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        for waiter in &row.waiters {
            work.step(cancel)?;
            if !waiter.settled {
                return Err(DomHostedError::PendingDocumentObligations);
            }
        }
        Ok(())
    }
}
impl DomHostedDestructionInvocation {
    pub fn request<'a>(
        &self,
        dom: &'a Dom,
    ) -> Result<&'a DomHostedDestructionInvocationRequest, DomHostedError> {
        let CreationPhase::Invocation(row) = &dom.hosted_operation(self.operation)?.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        if row.request.cleanup.is_none() {
            return Err(DomHostedError::IncompleteOperation);
        }
        Ok(&row.request)
    }
    pub fn begin_child(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        index: usize,
        cancel: &AtomicBool,
    ) -> Result<Self, DomHostedError> {
        dom.validate_hosted_source(source)?;
        dom.with_hosted_operation(self.operation, |dom, operation| {
            let CreationPhase::Invocation(row) = &mut operation.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            row.request.request.work.step(cancel)?;
            let creation = *row
                .request
                .children
                .values
                .get(index)
                .ok_or(DomHostedError::WrongPhase)?;
            let waiter = row
                .waiters
                .get_mut(index)
                .ok_or(DomHostedError::WrongPhase)?;
            if waiter.settled {
                return Err(DomHostedError::WrongPhase);
            }
            let child = if let Some(id) = waiter.invocation {
                dom.complete_hosted_invocation(source, id, cancel)?
            } else {
                let child = dom.begin_hosted_invocation(
                    source,
                    creation,
                    row.request.traversable,
                    row.request.request.work.clone(),
                    None,
                    cancel,
                )?;
                waiter.invocation = Some(child.operation);
                child
            };
            Ok(OperationResult::Retain(child))
        })
    }
    pub fn cleanup(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<DomHostedDestructionLease, DomHostedError> {
        dom.validate_hosted_source(source)?;
        let request = self.request(dom)?;
        request.request.work.step(cancel)?;
        let proof = dom.terminal_hosted_proof(source, request.creation)?;
        Ok(DomHostedDestructionLease {
            operation: request.cleanup_operation(),
            completed: proof.as_ref().map(|_| request.creation),
            proof,
        })
    }
    pub fn settle_child(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        index: usize,
        proof: &DomHostedDocumentDestructionProof,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        dom.validate_hosted_source(source)?;
        let operation = dom.hosted_operations.get_mut(self.operation)?;
        let CreationPhase::Invocation(row) = &mut operation.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        row.request.request.work.step(cancel)?;
        let child = row
            .request
            .children
            .values
            .get(index)
            .ok_or(DomHostedError::WrongPhase)?;
        proof.validate(source, child.document, child.global)?;
        let waiter = row
            .waiters
            .get_mut(index)
            .ok_or(DomHostedError::WrongPhase)?;
        if waiter.invocation.is_none() || waiter.settled {
            return Err(DomHostedError::WrongPhase);
        }
        waiter.settled = true;
        Ok(())
    }
    pub fn finish(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        proof: &DomHostedDocumentDestructionProof,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        dom.validate_hosted_source(source)?;
        dom.with_hosted_operation(self.operation, |_dom, operation| {
            let CreationPhase::Invocation(row) = &operation.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            row.request.request.work.step(cancel)?;
            proof.validate(
                source,
                row.request.creation.document,
                row.request.creation.global,
            )?;
            for waiter in &row.waiters {
                row.request.request.work.step(cancel)?;
                if !waiter.settled {
                    return Err(DomHostedError::PendingDocumentObligations);
                }
            }
            if matches!(row.history, HistoryDependency::Pending(_)) {
                return Err(DomHostedError::PendingDocumentObligations);
            }
            let remaining = if row.retiring_reserved {
                Some(
                    _dom.hosted_revisions
                        .checked_sub(1)
                        .ok_or(DomHostedError::WrongPhase)?,
                )
            } else {
                None
            };
            let permit = source.permit(&row.request.request, source.revision()?, true, cancel)?;
            permit.commit();
            if let Some(remaining) = remaining {
                _dom.hosted_revisions = remaining;
            }
            Ok(OperationResult::Complete(()))
        })
    }
}

impl Dom {
    pub(super) fn prepare_hosted_history_completion(
        &self,
        history: DomHostedOperationId,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<DomHostedOperationId, DomHostedError> {
        self.hosted_operations.find(work,cancel,|operation|matches!(&operation.phase,CreationPhase::Invocation(row) if row.history==HistoryDependency::Pending(history)))?
            .ok_or(DomHostedError::WrongPhase)
    }
    pub(super) fn settle_hosted_history_completion(
        &mut self,
        invocation: DomHostedOperationId,
        history: DomHostedOperationId,
    ) {
        let operation = self
            .hosted_operations
            .get_mut(invocation)
            .expect("prevalidated invoking row retained through fixed H commit");
        let CreationPhase::Invocation(row) = &mut operation.phase else {
            unreachable!("prevalidated original invoking algorithm")
        };
        assert!(
            row.history == HistoryDependency::Pending(history),
            "same exact original H occurrence"
        );
        row.history = HistoryDependency::Settled(history);
    }
}

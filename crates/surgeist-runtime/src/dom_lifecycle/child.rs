//! Original external objects and completed inputs of a Source-issued child.
//! The canonical Source phase and graph remain the sole algorithm cursor/data.
use super::*;
use surgeist_dom::{
    DomHostedChildPreparation, DomHostedChildRequest, DomHostedPreparedChild, DomSourceOperationId,
};

pub(super) struct Creation {
    pub operation: DomSourceOperationId,
    pub occurrence: u64,
    pub preparation: Option<DomHostedChildPreparation>,
    pub prepared: Option<DomHostedPreparedChild>,
    pub context: Option<records::BrowsingContext>,
    pub sample: Option<clock::Sample>,
    pub global: Option<DomHostedGlobalId>,
    pub policy: Option<DomHostedPolicyPublication>,
    pub publication: Option<DomHostedCreationPublication>,
    pub entry: Option<DomHostedEntryId>,
    pub navigable: Option<DomHostedNavigableId>,
    pub uuids: Option<uuid::HistoryUuids>,
    pub entry_initialization: Option<DomHostedEntryInitialization>,
    pub finished_sample: Option<clock::CoarsenedSample>,
}
impl Creation {
    fn new(request: &DomHostedChildRequest) -> Self {
        Self {
            operation: request.operation(),
            occurrence: request.occurrence(),
            preparation: None,
            prepared: None,
            context: None,
            sample: None,
            global: None,
            policy: None,
            publication: None,
            entry: None,
            navigable: None,
            uuids: None,
            entry_initialization: None,
            finished_sample: None,
        }
    }
}

impl HostedState {
    pub(super) fn child_creation(
        &mut self,
        request: &DomHostedChildRequest,
        lease: &surgeist_dom::DomSourceLease<'_>,
        cancel: &AtomicBool,
    ) -> Result<usize, DomHostedRuntimeError> {
        for index in 0..self.children.len() {
            self.step(cancel)?;
            if self.children[index].operation == request.operation()
                && self.children[index].occurrence == request.occurrence()
            {
                return Ok(index);
            }
        }
        if self.children.len() == self.children.capacity() {
            let required = self
                .children
                .len()
                .checked_add(1)
                .ok_or(DomHostedRuntimeError::Admission)?;
            let old = self.source_retained_bytes(lease, cancel)?;
            let requested = required
                .checked_mul(size_of::<Creation>())
                .ok_or(DomHostedRuntimeError::Admission)?;
            if requested > self.limits.scratch_bytes
                || old
                    .checked_add(requested)
                    .is_none_or(|bytes| bytes > self.limits.storage_bytes)
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            for _ in 0..self.children.len() {
                self.step(cancel)?;
            }
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(required)
                .map_err(|_| DomHostedRuntimeError::Admission)?;
            let actual = replacement
                .capacity()
                .checked_mul(size_of::<Creation>())
                .ok_or(DomHostedRuntimeError::Admission)?;
            if actual > self.limits.scratch_bytes
                || old
                    .checked_add(actual)
                    .is_none_or(|bytes| bytes > self.limits.storage_bytes)
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            replacement.append(&mut self.children);
            self.children = replacement;
        }
        self.step(cancel)?;
        let index = self.children.len();
        self.children.push(Creation::new(request));
        Ok(index)
    }
}

impl HostedRuntime {
    pub(super) fn prepare_source_child(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        request: &DomHostedChildRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        if state.dom != lease.snapshot().owner() {
            return Err(DomHostedRuntimeError::Binding);
        }
        let (h, t, c, g) = state.find_global(request.parent_global(), cancel, || Ok(()))?;
        let group = state.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        let parent = &group.clusters[c].agent.globals[g];
        if state.hosts[h].id != request.host()
            || parent.settings.document != Some(request.parent_document().document())
            || !parent.settings.ready
        {
            return Err(DomHostedRuntimeError::Binding);
        }
        let group_root = group.context;
        let index = state.child_creation(request, lease, cancel)?;
        let HostedState {
            source,
            clock,
            work,
            limits,
            children,
            ..
        } = &mut *state;
        let child = &mut children[index];
        if child.preparation.is_none() && child.prepared.is_none() {
            let preparation = lease
                .prepare_hosted_child(source, request, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
            child.context = Some(records::BrowsingContext {
                id: preparation.browsing_context(),
                group_root,
                creator: Some(request.parent_global()),
                global: None,
            });
            child.preparation = Some(preparation);
        }
        if child.sample.is_none() {
            spend(work, limits.work, cancel)?;
            child.sample = Some(clock.sample().map_err(clock_error)?);
        }
        if child.prepared.is_none() {
            child.prepared = Some(
                source
                    .seal_child_preparation(request, &mut child.preparation, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        lease
            .accept_hosted_child_preparation(source, request, &mut child.prepared, cancel)
            .map_err(DomHostedRuntimeError::Dom)
    }
}

impl HostedState {
    fn child_index(
        &mut self,
        request: &DomHostedChildRequest,
        cancel: &AtomicBool,
        mut admit: impl FnMut() -> Result<(), DomHostedRuntimeError>,
    ) -> Result<usize, DomHostedRuntimeError> {
        for index in 0..self.children.len() {
            self.step(cancel)?;
            admit()?;
            if self.children[index].operation == request.operation()
                && self.children[index].occurrence == request.occurrence()
            {
                return Ok(index);
            }
        }
        Err(DomHostedRuntimeError::Protocol)
    }
}

impl HostedRuntime {
    pub(super) fn execute_child(
        &self,
        child_request: &DomHostedChildRequest,
        request: &surgeist_dom::DomHostedExecutionRequest,
        lease: &surgeist_dom::DomSourceLease<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let parent = request
            .parent_global()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        if parent != child_request.parent_global() {
            return Err(DomHostedRuntimeError::Protocol);
        }
        let (h, t, _, _) = state.find_global(parent, cancel, || Ok(()))?;
        let index = state.child_index(child_request, cancel, || Ok(()))?;
        if state.children[index].publication.is_some() {
            return Ok(());
        }
        let retained = state.source_retained_bytes(lease, cancel)?;
        let HostedState {
            hosts,
            children,
            work,
            limits,
            source,
            ..
        } = &mut *state;
        let child = &mut children[index];
        let group = hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        let cluster = agent::find_cluster(
            group,
            request.origin(),
            work,
            limits.work,
            cancel,
            || Ok(()),
        )?
        .ok_or(DomHostedRuntimeError::Protocol)?;
        let mut added = 0usize;
        let sample = child
            .sample
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .coarsen()
            .map_err(clock_error)?;
        if sample.source != source.id() {
            return Err(DomHostedRuntimeError::Binding);
        }
        if child.global.is_none() {
            child.global = Some(
                source
                    .issue_global(request, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        let global = child.global.expect("retained actual child Global identity");
        let native = &mut group.clusters[cluster].agent;
        let mut installed = false;
        for value in &native.globals {
            spend(work, limits.work, cancel)?;
            if value.id == global {
                installed = true;
                break;
            }
        }
        if !installed {
            added = added
                .checked_add(agent::reserve_global(
                    native,
                    retained
                        .checked_add(added)
                        .ok_or(DomHostedRuntimeError::Admission)?,
                    *limits,
                    work,
                    cancel,
                )?)
                .ok_or(DomHostedRuntimeError::Admission)?;
            spend(work, limits.work, cancel)?;
            spend(work, limits.work, cancel)?;
            let payload = settings_url_payload_bytes(
                request.creation_url(),
                request.top_level_creation_url(),
            )?;
            if retained
                .checked_add(added)
                .and_then(|bytes| bytes.checked_add(payload))
                .is_none_or(|bytes| bytes > limits.storage_bytes)
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            let context = child
                .context
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            if context.id != request.browsing_context()
                || context.group_root != group.context
                || context.creator != Some(parent)
            {
                return Err(DomHostedRuntimeError::Protocol);
            }
            // The new Window row is an actual additional Runtime mutation,
            // admitted on the same original Source before either bank changes.
            spend(work, limits.work, cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            let mut context = child.context.take().expect("validated child BC object");
            context.global = Some(global);
            native
                .windows
                .rows
                .push(super::watcher::NativeWindow::fresh(global));
            native.globals.push(records::Global {
                context,
                id: global,
                settings: records::Settings {
                    navigation_start: sample,
                    document: None,
                    ready: false,
                    creation_url: request.creation_url().clone(),
                    top_level_creation_url: request.top_level_creation_url().clone(),
                    top_level_origin: request.top_level_origin().clone(),
                },
            });
        }
        if child.publication.is_none() {
            child.publication = Some(
                source
                    .seal_native_execution(request, global, sample.microseconds, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        Ok(())
    }

    pub(super) fn activate_child(
        &self,
        child_request: &DomHostedChildRequest,
        request: &surgeist_dom::DomHostedMakeActiveRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let index = state.child_index(child_request, cancel, || Ok(()))?;
        let (h, t, c, g) = state.find_global(request.global(), cancel, || Ok(()))?;
        state.step(cancel)?;
        let HostedState {
            hosts,
            children,
            source,
            ..
        } = &mut *state;
        let native = &mut hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters[c]
            .agent
            .globals[g];
        native.settings.document = Some(request.document());
        native.settings.ready = true;
        if children[index].publication.is_none() {
            children[index].publication = Some(
                source
                    .seal_make_active(request, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        Ok(())
    }

    pub(super) fn finish_loading_child(
        &self,
        child_request: &DomHostedChildRequest,
        request: &surgeist_dom::DomHostedFinishLoadingRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let index = state.child_index(child_request, cancel, || Ok(()))?;
        let (h, t, c, g) = state.find_global(request.global(), cancel, || Ok(()))?;
        state.step(cancel)?;
        let HostedState {
            hosts,
            children,
            source,
            clock,
            ..
        } = &mut *state;
        let native = &hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters[c]
            .agent
            .globals[g];
        let original = native.settings.navigation_start;
        if native.settings.document != Some(request.document())
            || original.source != request.navigation_start().source()
            || original.microseconds != request.navigation_start().microseconds()
        {
            return Err(DomHostedRuntimeError::Protocol);
        }
        let child = &mut children[index];
        if child.finished_sample.is_none() {
            child.finished_sample = Some(
                clock
                    .sample()
                    .map_err(clock_error)?
                    .coarsen()
                    .map_err(clock_error)?,
            );
        }
        let sample = child
            .finished_sample
            .expect("actual retained completion sample");
        if sample.source != source.id() {
            return Err(DomHostedRuntimeError::Binding);
        }
        if child.publication.is_none() {
            child.publication = Some(
                source
                    .seal_finish_loading(request, sample.microseconds, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        Ok(())
    }

    pub(super) fn initialize_child_entry(
        &self,
        child_request: &DomHostedChildRequest,
        request: &surgeist_dom::DomHostedEntryRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let index = state.child_index(child_request, cancel, || Ok(()))?;
        let HostedState {
            children,
            source,
            work,
            limits,
            ..
        } = &mut *state;
        let child = &mut children[index];
        if child.publication.is_none() {
            if child.uuids.is_none() && child.entry_initialization.is_none() {
                spend(work, limits.work, cancel)?;
                uuid::HistoryUuids::sample(&mut child.uuids, cancel)?;
            }
            if child.entry.is_none() {
                child.entry = Some(
                    source
                        .issue_entry(request, cancel)
                        .map_err(DomHostedRuntimeError::Dom)?,
                );
            }
            if child.navigable.is_none() {
                child.navigable = Some(
                    source
                        .issue_navigable(request, cancel)
                        .map_err(DomHostedRuntimeError::Dom)?,
                );
            }
            if child.entry_initialization.is_none() {
                let values = child.uuids.take().ok_or(DomHostedRuntimeError::Protocol)?;
                child.entry_initialization =
                    Some(DomHostedEntryInitialization::new(values.key, values.id));
            }
            child.publication = Some(
                source
                    .seal_initial_entry(
                        request,
                        child.entry.expect("actual issued child entry"),
                        child.navigable.expect("actual issued child navigable"),
                        &mut child.entry_initialization,
                        cancel,
                    )
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        Ok(())
    }

    pub(super) fn accept_child_publication(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        request: &DomHostedChildRequest,
        policy: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let index = state.child_index(request, cancel, || Ok(()))?;
        let HostedState {
            source, children, ..
        } = &mut *state;
        if policy {
            #[cfg(all(
                test,
                any(
                    target_os = "macos",
                    target_os = "linux",
                    target_os = "windows",
                    target_os = "android",
                    target_os = "ios"
                )
            ))]
            url_configuration_tests::cancel_before_policy_accept(cancel);
            lease
                .accept_hosted_child_policy(source, request, &mut children[index].policy, cancel)
                .map_err(DomHostedRuntimeError::Dom)
        } else {
            lease
                .accept_hosted_child_publication(
                    source,
                    request,
                    &mut children[index].publication,
                    cancel,
                )
                .map_err(DomHostedRuntimeError::Dom)
        }
    }
}

pub(crate) enum SourceProgress {
    Serviced,
    Url(surgeist_dom::DomSourceUrlStage),
}

impl HostedRuntime {
    fn policy_decision(
        &self,
        request: &surgeist_dom::DomHostedPolicyRequest,
        declared_matches: Option<bool>,
        cancel: &AtomicBool,
    ) -> Result<policy::Decision, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let HostedState { work, limits, .. } = &mut *state;
        policy::inherited(request, declared_matches, work, limits.work, cancel)
    }

    fn seal_child_policy(
        &self,
        child_request: &DomHostedChildRequest,
        request: &surgeist_dom::DomHostedPolicyRequest,
        inherited: surgeist_dom::DomFullscreenInherited,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let index = state.child_index(child_request, cancel, || Ok(()))?;
        let HostedState {
            source, children, ..
        } = &mut *state;
        if children[index].policy.is_none() {
            children[index].policy = Some(
                source
                    .seal_permissions_policy(request, inherited, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        Ok(())
    }

    fn sandbox_declared_matches(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        child: &DomHostedChildRequest,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.step(cancel)?;
        let origin = lease
            .declared_sandbox_origin(child, &mut state.source, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        let surgeist_dom::DomHostedChildStep::PermissionsPolicy(policy) = lease
            .hosted_child_step(child)
            .map_err(DomHostedRuntimeError::Dom)?
        else {
            return Err(DomHostedRuntimeError::Protocol);
        };
        Ok(&origin == policy.target_origin())
    }

    fn qualify_child_url_configuration(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        child: &DomHostedChildRequest,
        expected: Option<&surgeist_html_semantics::HtmlBaseFactsRevision>,
        cancel: &AtomicBool,
    ) -> Result<Option<surgeist_dom::DomSourceUrlStage>, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        // Resolve the existing Runtime custody before canonical retirement.
        // Nothing fallible may separate that retirement from clearing its
        // derived, still unpublished policy in this same original row.
        let index = state.child_index(child, cancel, || Ok(()))?;
        let stage = lease
            .qualify_source_url_configuration(child, expected, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        if stage.is_some() && expected.is_some() {
            state.children[index].policy = None;
        }
        Ok(stage)
    }

    pub(crate) fn service_source_child(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        child: &DomHostedChildRequest,
        url_revision: Option<&surgeist_html_semantics::HtmlBaseFactsRevision>,
        cancel: &AtomicBool,
    ) -> Result<SourceProgress, DomHostedRuntimeError> {
        use surgeist_dom::{DomHostedChildStep as Step, DomSourceUrlStage as UrlStage};
        let mut declared_matches = None;
        loop {
            // Qualify before either a retained response or its sealed policy
            // can advance this same unpublished stage. Missing service keeps
            // both forms of custody intact and reports the actual pending URL.
            if matches!(
                lease
                    .hosted_child_step(child)
                    .map_err(DomHostedRuntimeError::Dom)?,
                Step::PermissionsPolicy(_) | Step::NeedsUrl(_)
            ) && let Some(stage) =
                self.qualify_child_url_configuration(lease, child, url_revision, cancel)?
            {
                return Ok(SourceProgress::Url(stage));
            }
            lease
                .process_hosted_child_attributes(child, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
            if matches!(
                lease
                    .hosted_child_step(child)
                    .map_err(DomHostedRuntimeError::Dom)?,
                Step::NativeExecution(_)
            ) {
                self.prepare_child_window(lease, child, cancel)?;
            }
            let mut policy_phase = false;
            match lease
                .hosted_child_step(child)
                .map_err(DomHostedRuntimeError::Dom)?
            {
                Step::Prepare(_) => {
                    self.prepare_source_child(lease, child, cancel)?;
                    continue;
                }
                Step::PrepareRemoval(_) => {
                    let owner = self.state()?;
                    let mut state = owner
                        .try_borrow_mut()
                        .map_err(|_| DomHostedRuntimeError::Busy)?;
                    state.step(cancel)?;
                    lease
                        .prepare_hosted_child_removal(&mut state.source, child, cancel)
                        .map_err(DomHostedRuntimeError::Dom)?;
                    continue;
                }
                Step::PermissionsPolicy(request) => {
                    let sealed = {
                        let owner = self.state()?;
                        let mut state = owner
                            .try_borrow_mut()
                            .map_err(|_| DomHostedRuntimeError::Busy)?;
                        let index = state.child_index(child, cancel, || Ok(()))?;
                        state.children[index].policy.is_some()
                    };
                    if sealed {
                        self.accept_child_publication(lease, child, true, cancel)?;
                        declared_matches = None;
                        continue;
                    }
                    match self.policy_decision(request, declared_matches, cancel)? {
                        policy::Decision::Complete(value) => {
                            self.seal_child_policy(child, request, value, cancel)?;
                            policy_phase = true;
                        }
                        policy::Decision::SandboxOrigin => {
                            declared_matches =
                                Some(self.sandbox_declared_matches(lease, child, cancel)?);
                            continue;
                        }
                        policy::Decision::Url => {
                            let Some(observation) = lease
                                .source_url_observation(child, UrlStage::IframeDeclaredOrigin)
                                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?
                            else {
                                return Ok(SourceProgress::Url(UrlStage::IframeDeclaredOrigin));
                            };
                            // Canonical acceptance has checked the actual parse inputs,
                            // independent configuration and stage-specific relation.
                            declared_matches = Some(match observation.observation().outcome() {
                                surgeist_html_semantics::HtmlBaseParseOutcome::Failure => {
                                    request
                                        .embedder_policy()
                                        .ok_or(DomHostedRuntimeError::Protocol)?
                                        .parent_origin()
                                        == request.target_origin()
                                }
                                surgeist_html_semantics::HtmlBaseParseOutcome::Parsed {
                                    ..
                                } => match observation
                                    .origin()
                                    .ok_or(DomHostedRuntimeError::Protocol)?
                                {
                                    surgeist_dom::DomSourceUrlOrigin::Opaque(origin) => {
                                        origin == request.target_origin()
                                    }
                                    surgeist_dom::DomSourceUrlOrigin::UnequalToTarget => false,
                                },
                            });
                            continue;
                        }
                    }
                }
                Step::NativeExecution(request) => {
                    self.execute_child(child, request, lease, cancel)?
                }
                Step::MakeActive(request) => self.activate_child(child, request, cancel)?,
                Step::FinishLoading(request) => {
                    self.finish_loading_child(child, request, cancel)?
                }
                Step::InitialEntry(request) => {
                    self.initialize_child_entry(child, request, cancel)?
                }
                Step::QueueHistory(request) => {
                    self.queue_child_history(lease, child, request.operation(), cancel)?;
                    continue;
                }
                Step::NeedsUrl(stage) => return Ok(SourceProgress::Url(stage)),
                Step::Returned => {
                    self.release_child_inputs(child, cancel)?;
                    return Ok(SourceProgress::Serviced);
                }
                Step::Removal(request) => {
                    self.queue_child_removal(lease, child, request.operation(), cancel)?;
                    return Ok(SourceProgress::Serviced);
                }
            }
            self.accept_child_publication(lease, child, policy_phase, cancel)?;
            declared_matches = None;
        }
    }
}

impl HostedRuntime {
    fn queue_child_history(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        child: &DomHostedChildRequest,
        operation: surgeist_dom::DomHostedOperationId,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let (h, t, parent_cluster, _) =
            state.find_global(child.parent_global(), cancel, || Ok(()))?;
        let mut queued_index = None;
        {
            let HostedState {
                hosts,
                work,
                limits,
                ..
            } = &mut *state;
            let queue = &hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .traversal;
            if queue.traversable != Some(child.traversable()) {
                return Err(DomHostedRuntimeError::Binding);
            }
            for (index, row) in queue.rows.iter().enumerate() {
                spend(work, limits.work, cancel)?;
                if row.operation == operation {
                    queued_index = Some(index);
                    break;
                }
            }
        }
        let index = if let Some(index) = queued_index {
            index
        } else {
            let count = {
                let HostedState {
                    hosts,
                    work,
                    limits,
                    ..
                } = &mut *state;
                native_task_count(hosts, work, limits.work, cancel)?
            };
            if count >= state.limits.tasks {
                return Err(DomHostedRuntimeError::Admission);
            }
            let retained = state.source_retained_bytes(lease, cancel)?;
            let HostedState {
                hosts,
                limits,
                work,
                ..
            } = &mut *state;
            let initiating_loop = hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .clusters[parent_cluster]
                .agent
                .event_loop
                .clone();
            let queue = &mut hosts[h].top_levels[t]
                .group
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .traversal;
            history::reserve_traversal(queue, retained, *limits, work, cancel)?;
            spend(work, limits.work, cancel)?;
            let index = queue.rows.len();
            queue.rows.push(history::Queued {
                operation,
                initiating_loop,
                continuation: None,
                publication: None,
                removal: None,
            });
            index
        };
        let HostedState { hosts, source, .. } = &mut *state;
        let row = &mut hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .traversal
            .rows[index];
        if row.continuation.is_some() {
            return Err(DomHostedRuntimeError::Protocol);
        }
        if row.publication.is_none() {
            let surgeist_dom::DomHostedChildStep::QueueHistory(request) = lease
                .hosted_child_step(child)
                .map_err(DomHostedRuntimeError::Dom)?
            else {
                return Err(DomHostedRuntimeError::Protocol);
            };
            row.publication = Some(
                source
                    .seal_child_history(request, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        let continuation = lease
            .accept_hosted_child_history(source, child, &mut row.publication, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        // Every capacity/read/move needed by this ownership transfer preceded
        // canonical acceptance; no fallible action separates handoff and custody.
        row.continuation = Some(continuation);
        Ok(())
    }

    fn release_child_inputs(
        &self,
        child: &DomHostedChildRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let index = state.child_index(child, cancel, || Ok(()))?;
        for _ in index..state.children.len() {
            state.step(cancel)?;
        }
        state.children.remove(index);
        Ok(())
    }
}

impl HostedRuntime {
    pub(crate) fn admit_url_observation(
        &self,
        request: &surgeist_dom::DomSourceUrlRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        if request.document().store().owner() != state.dom {
            return Err(DomHostedRuntimeError::Binding);
        }
        // Invocation is an actual orchestration step. Do not call an external
        // observer with no remaining native work, even on a parked phase.
        state.step(cancel)
    }
}

impl HostedRuntime {
    fn queue_child_removal(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        child: &DomHostedChildRequest,
        operation: surgeist_dom::DomHostedOperationId,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let (h, t, c, _) = state.find_global(child.parent_global(), cancel, || Ok(()))?;
        let mut found = None;
        {
            let HostedState {
                hosts,
                work,
                limits,
                ..
            } = &mut *state;
            for (index, row) in hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .traversal
                .rows
                .iter()
                .enumerate()
            {
                spend(work, limits.work, cancel)?;
                if row.operation == operation {
                    found = Some(index);
                    break;
                }
            }
        }
        let index = if let Some(index) = found {
            index
        } else {
            let count = {
                let HostedState {
                    hosts,
                    work,
                    limits,
                    ..
                } = &mut *state;
                native_task_count(hosts, work, limits.work, cancel)?
            };
            if count >= state.limits.tasks {
                return Err(DomHostedRuntimeError::Admission);
            }
            let retained = state.source_retained_bytes(lease, cancel)?;
            let HostedState {
                hosts,
                work,
                limits,
                ..
            } = &mut *state;
            let group = hosts[h].top_levels[t]
                .group
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            if group.traversal.traversable != Some(child.traversable()) {
                return Err(DomHostedRuntimeError::Binding);
            }
            history::reserve_traversal(&mut group.traversal, retained, *limits, work, cancel)?;
            spend(work, limits.work, cancel)?;
            let index = group.traversal.rows.len();
            group.traversal.rows.push(history::Queued {
                operation,
                initiating_loop: group.clusters[c].agent.event_loop.clone(),
                continuation: None,
                publication: None,
                removal: None,
            });
            index
        };
        state.reserve_invocation(cancel)?;
        state.step(cancel)?;
        let HostedState {
            hosts,
            source,
            destruction,
            ..
        } = &mut *state;
        let row = &mut hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .traversal
            .rows[index];
        if row.continuation.is_some() {
            return Err(DomHostedRuntimeError::Protocol);
        }
        if row.removal.is_none() {
            let surgeist_dom::DomHostedChildStep::Removal(request) = lease
                .hosted_child_step(child)
                .map_err(DomHostedRuntimeError::Dom)?
            else {
                return Err(DomHostedRuntimeError::Protocol);
            };
            row.removal = Some(
                source
                    .seal_child_removal(request, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        let handoff = lease
            .accept_hosted_child_removal(source, child, &mut row.removal, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        let (invocation, history) = handoff.into_parts();
        destruction.retain(invocation, Some(history.operation()));
        row.continuation = Some(history);
        Ok(())
    }
}

#[cfg(all(
    test,
    any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    )
))]
mod url_configuration_tests {
    use super::*;
    use crate::{
        DomManipulationRuntimeLimits, DomSession, DomSessionLimits, DomSourceDriveProgress,
        DomSourceServices, DomSourceUnavailable, DomSourceUrlService, DomStyleRuntimeLimits,
    };
    use surgeist_dom::{
        DomBatch, DomError, DomHostedChildStep, DomHostedRecordLimits, DomLimits, DomSourceEdit,
        DomSourceLimits, DomSourceRequest, DomSourceUrlStage,
    };
    use surgeist_html::Namespace;
    use surgeist_html_semantics::{
        HtmlBaseFactsRevision, HtmlBaseParseObservation, HtmlBaseParseOutcome, HtmlBaseProducerId,
    };

    thread_local! {
        static CANCEL_POLICY_ACCEPT:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};
    }
    pub(super) fn cancel_before_policy_accept(cancel: &AtomicBool) {
        if CANCEL_POLICY_ACCEPT.with(|flag| flag.replace(false)) {
            cancel.store(true, Ordering::Release);
        }
    }
    struct CancelPolicyAccept;
    impl CancelPolicyAccept {
        fn arm() -> Self {
            CANCEL_POLICY_ACCEPT.with(|flag| flag.set(true));
            Self
        }
    }
    impl Drop for CancelPolicyAccept {
        fn drop(&mut self) {
            CANCEL_POLICY_ACCEPT.with(|flag| flag.set(false));
        }
    }

    fn source_limits(work: u64) -> DomSourceLimits {
        DomSourceLimits::new(32_000_000, 1_000_000, work, 256, 4096, 4096, 1)
    }
    fn fixture() -> DomSession {
        DomSession::new(
            Dom::new(DomLimits::new(256, 1_000_000, 512, 512, 512, 32, 4_000_000).unwrap())
                .unwrap(),
            DomSessionLimits {
                source_events: crate::DomSourceEventRuntimeLimits {
                    jobs: 0,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                },
                style: DomStyleRuntimeLimits {
                    roots: 8,
                    completions: 32,
                    reports: 32,
                    storage_bytes: 32_000_000,
                    work: 4_000_000,
                    css_input: surgeist_css::CssUtf16InputLimits::default(),
                },
                dom_manipulation: DomManipulationRuntimeLimits {
                    tasks: 32,
                    storage_bytes: 32_000_000,
                    work: 4_000_000,
                },
                hosted: DomHostedRuntimeLimits {
                    canonical: DomHostedLimits::new(
                        DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                        32_000_000,
                        32_000_000,
                        4_000_000,
                    ),
                    tasks: 32,
                    storage_bytes: 32_000_000,
                    scratch_bytes: 32_000_000,
                    work: 4_000_000,
                },
            },
        )
        .unwrap()
    }

    // The middle of this fixture uses the real owning lease, not a fabricated
    // cache or phase. The public weak driver cannot expose that custody to an
    // external integration test. Both the initial and final drive are public.
    fn admitted_reply_then_refusal(
        changed_configuration: bool,
        sealed: bool,
        missing_service: bool,
    ) {
        let cancel = AtomicBool::new(false);
        let mut session = fixture();
        let mut hosted = session.hosted_execution();
        let top = {
            let mut dom = session.dom();
            let host = hosted.create_host(&mut dom, &cancel).unwrap();
            hosted.create_top_level(&mut dom, host, &cancel).unwrap()
        };
        let snapshot = session.snapshot();
        let html = snapshot.node(top.document().node()).unwrap().children()[0];
        let body = snapshot.node(html).unwrap().children()[1];
        let iframe = {
            let mut batch = DomBatch::new();
            let closed = batch.create_html_document(top.document(), None);
            let iframe = batch.create_element_ns(&closed, Namespace::html(), "iframe");
            batch.set_attribute(&iframe, "src", "");
            batch.set_attribute(&iframe, "allow", "fullscreen 'src'");
            session
                .dom()
                .apply(batch, &cancel)
                .unwrap()
                .created(&iframe)
                .unwrap()
                .unwrap()
        };
        let mut source = session.source_execution();
        let pending = source
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(iframe, body, None),
                source_limits(20_000_000),
                &mut DomSourceServices {
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        let DomSourceDriveProgress::Pending {
            request: DomSourceRequest::HostedChild(child),
            reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
        } = pending
        else {
            panic!("the actual present-empty src must reach declared-origin URL");
        };
        let operation = child.operation();
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1);
        let next_revision = HtmlBaseFactsRevision::new(revision.producer().clone(), 2, 1, 1);
        let (context, sample) = {
            let owner = hosted.state().unwrap();
            let state = owner.borrow();
            let row = state
                .children
                .iter()
                .find(|row| row.operation == operation)
                .unwrap();
            (row.context.as_ref().unwrap().id, row.sample.unwrap())
        };
        let spent = {
            let mut dom = session.dom();
            let mut lease = dom
                .resume_source_edit(operation, source_limits(20_000_000), &cancel)
                .unwrap();
            let issued = lease
                .source_url_request(
                    &child,
                    DomSourceUrlStage::IframeDeclaredOrigin,
                    &revision,
                    &cancel,
                )
                .unwrap();
            assert!(issued.raw().is_empty());
            assert_eq!(issued.base_url().serialization(), &"about:blank".into());
            // The fixed empty relative URL cannot parse against this actual
            // opaque about:blank base. This is the existing literal URL case.
            let actual = HtmlBaseParseObservation::new(
                issued.raw().clone(),
                issued.base_url().clone(),
                issued.encoding().clone(),
                revision.clone(),
                HtmlBaseParseOutcome::Failure,
            )
            .unwrap();
            let mut response = Some(issued.observe(actual, None).unwrap());
            lease
                .accept_source_url_observation(&child, &issued, &mut response, &cancel)
                .unwrap();
            assert!(response.is_none());
            let accepted = lease
                .source_url_observation(&child, DomSourceUrlStage::IframeDeclaredOrigin)
                .unwrap()
                .unwrap();
            assert_eq!(accepted.request().revision(), &revision);
            assert!(matches!(
                accepted.observation().outcome(),
                HtmlBaseParseOutcome::Failure
            ));
            lease.status().spent_work()
        };
        {
            let mut dom = session.dom();
            if sealed {
                let mut lease = dom
                    .resume_source_edit(operation, source_limits(20_000_000), &cancel)
                    .unwrap();
                let _arm = CancelPolicyAccept::arm();
                assert!(matches!(
                    hosted.service_source_child(&mut lease, &child, Some(&revision), &cancel),
                    Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                        DomError::Cancelled
                    )))
                ));
                assert!(cancel.load(Ordering::Acquire));
                cancel.store(false, Ordering::Release);
            } else {
                assert!(
                    matches!(dom.resume_source_edit(operation,source_limits(spent),&cancel),
                    Err(DomError::SourceWorkLimit {limit}) if limit==spent)
                );
            }
            // The exact owning readback is reopened with a raised absolute
            // limit; no phase, input or cumulative counter is reconstructed.
            let lease = dom
                .resume_source_edit(operation, source_limits(20_000_000), &cancel)
                .unwrap();
            assert_eq!(lease.status().operation(), operation);
            assert!(lease.status().spent_work() >= spent);
            let DomHostedChildStep::PermissionsPolicy(policy) =
                lease.hosted_child_step(&child).unwrap()
            else {
                panic!("refusal must precede canonical policy completion");
            };
            assert_eq!(policy.browsing_context(), context);
            assert!(
                lease
                    .source_url_observation(&child, DomSourceUrlStage::IframeDeclaredOrigin)
                    .unwrap()
                    .is_some()
            );
        }
        {
            let owner = hosted.state().unwrap();
            let state = owner.borrow();
            let row = state
                .children
                .iter()
                .find(|row| row.operation == operation)
                .unwrap();
            assert_eq!(row.policy.is_some(), sealed);
            assert_eq!(row.context.as_ref().unwrap().id, context);
            assert_eq!(row.sample, Some(sample));
        }
        assert_eq!(
            session
                .snapshot()
                .node(iframe)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
        let before_resume = session.snapshot();
        let expected = if changed_configuration {
            &next_revision
        } else {
            &revision
        };
        if missing_service {
            let unavailable = source
                .resume(
                    &mut session.dom(),
                    operation,
                    source_limits(20_000_000),
                    &mut DomSourceServices {
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel,
                )
                .unwrap();
            assert!(matches!(
                unavailable,
                DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::HostedChild(ref current),
                    reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
                } if current.operation() == operation
            ));
            let mut dom = session.dom();
            let lease = dom
                .resume_source_edit(operation, source_limits(20_000_000), &cancel)
                .unwrap();
            assert!(lease.status().spent_work() >= spent);
            assert_eq!(
                lease
                    .source_url_observation(&child, DomSourceUrlStage::IframeDeclaredOrigin)
                    .unwrap()
                    .unwrap()
                    .request()
                    .revision(),
                &revision
            );
            let owner = hosted.state().unwrap();
            let state = owner.borrow();
            let row = state
                .children
                .iter()
                .find(|row| row.operation == operation)
                .unwrap();
            assert_eq!(row.policy.is_some(), sealed);
            assert_eq!(row.context.as_ref().unwrap().id, context);
            assert_eq!(row.sample, Some(sample));
        }
        let mut calls = Vec::new();
        let mut observe = |request: &surgeist_dom::DomSourceUrlRequest, _: &AtomicBool| {
            calls.push((request.stage(), request.revision().clone()));
            Ok(None)
        };
        let result = source
            .resume(
                &mut session.dom(),
                operation,
                source_limits(20_000_000),
                &mut DomSourceServices {
                    style_policy: &mut |_| None,
                    url: Some(DomSourceUrlService {
                        revision: expected,
                        observe: &mut observe,
                    }),
                },
                &cancel,
            )
            .unwrap();
        if changed_configuration {
            assert!(
                matches!(result,DomSourceDriveProgress::Pending {request:DomSourceRequest::HostedChild(ref current),
                reason:DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin)} if current.operation()==operation),
                "unavailable current configuration must not publish cached R1 policy: {result:?}"
            );
            assert_eq!(
                calls,
                [(DomSourceUrlStage::IframeDeclaredOrigin, next_revision)]
            );
            assert_eq!(
                session
                    .snapshot()
                    .node(iframe)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .child_navigable(),
                Some(None)
            );
            assert_eq!(
                session.dom().pending_source_edit().unwrap().operation(),
                operation
            );
            let owner = hosted.state().unwrap();
            let state = owner.borrow();
            let row = state
                .children
                .iter()
                .find(|row| row.operation == operation)
                .unwrap();
            assert_eq!(row.context.as_ref().unwrap().id, context);
            assert_eq!(row.sample, Some(sample));
            assert!(row.policy.is_none());
        } else {
            assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
            assert!(
                calls.is_empty(),
                "same-config accepted response must not be requested again"
            );
            let snapshot = session.snapshot();
            let child = snapshot
                .node(iframe)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable()
                .unwrap()
                .unwrap();
            let nav = snapshot.hosted_navigable(child).unwrap();
            assert_eq!(nav.browsing_context(), context);
            assert_eq!(
                snapshot
                    .document(nav.active_document().unwrap())
                    .unwrap()
                    .metadata()
                    .initial_permissions_policy()
                    .unwrap()
                    .inherited_fullscreen(),
                surgeist_dom::DomFullscreenInherited::Enabled
            );
        }
        assert_eq!(
            before_resume
                .node(iframe)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
    }

    #[test]
    fn changed_url_configuration_requalifies_admitted_reply_after_refusal() {
        admitted_reply_then_refusal(true, false, false);
    }
    #[test]
    fn unchanged_url_configuration_reuses_actual_admitted_reply_without_callback() {
        admitted_reply_then_refusal(false, false, false);
    }
    #[test]
    fn changed_url_configuration_requalifies_actual_sealed_unpublished_policy() {
        admitted_reply_then_refusal(true, true, false);
    }
    #[test]
    fn unchanged_url_configuration_reuses_actual_sealed_unpublished_policy() {
        admitted_reply_then_refusal(false, true, false);
    }
    #[test]
    fn missing_url_service_preserves_admitted_reply_until_same_configuration_returns() {
        admitted_reply_then_refusal(false, false, true);
    }
    #[test]
    fn missing_url_service_preserves_sealed_policy_until_same_configuration_returns() {
        admitted_reply_then_refusal(false, true, true);
    }

    mod receipt_tests;
    mod window_work_tests;
}

impl HostedRuntime {
    fn prepare_child_window(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        child: &DomHostedChildRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let (parent, origin) = match lease
            .hosted_child_step(child)
            .map_err(DomHostedRuntimeError::Dom)?
        {
            surgeist_dom::DomHostedChildStep::NativeExecution(request) => (
                request
                    .parent_global()
                    .ok_or(DomHostedRuntimeError::Protocol)?,
                request.origin().clone(),
            ),
            _ => return Err(DomHostedRuntimeError::Protocol),
        };
        if parent != child.parent_global() {
            return Err(DomHostedRuntimeError::Protocol);
        }
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let (h, t, _, _) = state.find_global(parent, cancel, || {
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
        })?;
        let index = state.child_index(child, cancel, || {
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
        })?;
        if state.children[index].publication.is_some() {
            return Ok(());
        }
        let retained = state.source_retained_bytes(lease, cancel)?;
        let HostedState {
            hosts,
            source,
            work,
            limits,
            ..
        } = &mut *state;
        let group = hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        let found = agent::find_cluster(group, &origin, work, limits.work, cancel, || {
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
        })?;
        let (cluster, added) = if let Some(cluster) = found {
            (cluster, 0)
        } else {
            // The original missing-cluster formation moves the full bank only
            // when necessary, then inserts one actual agent. Scalar admission
            // precedes the mutable canonical payload-retention callback.
            let moves = if group.clusters.len() == group.clusters.capacity() {
                group.clusters.len()
            } else {
                0
            };
            lease
                .admit_hosted_runtime_work(
                    moves
                        .checked_add(1)
                        .ok_or(DomHostedRuntimeError::Admission)?,
                    cancel,
                )
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            agent::form_cluster(group, &origin, retained, *limits, work, cancel, |bytes| {
                lease.retain_hosted_runtime_payload(source, bytes, cancel)
            })?
        };
        lease
            .admit_hosted_runtime_work(
                super::watcher::growth_steps(&group.clusters[cluster].agent.windows.rows)?,
                cancel,
            )
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        super::watcher::reserve_window(
            &mut group.clusters[cluster].agent,
            retained
                .checked_add(added)
                .ok_or(DomHostedRuntimeError::Admission)?,
            *limits,
            work,
            |bytes| lease.retain_hosted_runtime_payload(source, bytes, cancel),
            cancel,
        )?;
        Ok(())
    }
}

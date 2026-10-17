//! Actual origin-keyed native agents within one retained browsing-context group.
use super::*;

/// Reuse the actual origin-keyed cluster, or form its actual agent and loop.
/// Returned bytes are additional retained backing relative to `retained`.
pub(super) fn ensure_cluster(
    group: &mut records::Group,
    origin: &surgeist_dom::DomOpaqueOrigin,
    retained: usize,
    limits: DomHostedRuntimeLimits,
    work: &mut u64,
    cancel: &AtomicBool,
    retain: impl FnMut(usize) -> Result<surgeist_dom::DomHostedRuntimeRetention, surgeist_dom::DomError>,
) -> Result<(usize, usize), DomHostedRuntimeError> {
    if let Some(index) = find_cluster(group, origin, work, limits.work, cancel, || Ok(()))? {
        return Ok((index, 0));
    }
    form_cluster(group, origin, retained, limits, work, cancel, retain)
}

/// Form the actual missing agent after the original lookup found no match.
pub(super) fn form_cluster(
    group: &mut records::Group,
    origin: &surgeist_dom::DomOpaqueOrigin,
    retained: usize,
    limits: DomHostedRuntimeLimits,
    work: &mut u64,
    cancel: &AtomicBool,
    mut retain: impl FnMut(
        usize,
    )
        -> Result<surgeist_dom::DomHostedRuntimeRetention, surgeist_dom::DomError>,
) -> Result<(usize, usize), DomHostedRuntimeError> {
    let header = retain(super::watcher::WindowBank::header_bytes())
        .map_err(|error| DomHostedRuntimeError::Dom(DomHostedError::Dom(error)))?;
    let mut added = 0usize;
    if group.clusters.len() == group.clusters.capacity() {
        let required = group
            .clusters
            .len()
            .checked_add(1)
            .ok_or(DomHostedRuntimeError::Admission)?;
        let requested = required
            .checked_mul(size_of::<records::OriginCluster>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if requested > limits.scratch_bytes
            || retained
                .checked_add(requested)
                .is_none_or(|bytes| bytes > limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        for _ in 0..group.clusters.len() {
            spend(work, limits.work, cancel)?;
        }
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(required)
            .map_err(|_| DomHostedRuntimeError::Admission)?;
        let actual = replacement
            .capacity()
            .checked_mul(size_of::<records::OriginCluster>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if actual > limits.scratch_bytes
            || retained
                .checked_add(actual)
                .is_none_or(|bytes| bytes > limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        let old = group
            .clusters
            .capacity()
            .checked_mul(size_of::<records::OriginCluster>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        added = actual
            .checked_sub(old)
            .ok_or(DomHostedRuntimeError::Admission)?;
        replacement.append(&mut group.clusters);
        group.clusters = replacement;
    }
    let loop_bytes = size_of::<RefCell<records::WindowLoop>>();
    let total = added
        .checked_add(loop_bytes)
        .ok_or(DomHostedRuntimeError::Admission)?;
    if retained
        .checked_add(total)
        .is_none_or(|bytes| bytes > limits.storage_bytes)
    {
        return Err(DomHostedRuntimeError::Admission);
    }
    spend(work, limits.work, cancel)?;
    let index = group.clusters.len();
    group.clusters.push(records::OriginCluster {
        origin: origin.clone(),
        agent: records::NativeAgent {
            event_loop: Rc::new(RefCell::new(records::WindowLoop::new())),
            globals: Vec::new(),
            windows: super::watcher::WindowBank::new(header),
        },
    });
    Ok((index, total))
}

// Grow the actual agent bank with old and full replacement storage admitted
// together. The returned delta is the new retained capacity after old release.
pub(super) fn reserve_global(
    agent: &mut records::NativeAgent,
    retained: usize,
    limits: DomHostedRuntimeLimits,
    work: &mut u64,
    cancel: &AtomicBool,
) -> Result<usize, DomHostedRuntimeError> {
    spend(work, limits.work, cancel)?;
    if agent.globals.len() < agent.globals.capacity() {
        return Ok(0);
    }
    let required = agent
        .globals
        .len()
        .checked_add(1)
        .ok_or(DomHostedRuntimeError::Admission)?;
    let requested = required
        .checked_mul(size_of::<records::Global>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    if requested > limits.scratch_bytes
        || retained
            .checked_add(requested)
            .is_none_or(|bytes| bytes > limits.storage_bytes)
    {
        return Err(DomHostedRuntimeError::Admission);
    }
    for _ in 0..agent.globals.len() {
        spend(work, limits.work, cancel)?;
    }
    let mut replacement = Vec::new();
    replacement
        .try_reserve_exact(required)
        .map_err(|_| DomHostedRuntimeError::Admission)?;
    let actual = replacement
        .capacity()
        .checked_mul(size_of::<records::Global>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    if actual > limits.scratch_bytes
        || retained
            .checked_add(actual)
            .is_none_or(|bytes| bytes > limits.storage_bytes)
    {
        return Err(DomHostedRuntimeError::Admission);
    }
    let old = agent
        .globals
        .capacity()
        .checked_mul(size_of::<records::Global>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    let added = actual
        .checked_sub(old)
        .ok_or(DomHostedRuntimeError::Admission)?;
    replacement.append(&mut agent.globals);
    agent.globals = replacement;
    Ok(added)
}

impl HostedState {
    /// Capture the actual canonical ancestry, not the BC's original creator
    /// chain. Every visited active parent Document must be presently available;
    /// a missing/retired association never becomes a false closed-root result.
    pub(super) fn capture_cross_site(
        &mut self,
        dom: &DomEventDom<'_>,
        document: surgeist_dom::DocumentId,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedRuntimeError> {
        let snapshot = dom.snapshot();
        let mut current = document;
        let mut descendants = Vec::new();
        loop {
            self.step(cancel)?;
            let global = dom
                .raw()
                .relevant_global(current.node())
                .map_err(DomHostedRuntimeError::Dom)?
                .ok_or(DomHostedRuntimeError::Binding)?;
            if dom
                .raw()
                .global_document(global)
                .map_err(DomHostedRuntimeError::Dom)?
                != current
            {
                return Err(DomHostedRuntimeError::Binding);
            }
            let (h, t, c, g) = self.find_global(global, cancel, || Ok(()))?;
            let native = &self.hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .clusters[c]
                .agent
                .globals[g];
            if native.settings.document != Some(current) {
                return Err(DomHostedRuntimeError::Binding);
            }
            let view = snapshot
                .document(current)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            let navigable = view.navigable().ok_or(DomHostedRuntimeError::Inactive)?;
            self.step(cancel)?;
            let node = dom
                .raw()
                .hosted_navigable(navigable)
                .map_err(DomHostedRuntimeError::Dom)?;
            if node.is_retiring() || node.is_retired() {
                return Err(DomHostedRuntimeError::Inactive);
            }
            let Some(parent) = node.parent() else {
                // The recursive source evaluates the parent result before
                // comparing this child. Unwind from the actual root, so a
                // true ancestor result avoids all later origin comparisons.
                let mut parent_document = current;
                while let Some(child_document) = descendants.pop() {
                    self.step(cancel)?;
                    let parent_view = snapshot
                        .document(parent_document)
                        .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
                    let child_view = snapshot
                        .document(child_document)
                        .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
                    if parent_view.metadata().origin() != child_view.metadata().origin() {
                        return Ok(true);
                    }
                    parent_document = child_document;
                }
                return Ok(false);
            };
            self.step(cancel)?;
            let parent = dom
                .raw()
                .hosted_navigable(parent)
                .map_err(DomHostedRuntimeError::Dom)?;
            let parent_document = parent
                .active_document()
                .ok_or(DomHostedRuntimeError::Inactive)?;
            if descendants.len() == descendants.capacity() {
                let required = descendants
                    .len()
                    .checked_add(1)
                    .ok_or(DomHostedRuntimeError::Admission)?;
                let requested = required
                    .checked_mul(size_of::<surgeist_dom::DocumentId>())
                    .ok_or(DomHostedRuntimeError::Admission)?;
                let old = descendants
                    .capacity()
                    .checked_mul(size_of::<surgeist_dom::DocumentId>())
                    .ok_or(DomHostedRuntimeError::Admission)?;
                let retained = self.retained_bytes(cancel)?;
                let peak = old
                    .checked_add(requested)
                    .ok_or(DomHostedRuntimeError::Admission)?;
                if peak > self.limits.scratch_bytes
                    || retained
                        .checked_add(peak)
                        .is_none_or(|n| n > self.limits.storage_bytes)
                {
                    return Err(DomHostedRuntimeError::Admission);
                }
                for _ in 0..descendants.len() {
                    self.step(cancel)?;
                }
                let mut next = Vec::new();
                next.try_reserve_exact(required)
                    .map_err(|_| DomHostedRuntimeError::Admission)?;
                let actual = next
                    .capacity()
                    .checked_mul(size_of::<surgeist_dom::DocumentId>())
                    .ok_or(DomHostedRuntimeError::Admission)?;
                let peak = old
                    .checked_add(actual)
                    .ok_or(DomHostedRuntimeError::Admission)?;
                if peak > self.limits.scratch_bytes
                    || retained
                        .checked_add(peak)
                        .is_none_or(|n| n > self.limits.storage_bytes)
                {
                    return Err(DomHostedRuntimeError::Admission);
                }
                next.append(&mut descendants);
                descendants = next;
            }
            self.step(cancel)?;
            descendants.push(current);
            current = parent_document;
        }
    }
}

pub(super) fn find_cluster(
    group: &records::Group,
    origin: &surgeist_dom::DomOpaqueOrigin,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
    mut admit: impl FnMut() -> Result<(), DomHostedRuntimeError>,
) -> Result<Option<usize>, DomHostedRuntimeError> {
    for (index, cluster) in group.clusters.iter().enumerate() {
        spend(work, limit, cancel)?;
        admit()?;
        if &cluster.origin == origin {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

use crate::accounting::{
    Account, AccountedVec, RenderAccountingError, RenderResource, RenderWork, SharedValue,
};
use std::{
    cell::{Cell, RefCell},
    ops::{Deref, DerefMut},
};

use super::lower::{
    runtime_pass_cache_keys, runtime_resource_format, shader_binding_role, shader_sampling_edge,
};
use super::model::{
    LoweredGraphPlan, RuntimeBlur, RuntimeBlurAxis, RuntimeBlurInput, RuntimeClipCoverage,
    RuntimeColorClampBoundary, RuntimeColorFilter, RuntimeColorOperation,
    RuntimeColorOperationKind, RuntimeComposite, RuntimeCompositeKind,
    RuntimeDestinationToLayerLocal, RuntimeDropShadow, RuntimeFilterSpatialMapping,
    RuntimeInitialization, RuntimeLayerCompositeParameters, RuntimeMaskTexelCenterFacts,
    RuntimeOuterClip, RuntimePass, RuntimePassId, RuntimePassKind, RuntimeReadBinding,
    RuntimeReadRole, RuntimeResolvedAlphaMaskComposition, RuntimeResourceFormat, RuntimeResourceId,
    RuntimeResourceImport, RuntimeResourceProducer, RuntimeResourceRequest, RuntimeResourceRole,
    RuntimeResultBinding, RuntimeSamplingEdge, RuntimeSamplingFilter, RuntimeSpatialDescriptor,
    RuntimeVelloCapture, RuntimeVelloSpan, RuntimeVelloSpanScope,
    runtime_affine_is_finite_and_non_singular,
};
use crate::{
    BackendErrorCode, Color, Error, Format, PhysicalSize, Point, Result, Transform,
    command::RenderClip,
    renderer::Antialiasing,
    resource::WorkingFormat,
    shader::{SamplerKey, ShaderMaskQualityKey, ShaderMaskSamplingKey, ShaderSamplingFilterKey},
};

/// Pure proof routines keep their semantic Option/bool result while this
/// adapter preserves the first accounting failure for their fallible owner.
/// No native or publication effect occurs inside a proof.
struct ProofContext<'a> {
    account: &'a Account,
    error: RefCell<Option<Error>>,
    invalid: Cell<bool>,
}
impl<'a> ProofContext<'a> {
    fn new(account: &'a Account) -> Self {
        Self {
            account,
            error: RefCell::new(None),
            invalid: Cell::new(false),
        }
    }
    fn capture<T>(&self, result: Result<T>) -> Option<T> {
        if self.error.borrow().is_some() {
            return None;
        }
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                *self.error.borrow_mut() = Some(error);
                None
            }
        }
    }
    fn capture_storage_write(&self, result: Result<()>) -> Option<()> {
        // Successful writes keep the original traversal prefix after a fault.
        match result {
            Ok(()) => Some(()),
            Err(error) => self.capture(Err(error)),
        }
    }
    fn work(&self, work: RenderWork, amount: usize) -> bool {
        if self.error.borrow().is_some() {
            return false;
        }
        self.capture(self.account.work(work, amount)).is_some()
    }
    fn finish(&self) -> Result<()> {
        match self.error.borrow_mut().take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
    fn shared_copy<T: Clone>(&self, source: &[T]) -> Option<SharedValue<Vec<T>>> {
        if self.error.borrow().is_some() {
            return None;
        }
        let mut values = self.capture(AccountedVec::new(
            self.account,
            source.len(),
            RenderResource::RetainedCpuBytes,
        ))?;
        self.capture(values.extend_from_slice(source))?;
        self.capture(SharedValue::from_accounted(values, &self.account.storage()))
    }
}
/// One sorted slot per source identity. Updates never allocate or shift entries.
/// Only actual typed Vec capacity is charged; opaque tree/bucket sizes are absent.
struct ProofMap<'a, K, V> {
    entries: AccountedVec<(K, Option<V>)>,
    proof: &'a ProofContext<'a>,
    live: usize,
}
impl<'a, K: Ord, V> ProofMap<'a, K, V> {
    fn new(
        keys: impl IntoIterator<Item = K>,
        count: usize,
        proof: &'a ProofContext<'a>,
    ) -> Result<Self> {
        let mut entries = AccountedVec::new(proof.account, count, RenderResource::ScratchCpuBytes)?;
        for key in keys {
            proof.account.work(RenderWork::GraphNodeVisits, 1)?;
            entries.append_slot()?.write((key, None));
        }
        sort_proof_entries(&mut entries, proof.account)?;
        for pair in entries.windows(2) {
            proof.account.work(RenderWork::ResourceProbes, 1)?;
            if pair[0].0 == pair[1].0 {
                proof.invalid.set(true);
                break;
            }
        }
        Ok(Self {
            entries,
            proof,
            live: 0,
        })
    }
    fn position(&self, key: &K) -> Option<usize> {
        let mut lower = 0;
        let mut upper = self.entries.len();
        while lower < upper {
            let middle = lower + (upper - lower) / 2;
            if !self.proof.work(RenderWork::ResourceProbes, 1) {
                return None;
            }
            match self.entries[middle].0.cmp(key) {
                std::cmp::Ordering::Less => lower = middle + 1,
                std::cmp::Ordering::Greater => upper = middle,
                std::cmp::Ordering::Equal => return Some(middle),
            }
        }
        None
    }
    fn get(&self, key: &K) -> Option<&V> {
        self.entries[self.position(key)?].1.as_ref()
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let Some(position) = self.position(&key) else {
            if self.proof.error.borrow().is_none() {
                self.proof.invalid.set(true);
            }
            return None;
        };
        let old = self.entries[position].1.replace(value);
        if old.is_none() {
            self.live += 1;
        }
        old
    }
    fn get_or_insert_default(&mut self, key: K) -> Option<&mut V>
    where
        V: Default,
    {
        let Some(position) = self.position(&key) else {
            self.proof.invalid.set(true);
            return None;
        };
        if self.entries[position].1.is_none() {
            self.live += 1;
        }
        Some(self.entries[position].1.get_or_insert_default())
    }
    fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    fn len(&self) -> usize {
        self.live
    }
}

/// In-place heapsort can stop at the first refused comparison. Unlike an
/// infallible sort comparator, it never needs an invented ordering on refusal.
fn sort_proof_entries<K: Ord, V>(entries: &mut [(K, Option<V>)], account: &Account) -> Result<()> {
    fn sift_down<K: Ord, V>(
        entries: &mut [(K, Option<V>)],
        mut root: usize,
        end: usize,
        account: &Account,
    ) -> Result<()> {
        while root < end / 2 {
            let left = root * 2 + 1;
            let right = left + 1;
            let child = if right < end {
                account.work(RenderWork::ResourceProbes, 1)?;
                if entries[left].0.cmp(&entries[right].0).is_lt() {
                    right
                } else {
                    left
                }
            } else {
                left
            };
            account.work(RenderWork::ResourceProbes, 1)?;
            if entries[root].0.cmp(&entries[child].0).is_ge() {
                break;
            }
            entries.swap(root, child);
            root = child;
        }
        Ok(())
    }

    let len = entries.len();
    for root in (0..len / 2).rev() {
        sift_down(entries, root, len, account)?;
    }
    for end in (1..len).rev() {
        entries.swap(0, end);
        sift_down(entries, 0, end, account)?;
    }
    Ok(())
}
/// Reusable source-key set. Epoch changes clear a pass in constant time; final
/// disposal remains one linear drop of the actual typed backing.
struct ProofSet<'a, K> {
    slots: ProofMap<'a, K, u64>,
    epoch: u64,
    count: usize,
}
impl<'a, K: Ord> ProofSet<'a, K> {
    fn new(
        keys: impl IntoIterator<Item = K>,
        count: usize,
        proof: &'a ProofContext<'a>,
    ) -> Result<Self> {
        let mut slots = ProofMap::new(keys, count, proof)?;
        for entry in slots.entries.iter_mut() {
            entry.1 = Some(0);
        }
        slots.live = slots.entries.len();
        Ok(Self {
            slots,
            epoch: 1,
            count: 0,
        })
    }
    fn clear(&mut self) -> Option<()> {
        let Some(next) = self.epoch.checked_add(1) else {
            return self
                .slots
                .proof
                .capture(Err(RenderAccountingError::Invariant.into()));
        };
        self.epoch = next;
        self.count = 0;
        Some(())
    }
    fn insert(&mut self, key: K) -> bool {
        let Some(position) = self.slots.position(&key) else {
            if self.slots.proof.error.borrow().is_none() {
                self.slots.proof.invalid.set(true);
            }
            return false;
        };
        let slot = &mut self.slots.entries[position].1;
        if *slot == Some(self.epoch) {
            return false;
        }
        *slot = Some(self.epoch);
        self.count += 1;
        true
    }
    fn extend(&mut self, keys: impl IntoIterator<Item = K>) {
        for key in keys {
            self.insert(key);
        }
    }
    fn contains(&self, key: &K) -> bool {
        self.slots.get(key) == Some(&self.epoch)
    }
    fn len(&self) -> usize {
        self.count
    }
    fn iter(&self) -> impl Iterator<Item = &K> {
        self.slots
            .entries
            .iter()
            .filter_map(|(key, epoch)| (*epoch == Some(self.epoch)).then_some(key))
    }
}
/// The traversal has at most one fact/context per consumed pass. Admission and
/// allocation happen before the pure traversal; each write requires a reserved slot.
struct ProofVec<T>(AccountedVec<T>);
impl<T> ProofVec<T> {
    fn new(account: &Account, count: usize, resource: RenderResource) -> Result<Self> {
        Ok(Self(AccountedVec::new(account, count, resource)?))
    }
    fn push(&mut self, value: T) -> Result<()> {
        self.0.append_slot()?.write(value);
        Ok(())
    }
    fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
    fn into_shared(self, account: &Account) -> Result<SharedValue<Vec<T>>> {
        SharedValue::from_accounted(self.0, &account.storage())
    }
}
impl<T> Deref for ProofVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.0
    }
}
impl<T> DerefMut for ProofVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

fn proof_nodes<'a, T>(nodes: &'a [T], proof: &'a ProofContext<'_>) -> impl Iterator<Item = &'a T> {
    nodes
        .iter()
        .take_while(move |_| proof.work(RenderWork::GraphNodeVisits, 1))
}

fn proof_passes<'a>(
    plan: &'a LoweredGraphPlan,
    proof: &'a ProofContext<'_>,
) -> impl Iterator<Item = &'a RuntimePass> {
    proof_nodes(&plan.passes, proof)
}
fn proof_resources<'a>(
    plan: &'a LoweredGraphPlan,
    proof: &'a ProofContext<'_>,
) -> impl Iterator<Item = &'a RuntimeResourceRequest> {
    proof_nodes(&plan.resources, proof)
}

fn proof_relationships_equal<A, E>(
    actual: impl IntoIterator<Item = A>,
    expected: impl IntoIterator<Item = E>,
    proof: &ProofContext<'_>,
    equal: impl Fn(A, E) -> bool,
) -> bool {
    let mut expected = expected.into_iter();
    for actual in actual {
        if !proof.work(RenderWork::GraphEdgeVisits, 1) {
            return false;
        }
        if expected
            .next()
            .is_none_or(|expected| !equal(actual, expected))
        {
            return false;
        }
    }
    expected.next().is_none()
}

fn proof_edges_equal<T: PartialEq>(actual: &[T], expected: &[T], proof: &ProofContext<'_>) -> bool {
    actual.len() == expected.len()
        && proof_relationships_equal(actual, expected, proof, |actual, expected| {
            actual == expected
        })
}

fn proof_contains_edge<T: PartialEq>(edges: &[T], expected: &T, proof: &ProofContext<'_>) -> bool {
    for edge in edges {
        if !proof.work(RenderWork::GraphEdgeVisits, 1) {
            return false;
        }
        if edge == expected {
            return true;
        }
    }
    false
}

fn proof_reads<'a>(
    pass: &'a RuntimePass,
    proof: &'a ProofContext<'_>,
) -> impl Iterator<Item = &'a RuntimeReadBinding> {
    pass.reads
        .iter()
        .take_while(move |_| proof.work(RenderWork::GraphEdgeVisits, 1))
}
fn proof_read<'a>(
    pass: &'a RuntimePass,
    index: usize,
    proof: &ProofContext<'_>,
) -> Option<&'a RuntimeReadBinding> {
    let read = pass.reads.get(index)?;
    proof.work(RenderWork::GraphEdgeVisits, 1).then_some(read)
}

#[derive(Clone)]
pub(super) struct ExecutableLayerCompositionFacts {
    pub(super) pass: RuntimePassId,
    pub(super) parent: RuntimeResourceId,
    pub(super) source: RuntimeResourceId,
    pub(super) clip_coverage: Option<RuntimeResourceId>,
    pub(super) alpha_mask: Option<RuntimeResourceId>,
    pub(super) result: RuntimeResourceId,
    pub(super) composite: RuntimeComposite,
}

#[derive(Clone)]
pub(super) struct ExecutableColorFilterFacts {
    pub(super) pass: RuntimePassId,
    pub(super) source: RuntimeResourceId,
    pub(super) result: RuntimeResourceId,
    pub(super) filter: RuntimeColorFilter,
}

#[derive(Clone)]
pub(super) struct ExecutableBlurFacts {
    pub(super) horizontal: RuntimePassId,
    pub(super) vertical: RuntimePassId,
    pub(super) source: RuntimeResourceId,
    pub(super) intermediate: RuntimeResourceId,
    pub(super) result: RuntimeResourceId,
    pub(super) blur: RuntimeBlur,
}

#[derive(Clone)]
pub(super) struct ExecutableDropShadowFacts {
    pub(super) horizontal: RuntimePassId,
    pub(super) vertical: RuntimePassId,
    pub(super) colorize: RuntimePassId,
    pub(super) merge: RuntimePassId,
    pub(super) source: RuntimeResourceId,
    pub(super) horizontal_result: RuntimeResourceId,
    pub(super) vertical_result: RuntimeResourceId,
    pub(super) shadow: RuntimeResourceId,
    pub(super) result: RuntimeResourceId,
    pub(super) blur: RuntimeBlur,
    pub(super) parameters: RuntimeDropShadow,
}

#[derive(Clone)]
pub(super) struct ExecutableBackdropFacts {
    pub(super) copy: RuntimePassId,
    pub(super) completed_parent: RuntimeResourceId,
    pub(super) copied: RuntimeResourceId,
    pub(super) foreground: Option<RuntimeResourceId>,
    pub(super) filter_steps: SharedValue<Vec<ExecutableFilterStepFacts>>,
    pub(super) filtered: RuntimeResourceId,
    pub(super) group_clear: RuntimePassId,
    pub(super) backdrop_composite: RuntimePassId,
    pub(super) foreground_composite: Option<RuntimePassId>,
    pub(super) outer_composite: RuntimePassId,
    pub(super) completed_group: RuntimeResourceId,
    pub(super) result: RuntimeResourceId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExecutableFilterStepFacts {
    Color(RuntimePassId),
    Blur {
        horizontal: RuntimePassId,
        vertical: RuntimePassId,
    },
    DropShadow {
        horizontal: RuntimePassId,
        vertical: RuntimePassId,
        colorize: RuntimePassId,
        merge: RuntimePassId,
    },
}

#[derive(Clone)]
pub(super) struct ClosedExecutableGraphFacts {
    pub(super) working_format: WorkingFormat,
    pub(super) output_format: Format,
    pub(super) captures: SharedValue<Vec<ExecutableVelloCaptureFacts>>,
    pub(super) layer_compositions: SharedValue<Vec<ExecutableLayerCompositionFacts>>,
    pub(super) color_filters: SharedValue<Vec<ExecutableColorFilterFacts>>,
    pub(super) blurs: SharedValue<Vec<ExecutableBlurFacts>>,
    pub(super) drop_shadows: SharedValue<Vec<ExecutableDropShadowFacts>>,
    pub(super) filter_steps: SharedValue<Vec<ExecutableFilterStepFacts>>,
    pub(super) backdrops: SharedValue<Vec<ExecutableBackdropFacts>>,
}

#[derive(Clone, Copy)]
struct ExecutableCompositionContext {
    current: RuntimeResourceId,
    producer: RuntimePassId,
    contains_captured_source: bool,
}

#[must_use = "a closed executable graph must reach dispatch or explicit rejection"]
pub(super) struct ClosedExecutableGraph {
    pub(super) lowered: LoweredGraphPlan,
    pub(super) facts: SharedValue<ClosedExecutableGraphFacts>,
}

impl ClosedExecutableGraph {
    pub(super) fn try_from_lowered(
        lowered: LoweredGraphPlan,
        account: &Account,
    ) -> Result<std::result::Result<Self, LoweredGraphPlan>> {
        let Some(facts) = lowered.closed_executable_graph_facts(account)? else {
            return Ok(Err(lowered));
        };
        if !facts.proves_exact_facts_for(&lowered, account)? {
            return Ok(Err(lowered));
        }
        let facts = SharedValue::new(facts, &account.storage())?;
        Ok(Ok(Self { lowered, facts }))
    }

    pub(super) fn has_layer_composition(&self) -> bool {
        !self.facts.layer_compositions.is_empty()
    }

    fn has_color_filters(&self) -> bool {
        !self.facts.color_filters.is_empty()
    }

    fn has_spatial_filters(&self) -> bool {
        !self.facts.blurs.is_empty() || !self.facts.drop_shadows.is_empty()
    }

    fn has_backdrops(&self) -> bool {
        !self.facts.backdrops.is_empty()
    }
}

impl ClosedExecutableGraphFacts {
    pub(super) fn proves_exact_facts_for(
        &self,
        plan: &LoweredGraphPlan,
        account: &Account,
    ) -> Result<bool> {
        let proof = ProofContext::new(account);
        let exact = (|| -> bool {
            if self.working_format != plan.working_format
                || self.output_format != plan.output_format
                || (self.captures.is_empty() && self.layer_compositions.is_empty())
            {
                return false;
            }
            let captures_are_exact = proof_nodes(&self.captures, &proof).all(|capture| {
                proof_passes(plan, &proof).any(|pass| {
                    pass.id == capture.pass()
                        && matches!(
                            &pass.kind,
                            RuntimePassKind::VelloCapture(Some(work))
                                if same_source_capture(work, capture.work())
                                    && work.antialiasing() == capture.antialiasing()
                        )
                        && pass.result == RuntimeResultBinding::Resource(capture.target())
                })
            });
            let layers_are_exact = proof_nodes(&self.layer_compositions, &proof).all(|layer| {
                let Some(pass) = proof_passes(plan, &proof).find(|pass| pass.id == layer.pass)
                else {
                    return false;
                };
                let RuntimePassKind::Composite(Some(composite)) = &pass.kind else {
                    return false;
                };
                let expected_reads = [
                    Some(layer.parent),
                    Some(layer.source),
                    layer.clip_coverage,
                    layer.alpha_mask,
                ]
                .into_iter()
                .flatten();
                same_source_composite(composite, &layer.composite)
                    && proof_reads(pass, &proof)
                        .map(|read| read.resource)
                        .eq(expected_reads)
                    && pass.result == RuntimeResultBinding::Resource(layer.result)
            });
            let color_filters_are_exact = proof_nodes(&self.color_filters, &proof).all(|color| {
                proof_passes(plan, &proof).any(|pass| {
                    pass.id == color.pass
                        && matches!(
                            &pass.kind,
                            RuntimePassKind::ColorFilter(Some(filter)) if same_source_color_filter(filter, &color.filter)
                        )
                        && pass.reads.len() == 1
                        && proof_read(pass, 0, &proof)
                            .is_some_and(|read| read.resource == color.source)
                        && pass.result == RuntimeResultBinding::Resource(color.result)
                })
            }) && self.color_filters.len()
                == proof_passes(plan, &proof)
                    .filter(|pass| matches!(pass.kind, RuntimePassKind::ColorFilter(Some(_))))
                    .count();
            let blurs_are_exact = proof_nodes(&self.blurs, &proof)
                .all(|blur| blur.proves_exact_facts_for(plan, &proof))
                && self.blurs.len()
                    == proof_passes(plan, &proof)
                        .filter(|pass| {
                            matches!(
                                pass.kind,
                                RuntimePassKind::BlurHorizontal(Some(RuntimeBlur {
                                    input: RuntimeBlurInput::Rgba,
                                    ..
                                }))
                            )
                        })
                        .count();
            let drop_shadows_are_exact = proof_nodes(&self.drop_shadows, &proof)
                .all(|shadow| shadow.proves_exact_facts_for(plan, &proof))
                && self.drop_shadows.len()
                    == proof_passes(plan, &proof)
                        .filter(|pass| {
                            matches!(pass.kind, RuntimePassKind::DropShadowColorize(Some(_)))
                        })
                        .count();
            let filter_order_is_exact =
                filter_step_order_is_exact(plan, &self.filter_steps, &proof);
            let backdrops_are_exact = proof_nodes(&self.backdrops, &proof)
                .all(|backdrop| backdrop.proves_exact_facts_for(plan, &proof));
            captures_are_exact
                && layers_are_exact
                && color_filters_are_exact
                && blurs_are_exact
                && drop_shadows_are_exact
                && filter_order_is_exact
                && backdrops_are_exact
        })();
        proof.finish()?;
        Ok(exact)
    }
}

impl ExecutableBackdropFacts {
    fn proves_exact_facts_for(&self, plan: &LoweredGraphPlan, proof: &ProofContext<'_>) -> bool {
        let pass = |id| proof_passes(plan, proof).find(|candidate| candidate.id == id);
        let Some(copy) = pass(self.copy) else {
            return false;
        };
        let foreground_is_distinct = self
            .foreground
            .is_none_or(|foreground| foreground != self.copied && foreground != self.filtered);
        matches!(copy.kind, RuntimePassKind::CopyBackdrop)
            && copy.reads.len() == 1
            && proof_read(copy, 0, proof).is_some_and(|read| read.resource == self.completed_parent)
            && copy.result == RuntimeResultBinding::Resource(self.copied)
            && pass(self.group_clear).is_some_and(|clear| {
                matches!(
                    clear.kind,
                    RuntimePassKind::ClearRoot {
                        initialization: RuntimeInitialization::Transparent,
                        color,
                    } if color == Color::TRANSPARENT
                )
            })
            && pass(self.backdrop_composite).is_some()
            && self
                .foreground_composite
                .is_none_or(|id| pass(id).is_some())
            && pass(self.outer_composite).is_some()
            && self.completed_group != self.completed_parent
            && self.result != self.completed_group
            && foreground_is_distinct
    }
}

impl ExecutableBlurFacts {
    fn proves_exact_facts_for(&self, plan: &LoweredGraphPlan, proof: &ProofContext<'_>) -> bool {
        let Some(horizontal) = proof_passes(plan, proof).find(|pass| pass.id == self.horizontal)
        else {
            return false;
        };
        let Some(vertical) = proof_passes(plan, proof).find(|pass| pass.id == self.vertical) else {
            return false;
        };
        matches!(
            &horizontal.kind,
            RuntimePassKind::BlurHorizontal(Some(blur))
                if runtime_blur_matches_axis(blur, &self.blur, RuntimeBlurAxis::Horizontal)
        ) && matches!(
            &vertical.kind,
            RuntimePassKind::BlurVertical(Some(blur))
                if runtime_blur_matches_axis(blur, &self.blur, RuntimeBlurAxis::Vertical)
        ) && proof_read(horizontal, 0, proof).is_some_and(|read| read.resource == self.source)
            && horizontal.result == RuntimeResultBinding::Resource(self.intermediate)
            && proof_read(vertical, 0, proof).is_some_and(|read| read.resource == self.intermediate)
            && vertical.result == RuntimeResultBinding::Resource(self.result)
    }
}

impl ExecutableDropShadowFacts {
    fn proves_exact_facts_for(&self, plan: &LoweredGraphPlan, proof: &ProofContext<'_>) -> bool {
        let pass = |id| proof_passes(plan, proof).find(|pass| pass.id == id);
        let (Some(horizontal), Some(vertical), Some(colorize), Some(merge)) = (
            pass(self.horizontal),
            pass(self.vertical),
            pass(self.colorize),
            pass(self.merge),
        ) else {
            return false;
        };
        matches!(
            &horizontal.kind,
            RuntimePassKind::BlurHorizontal(Some(blur))
                if runtime_blur_matches_axis(blur, &self.blur, RuntimeBlurAxis::Horizontal)
        ) && matches!(
            &vertical.kind,
            RuntimePassKind::BlurVertical(Some(blur))
                if runtime_blur_matches_axis(blur, &self.blur, RuntimeBlurAxis::Vertical)
        ) && matches!(
            &colorize.kind,
            RuntimePassKind::DropShadowColorize(Some(parameters))
                if parameters == &self.parameters
        ) && matches!(
            &merge.kind,
            RuntimePassKind::Composite(Some(RuntimeComposite {
                kind: RuntimeCompositeKind::DropShadow,
                ..
            }))
        ) && proof_read(horizontal, 0, proof).is_some_and(|read| read.resource == self.source)
            && horizontal.result == RuntimeResultBinding::Resource(self.horizontal_result)
            && vertical.result == RuntimeResultBinding::Resource(self.vertical_result)
            && colorize.result == RuntimeResultBinding::Resource(self.shadow)
            && merge.result == RuntimeResultBinding::Resource(self.result)
    }
}

fn runtime_blur_matches_axis(
    candidate: &RuntimeBlur,
    expected: &RuntimeBlur,
    axis: RuntimeBlurAxis,
) -> bool {
    let expected_spatial = match axis {
        RuntimeBlurAxis::Horizontal => expected.spatial,
        RuntimeBlurAxis::Vertical => RuntimeFilterSpatialMapping {
            source: expected.spatial.result,
            result: expected.spatial.result,
        },
    };
    candidate.axis == axis
        && candidate.input == expected.input
        && candidate.standard_deviation == expected.standard_deviation
        && candidate.support_radius == expected.support_radius
        && candidate.kernel == expected.kernel
        && candidate.spatial == expected_spatial
        && candidate.edge == expected.edge
}

// These private source checks are used only for facts cloned from the exact
// borrowed plan payload: executable_vello_capture_facts, validate_closed_color_filter
// and validate_closed_layer_composite are the sole production constructors.
// Closed/Base/repeated proofs retain that immutable plan. Normalization and the
// mandatory planning/bounds validation establish finite, reflexive payloads.
// Compare every fixed field, while shared variable payload identity proves that
// no content walk is needed here. Generic PartialEq retains content semantics.
fn same_source_span(actual: &RuntimeVelloSpan, expected: &RuntimeVelloSpan) -> bool {
    actual.scope == expected.scope
        && actual.source_space == expected.source_space
        && actual
            .commands
            .commands
            .same_allocation(&expected.commands.commands)
        && actual.capture_transform == expected.capture_transform
        && actual.parent_to_surface == expected.parent_to_surface
        && actual.antialiasing == expected.antialiasing
        && actual.captured_before_outer_semantics == expected.captured_before_outer_semantics
}

fn same_source_capture(actual: &RuntimeVelloCapture, expected: &RuntimeVelloCapture) -> bool {
    match (actual, expected) {
        (RuntimeVelloCapture::Span(actual), RuntimeVelloCapture::Span(expected)) => {
            same_source_span(actual, expected)
        }
        (
            RuntimeVelloCapture::ClipCoverage(actual),
            RuntimeVelloCapture::ClipCoverage(expected),
        ) => {
            actual.elements.same_allocation(&expected.elements)
                && actual.antialiasing == expected.antialiasing
        }
        _ => false,
    }
}

fn same_source_color_filter(actual: &RuntimeColorFilter, expected: &RuntimeColorFilter) -> bool {
    actual.operations.same_allocation(&expected.operations)
        && actual.spatial == expected.spatial
        && actual.edge == expected.edge
}

fn same_source_composite(actual: &RuntimeComposite, expected: &RuntimeComposite) -> bool {
    actual.source_captured_before_outer_semantics == expected.source_captured_before_outer_semantics
        && match (&actual.kind, &expected.kind) {
            (RuntimeCompositeKind::SpanSourceOver, RuntimeCompositeKind::SpanSourceOver)
            | (RuntimeCompositeKind::DropShadow, RuntimeCompositeKind::DropShadow) => true,
            (
                RuntimeCompositeKind::Layer {
                    transform: a,
                    parameters: ap,
                    clip: ac,
                    outer_clips: ao,
                    clip_coverage: av,
                },
                RuntimeCompositeKind::Layer {
                    transform: b,
                    parameters: bp,
                    clip: bc,
                    outer_clips: bo,
                    clip_coverage: bv,
                },
            ) => {
                a == b
                    && **ap == **bp
                    && match (ac, bc) {
                        (None, None) => true,
                        (Some(a), Some(b)) => same_source_clip(a, b),
                        _ => false,
                    }
                    && ao.same_allocation(bo)
                    && av == bv
            }
            _ => false,
        }
}

fn same_source_clip(actual: &RenderClip, expected: &RenderClip) -> bool {
    use crate::command::RenderClipGeometry;
    let geometry_matches = match (actual.geometry(), expected.geometry()) {
        (
            RenderClipGeometry::Path {
                path: a,
                fill_rule: ar,
            },
            RenderClipGeometry::Path {
                path: b,
                fill_rule: br,
            },
        ) => a.same_allocation(b) && ar == br,
        (RenderClipGeometry::Path { .. }, _) | (_, RenderClipGeometry::Path { .. }) => false,
        (a, b) => a == b,
    };
    geometry_matches && actual.coordinate_space() == expected.coordinate_space()
}

fn clips_match(actual: &RenderClip, expected: &RenderClip, proof: &ProofContext<'_>) -> bool {
    use crate::command::RenderClipGeometry;
    let geometry_matches = match (actual.geometry(), expected.geometry()) {
        (
            RenderClipGeometry::Path {
                path: a,
                fill_rule: ar,
            },
            RenderClipGeometry::Path {
                path: b,
                fill_rule: br,
            },
        ) => {
            (a.same_allocation(b)
                || (a.elements().len() == b.elements().len()
                    && a.elements()
                        .iter()
                        .zip(b.elements())
                        .all(|(a, b)| proof.work(RenderWork::PathElementVisits, 1) && a == b)))
                && ar == br
        }
        (RenderClipGeometry::Path { .. }, _) | (_, RenderClipGeometry::Path { .. }) => false,
        (a, b) => a == b,
    };
    geometry_matches && actual.coordinate_space() == expected.coordinate_space()
}

fn filter_steps_match(
    actual: &[ExecutableFilterStepFacts],
    expected: &[ExecutableFilterStepFacts],
    account: &Account,
) -> Result<bool> {
    if actual.len() != expected.len() {
        return Ok(false);
    }
    for (a, b) in actual.iter().zip(expected) {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        if a != b {
            return Ok(false);
        }
    }
    Ok(true)
}

fn filter_step_order_is_exact(
    plan: &LoweredGraphPlan,
    expected: &[ExecutableFilterStepFacts],
    proof: &ProofContext<'_>,
) -> bool {
    let result = (|| -> Option<()> {
        let mut step_index = 0;
        macro_rules! step {
            ($step:expr) => {{
                let expected_step = expected.get(step_index)?;
                if !proof.work(RenderWork::GraphNodeVisits, 1) {
                    return None;
                }
                if expected_step != &$step {
                    return None;
                }
                step_index += 1;
            }};
        }
        let mut cursor = 0_usize;
        while cursor < plan.passes.len() {
            if !proof.work(RenderWork::GraphNodeVisits, 1) {
                return None;
            }
            let pass = &plan.passes[cursor];
            match &pass.kind {
                RuntimePassKind::ColorFilter(Some(_)) => {
                    step!(ExecutableFilterStepFacts::Color(pass.id));
                    cursor = cursor.checked_add(1)?;
                }
                RuntimePassKind::BlurHorizontal(Some(blur))
                    if blur.input == RuntimeBlurInput::Rgba =>
                {
                    let vertical = plan.passes.get(cursor.checked_add(1)?)?;
                    if !proof.work(RenderWork::GraphNodeVisits, 1) {
                        return None;
                    }
                    step!(ExecutableFilterStepFacts::Blur {
                        horizontal: pass.id,
                        vertical: vertical.id,
                    });
                    cursor = cursor.checked_add(2)?;
                }
                RuntimePassKind::BlurHorizontal(Some(blur))
                    if blur.input == RuntimeBlurInput::SourceAlpha =>
                {
                    let vertical = plan.passes.get(cursor.checked_add(1)?)?;
                    if !proof.work(RenderWork::GraphNodeVisits, 1) {
                        return None;
                    }
                    let colorize = plan.passes.get(cursor.checked_add(2)?)?;
                    if !proof.work(RenderWork::GraphNodeVisits, 1) {
                        return None;
                    }
                    let merge = plan.passes.get(cursor.checked_add(3)?)?;
                    if !proof.work(RenderWork::GraphNodeVisits, 1) {
                        return None;
                    }
                    step!(ExecutableFilterStepFacts::DropShadow {
                        horizontal: pass.id,
                        vertical: vertical.id,
                        colorize: colorize.id,
                        merge: merge.id,
                    });
                    cursor = cursor.checked_add(4)?;
                }
                _ => cursor = cursor.checked_add(1)?,
            }
        }
        (step_index == expected.len()).then_some(())
    })();
    result.is_some()
}
#[must_use]
#[derive(Clone)]
pub(crate) struct BaseExecutionFacts {
    pub(super) working_format: WorkingFormat,
    pub(super) output_format: Format,
    pub(super) captures: SharedValue<Vec<ExecutableVelloCaptureFacts>>,
}

impl BaseExecutionFacts {
    #[must_use]
    pub(crate) const fn working_format(&self) -> WorkingFormat {
        self.working_format
    }

    #[must_use]
    pub(crate) const fn output_format(&self) -> Format {
        self.output_format
    }

    #[must_use]
    pub(crate) fn captures(&self) -> &[ExecutableVelloCaptureFacts] {
        &self.captures
    }

    pub(super) fn proves_exact_execution_facts_for(
        &self,
        plan: &LoweredGraphPlan,
        account: &Account,
    ) -> Result<bool> {
        let proof = ProofContext::new(account);
        let mut passes = ProofSet::new(
            plan.passes.iter().map(|pass| pass.id),
            plan.passes.len(),
            &proof,
        )?;
        let mut targets = ProofSet::new(
            plan.resources.iter().map(|resource| resource.id),
            plan.resources.len(),
            &proof,
        )?;
        let exact = (|| -> bool {
            if self.working_format() != plan.working_format
                || self.output_format() != plan.output_format
            {
                return false;
            }
            proof_nodes(self.captures(), &proof).all(|capture| {
                let Some(pass) = proof_passes(plan, &proof).find(|pass| pass.id == capture.pass())
                else {
                    return false;
                };
                let RuntimePassKind::VelloCapture(Some(RuntimeVelloCapture::Span(span))) =
                    &pass.kind
                else {
                    return false;
                };
                let Some(capture_span) = capture.span() else {
                    return false;
                };
                let Some(target) =
                    proof_resources(plan, &proof).find(|resource| resource.id == capture.target())
                else {
                    return false;
                };
                passes.insert(capture.pass())
                    && targets.insert(capture.target())
                    && capture_span.scope == RuntimeVelloSpanScope::CurrentParent
                    && same_source_span(capture_span, span)
                    && capture
                        .initial_transform()
                        .as_array()
                        .iter()
                        .all(|value| value.is_finite())
                    && capture.antialiasing() == span.antialiasing
                    && capture.target_extent() == target.spatial.device_extent
                    && capture.texel_origin() == target.spatial.texel_origin
                    && capture.raster_scale() == target.spatial.raster_scale
            })
        })();
        proof.finish()?;
        Ok(exact && !proof.invalid.get())
    }
}

#[must_use]
pub(crate) struct BasePreparableGraph {
    pub(super) lowered: LoweredGraphPlan,
    pub(super) execution: BaseExecutionFacts,
}

impl BasePreparableGraph {
    pub(super) fn try_from_closed(
        closed: ClosedExecutableGraph,
        account: &Account,
    ) -> Result<std::result::Result<Self, ClosedExecutableGraph>> {
        if closed.has_layer_composition()
            || closed.has_color_filters()
            || closed.has_spatial_filters()
            || closed.has_backdrops()
            || closed.facts.captures.is_empty()
        {
            return Ok(Err(closed));
        }
        for capture in &closed.facts.captures {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            if capture
                .span()
                .is_none_or(|span| span.scope != RuntimeVelloSpanScope::CurrentParent)
            {
                return Ok(Err(closed));
            }
        }
        let execution = BaseExecutionFacts {
            working_format: closed.facts.working_format,
            output_format: closed.facts.output_format,
            captures: closed.facts.captures.clone(),
        };
        if !execution.proves_exact_execution_facts_for(&closed.lowered, account)? {
            return Ok(Err(closed));
        }
        Ok(Ok(Self {
            lowered: closed.lowered,
            execution,
        }))
    }

    pub(super) fn parts(&self) -> (&LoweredGraphPlan, &BaseExecutionFacts) {
        (&self.lowered, &self.execution)
    }

    pub(crate) const fn working_format(&self) -> WorkingFormat {
        self.execution.working_format()
    }

    pub(crate) const fn output_format(&self) -> Format {
        self.execution.output_format()
    }

    pub(crate) fn output_extent(&self, account: &Account) -> Result<PhysicalSize> {
        let proof = ProofContext::new(account);
        let result = proof_resources(&self.lowered, &proof)
            .find(|resource| resource.id == self.lowered.root_working_image)
            .map(|resource| resource.spatial.device_extent)
            .ok_or_else(|| preparation_error("the base-graph root output resource is missing"));
        proof.finish()?;
        result
    }
}

#[must_use]
pub(crate) struct CompositionPreparableGraph {
    closed: ClosedExecutableGraph,
}

impl CompositionPreparableGraph {
    pub(super) fn try_from_closed(
        closed: ClosedExecutableGraph,
    ) -> std::result::Result<Self, ClosedExecutableGraph> {
        if !closed.has_layer_composition()
            || closed.has_color_filters()
            || closed.has_spatial_filters()
            || closed.has_backdrops()
        {
            return Err(closed);
        }
        Ok(Self { closed })
    }

    pub(super) fn closed(&self) -> &ClosedExecutableGraph {
        &self.closed
    }

    pub(super) fn into_closed(self) -> ClosedExecutableGraph {
        self.closed
    }

    pub(crate) fn working_format(&self) -> WorkingFormat {
        self.closed.facts.working_format
    }

    pub(crate) fn output_format(&self) -> Format {
        self.closed.facts.output_format
    }
}

#[must_use]
pub(crate) struct ColorFilterPreparableGraph {
    pub(super) closed: ClosedExecutableGraph,
}

impl ColorFilterPreparableGraph {
    pub(super) fn try_from_closed(
        closed: ClosedExecutableGraph,
    ) -> std::result::Result<Self, ClosedExecutableGraph> {
        if !closed.has_color_filters() || closed.has_spatial_filters() || closed.has_backdrops() {
            return Err(closed);
        }
        Ok(Self { closed })
    }

    pub(super) fn proves_closed_color_facts(&self, account: &Account) -> Result<bool> {
        Ok(self.closed.has_color_filters()
            && self
                .closed
                .facts
                .proves_exact_facts_for(&self.closed.lowered, account)?)
    }
}

#[must_use]
pub(crate) struct SpatialFilterPreparableGraph {
    pub(super) closed: ClosedExecutableGraph,
}

impl SpatialFilterPreparableGraph {
    fn try_from_closed(
        closed: ClosedExecutableGraph,
    ) -> std::result::Result<Self, ClosedExecutableGraph> {
        if !closed.has_spatial_filters() || closed.has_backdrops() {
            return Err(closed);
        }
        Ok(Self { closed })
    }

    pub(super) fn proves_closed_filter_facts(&self, account: &Account) -> Result<bool> {
        Ok(self.closed.has_spatial_filters()
            && self
                .closed
                .facts
                .proves_exact_facts_for(&self.closed.lowered, account)?)
    }

    pub(crate) fn working_format(&self) -> WorkingFormat {
        self.closed.facts.working_format
    }

    pub(crate) fn output_format(&self) -> Format {
        self.closed.facts.output_format
    }

    pub(crate) fn output_extent(&self, account: &Account) -> Result<PhysicalSize> {
        for resource in &self.closed.lowered.resources {
            account.work(RenderWork::ResourceProbes, 1)?;
            if resource.id == self.closed.lowered.root_working_image {
                return Ok(resource.spatial.device_extent);
            }
        }
        Err(preparation_error(
            "the spatial-filter root output resource is missing",
        ))
    }
}

/// Exact ordinary Gaussian pairs may execute with the closed layer composition.
#[must_use]
pub(crate) struct ForegroundBlurPreparableGraph(SpatialFilterPreparableGraph);

impl ForegroundBlurPreparableGraph {
    pub(super) fn qualify(
        graph: SpatialFilterPreparableGraph,
        account: &Account,
    ) -> Result<Option<Self>> {
        let facts = &graph.closed.facts;
        if facts.blurs.is_empty()
            || !facts.color_filters.is_empty()
            || !facts.drop_shadows.is_empty()
            || !facts.backdrops.is_empty()
        {
            return Ok(None);
        }
        for blur in &facts.blurs {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            if blur.blur.input != RuntimeBlurInput::Rgba
                || blur.blur.edge != RuntimeSamplingEdge::TransparentBlack
            {
                return Ok(None);
            }
        }
        Ok(Some(Self(graph)))
    }

    pub(crate) fn spatial_filter(&self) -> &SpatialFilterPreparableGraph {
        &self.0
    }
}

#[must_use]
pub(crate) struct BackdropPreparableGraph {
    pub(super) closed: ClosedExecutableGraph,
}

impl BackdropPreparableGraph {
    fn try_from_closed(
        closed: ClosedExecutableGraph,
        account: &Account,
    ) -> Result<std::result::Result<Self, ClosedExecutableGraph>> {
        let [backdrop] = closed.facts.backdrops.as_slice() else {
            return Ok(Err(closed));
        };
        if !filter_steps_match(&backdrop.filter_steps, &closed.facts.filter_steps, account)? {
            return Ok(Err(closed));
        }
        Ok(Ok(Self { closed }))
    }

    pub(super) fn proves_closed_backdrop_facts(&self, account: &Account) -> Result<bool> {
        let [backdrop] = self.closed.facts.backdrops.as_slice() else {
            return Ok(false);
        };
        Ok(filter_steps_match(
            &backdrop.filter_steps,
            &self.closed.facts.filter_steps,
            account,
        )? && self
            .closed
            .facts
            .proves_exact_facts_for(&self.closed.lowered, account)?)
    }

    pub(crate) fn working_format(&self) -> WorkingFormat {
        self.closed.facts.working_format
    }

    pub(crate) fn output_format(&self) -> Format {
        self.closed.facts.output_format
    }

    pub(crate) fn output_extent(&self, account: &Account) -> Result<PhysicalSize> {
        let proof = ProofContext::new(account);
        let result = proof_resources(&self.closed.lowered, &proof)
            .find(|resource| resource.id == self.closed.lowered.root_working_image)
            .map(|resource| resource.spatial.device_extent)
            .ok_or_else(|| preparation_error("the backdrop root output resource is missing"));
        proof.finish()?;
        result
    }
}
#[derive(Clone)]
pub(crate) struct ExecutableVelloCaptureFacts {
    pass: RuntimePassId,
    target: RuntimeResourceId,
    work: RuntimeVelloCapture,
    initial_transform: Transform,
    antialiasing: Antialiasing,
    target_extent: PhysicalSize,
    texel_origin: Point,
    raster_scale: f64,
}

impl ExecutableVelloCaptureFacts {
    #[must_use]
    pub(crate) const fn pass(&self) -> RuntimePassId {
        self.pass
    }

    #[must_use]
    pub(crate) const fn target(&self) -> RuntimeResourceId {
        self.target
    }

    #[must_use]
    pub(super) fn span(&self) -> Option<&RuntimeVelloSpan> {
        self.work.span()
    }

    #[must_use]
    pub(super) const fn work(&self) -> &RuntimeVelloCapture {
        &self.work
    }

    #[must_use]
    pub(crate) const fn initial_transform(&self) -> Transform {
        self.initial_transform
    }

    #[must_use]
    pub(crate) const fn antialiasing(&self) -> Antialiasing {
        self.antialiasing
    }

    #[must_use]
    pub(crate) const fn target_extent(&self) -> PhysicalSize {
        self.target_extent
    }

    #[must_use]
    pub(crate) const fn texel_origin(&self) -> Point {
        self.texel_origin
    }

    #[must_use]
    pub(crate) const fn raster_scale(&self) -> f64 {
        self.raster_scale
    }
}
struct ClosedGraphMaps<'plan, 'proof> {
    resource_by_id: ProofMap<'proof, RuntimeResourceId, &'plan RuntimeResourceRequest>,
    pass_positions: ProofMap<'proof, RuntimePassId, usize>,
}
impl<'plan, 'proof> ClosedGraphMaps<'plan, 'proof> {
    fn try_new(plan: &'plan LoweredGraphPlan, proof: &'proof ProofContext<'_>) -> Result<Self> {
        let mut resource_by_id = ProofMap::new(
            plan.resources.iter().map(|r| r.id),
            plan.resources.len(),
            proof,
        )?;
        for resource in &plan.resources {
            proof.account.work(RenderWork::GraphNodeVisits, 1)?;
            resource_by_id.insert(resource.id, resource);
            proof.finish()?;
        }
        let mut pass_positions =
            ProofMap::new(plan.passes.iter().map(|p| p.id), plan.passes.len(), proof)?;
        for (position, pass) in plan.passes.iter().enumerate() {
            proof.account.work(RenderWork::GraphNodeVisits, 1)?;
            pass_positions.insert(pass.id, position);
            proof.finish()?;
        }
        if resource_by_id.len() != plan.resources.len() || pass_positions.len() != plan.passes.len()
        {
            proof.invalid.set(true);
        }
        Ok(Self {
            resource_by_id,
            pass_positions,
        })
    }
}
struct ClosedGraphAccounting<'proof> {
    actual_reads: ProofMap<'proof, RuntimeResourceId, u32>,
    actual_last_reads: ProofMap<'proof, RuntimeResourceId, RuntimePassId>,
    releases: ProofMap<'proof, RuntimeResourceId, RuntimePassId>,
    results: ProofMap<'proof, RuntimeResourceId, RuntimePassId>,
}

fn validate_closed_graph_accounting(
    plan: &LoweredGraphPlan,
    maps: &ClosedGraphMaps<'_, '_>,
) -> Option<()> {
    let accounting = validate_closed_pass_accounting(plan, maps)?;
    for resource in &plan.resources {
        if !maps
            .resource_by_id
            .proof
            .work(RenderWork::GraphNodeVisits, 1)
        {
            return None;
        }
        if resource.format != runtime_resource_format(resource.role, plan.working_format)
            || resource.expected_reads == 0
            || accounting.actual_reads.get(&resource.id).copied() != Some(resource.expected_reads)
            || accounting.actual_last_reads.get(&resource.id).copied() != Some(resource.last_use)
            || accounting.releases.get(&resource.id).copied() != Some(resource.last_use)
            || resource.spatial.device_extent.width() == 0
            || resource.spatial.device_extent.height() == 0
            || !resource.spatial.texel_origin.x().is_finite()
            || !resource.spatial.texel_origin.y().is_finite()
            || !resource.spatial.raster_scale.is_finite()
            || resource.spatial.raster_scale <= 0.0
        {
            return None;
        }
        match (&resource.producer, &resource.import) {
            (
                RuntimeResourceProducer::Imported,
                Some(RuntimeResourceImport::ResolvedAlphaMask(_)),
            ) if resource.role == RuntimeResourceRole::ImportedImage
                && resource.format == RuntimeResourceFormat::ResolvedMaskRgba8Unorm => {}
            (RuntimeResourceProducer::Pass(pass), None)
                if accounting.results.get(&resource.id).copied() == Some(*pass) => {}
            _ => return None,
        }
    }
    Some(())
}

fn validate_closed_pass_accounting<'proof>(
    plan: &LoweredGraphPlan,
    maps: &ClosedGraphMaps<'_, 'proof>,
) -> Option<ClosedGraphAccounting<'proof>> {
    let proof = maps.resource_by_id.proof;
    let keys = || plan.resources.iter().map(|r| r.id);
    let mut accounting = ClosedGraphAccounting {
        actual_reads: proof.capture(ProofMap::new(keys(), plan.resources.len(), proof))?,
        actual_last_reads: proof.capture(ProofMap::new(keys(), plan.resources.len(), proof))?,
        releases: proof.capture(ProofMap::new(keys(), plan.resources.len(), proof))?,
        results: proof.capture(ProofMap::new(keys(), plan.resources.len(), proof))?,
    };
    let mut dependencies = proof.capture(ProofSet::new(
        plan.passes.iter().map(|p| p.id),
        plan.passes.len(),
        proof,
    ))?;
    let mut pass_reads = proof.capture(ProofSet::new(keys(), plan.resources.len(), proof))?;
    let mut pass_releases = proof.capture(ProofSet::new(keys(), plan.resources.len(), proof))?;
    for (position, pass) in plan.passes.iter().enumerate() {
        if !proof.work(RenderWork::GraphNodeVisits, 1) {
            return None;
        }
        dependencies.clear()?;
        if pass.dependencies.iter().any(|dependency| {
            !proof.work(RenderWork::GraphEdgeVisits, 1)
                || !dependencies.insert(*dependency)
                || maps
                    .pass_positions
                    .get(dependency)
                    .is_none_or(|dependency_position| *dependency_position >= position)
        }) {
            return None;
        }
        pass_reads.clear()?;
        for read in &pass.reads {
            if !proof.work(RenderWork::GraphEdgeVisits, 1) {
                return None;
            }
            if !pass_reads.insert(read.resource)
                || !runtime_read_sampler_is_exact(read, &maps.resource_by_id)
            {
                return None;
            }
            validate_closed_read(plan, maps, &mut accounting, pass, read, position)?;
        }
        validate_closed_result(plan, maps, &mut accounting, pass)?;
        pass_releases.clear()?;
        if pass.releases.iter().any(|resource| {
            !proof.work(RenderWork::GraphEdgeVisits, 1)
                || !pass_releases.insert(*resource)
                || !pass_reads.contains(resource)
                || accounting.releases.insert(*resource, pass.id).is_some()
        }) {
            return None;
        }
        let expected_cache_keys = proof.capture(runtime_pass_cache_keys(
            &pass.kind,
            &pass.reads,
            pass.result,
            plan.working_format,
            plan.output_format,
            |id| Ok(maps.resource_by_id.get(&id).map(|resource| resource.format)),
            proof.account,
        ))?;
        let keys_match = match (&expected_cache_keys, &pass.cache_keys) {
            (Some(expected), Some(actual)) => {
                proof.capture(expected.matches_in(actual, proof.account))?
            }
            (None, None) => true,
            _ => false,
        };
        if !keys_match {
            return None;
        }
    }
    Some(accounting)
}

fn validate_closed_read(
    _plan: &LoweredGraphPlan,
    maps: &ClosedGraphMaps<'_, '_>,
    accounting: &mut ClosedGraphAccounting<'_>,
    pass: &RuntimePass,
    read: &RuntimeReadBinding,
    position: usize,
) -> Option<()> {
    let resource = maps.resource_by_id.get(&read.resource).copied()?;
    if pass.result == RuntimeResultBinding::Resource(read.resource) {
        return None;
    }
    if let RuntimeResourceProducer::Pass(producer) = resource.producer
        && (maps
            .pass_positions
            .get(&producer)
            .is_none_or(|producer_position| *producer_position >= position)
            || !proof_contains_edge(&pass.dependencies, &producer, maps.resource_by_id.proof))
    {
        return None;
    }
    let reads = accounting
        .actual_reads
        .get_or_insert_default(read.resource)?;
    *reads = reads.checked_add(1)?;
    accounting.actual_last_reads.insert(read.resource, pass.id);
    Some(())
}

fn validate_closed_result(
    plan: &LoweredGraphPlan,
    maps: &ClosedGraphMaps<'_, '_>,
    accounting: &mut ClosedGraphAccounting<'_>,
    pass: &RuntimePass,
) -> Option<()> {
    match pass.result {
        RuntimeResultBinding::Resource(resource) => {
            let request = maps.resource_by_id.get(&resource).copied()?;
            if request.producer != RuntimeResourceProducer::Pass(pass.id)
                || accounting.results.insert(resource, pass.id).is_some()
            {
                return None;
            }
        }
        RuntimeResultBinding::Output(format) => {
            if !matches!(pass.kind, RuntimePassKind::Present) || format != plan.output_format {
                return None;
            }
        }
        RuntimeResultBinding::Empty => {}
    }
    Some(())
}

fn closed_graph_root<'plan>(
    plan: &'plan LoweredGraphPlan,
    resources: &ProofMap<'_, RuntimeResourceId, &'plan RuntimeResourceRequest>,
) -> Option<(&'plan RuntimePass, &'plan RuntimeResourceRequest)> {
    let clear = plan.passes.first()?;
    if !resources.proof.work(RenderWork::GraphNodeVisits, 1) {
        return None;
    }
    let RuntimePassKind::ClearRoot {
        initialization: RuntimeInitialization::SurfaceBaseColor,
        ..
    } = clear.kind
    else {
        return None;
    };
    if !clear.dependencies.is_empty()
        || !clear.reads.is_empty()
        || !clear.releases.is_empty()
        || clear.cache_keys.is_some()
        || clear.result != RuntimeResultBinding::Resource(plan.root_working_image)
    {
        return None;
    }
    let root = resources.get(&plan.root_working_image).copied()?;
    base_graph_resource_has_fixed_facts(
        root,
        RuntimeResourceRole::RootWorkingImage,
        RuntimeResourceFormat::Working(plan.working_format),
        RuntimeResourceProducer::Pass(clear.id),
    )
    .then_some((clear, root))
}

#[derive(Default)]
struct ProofBounds {
    contexts: usize,
    captures: usize,
    layers: usize,
    colors: usize,
    blurs: usize,
    shadows: usize,
    steps: usize,
    backdrops: usize,
}
impl ProofBounds {
    fn for_plan(plan: &LoweredGraphPlan, account: &Account) -> Result<Self> {
        let mut bounds = Self {
            contexts: 1,
            ..Self::default()
        };
        for pass in &plan.passes {
            account.work(RenderWork::GraphNodeVisits, 1)?;
            match &pass.kind {
                RuntimePassKind::ClearRoot {
                    initialization: RuntimeInitialization::Transparent,
                    ..
                } => bounds.contexts += 1,
                RuntimePassKind::VelloCapture(Some(_)) => bounds.captures += 1,
                RuntimePassKind::Composite(Some(RuntimeComposite {
                    kind: RuntimeCompositeKind::Layer { .. },
                    ..
                })) => bounds.layers += 1,
                RuntimePassKind::ColorFilter(Some(_)) => {
                    bounds.colors += 1;
                    bounds.steps += 1;
                }
                RuntimePassKind::BlurHorizontal(Some(blur)) => {
                    bounds.steps += 1;
                    if blur.input == RuntimeBlurInput::Rgba {
                        bounds.blurs += 1;
                    } else {
                        bounds.shadows += 1;
                    }
                }
                RuntimePassKind::CopyBackdrop => bounds.backdrops += 1,
                _ => {}
            }
        }
        Ok(bounds)
    }
}

struct ClosedGraphTraversal<'plan, 'proof> {
    plan: &'plan LoweredGraphPlan,
    proof: &'proof ProofContext<'proof>,
    resources: ProofMap<'proof, RuntimeResourceId, &'plan RuntimeResourceRequest>,
    contexts: ProofVec<ExecutableCompositionContext>,
    captures: ProofVec<ExecutableVelloCaptureFacts>,
    layer_compositions: ProofVec<ExecutableLayerCompositionFacts>,
    color_filters: ProofVec<ExecutableColorFilterFacts>,
    blurs: ProofVec<ExecutableBlurFacts>,
    drop_shadows: ProofVec<ExecutableDropShadowFacts>,
    filter_steps: ProofVec<ExecutableFilterStepFacts>,
    backdrops: ProofVec<ExecutableBackdropFacts>,
    expected_resources: ProofSet<'proof, RuntimeResourceId>,
    cursor: usize,
}

impl<'plan, 'proof> ClosedGraphTraversal<'plan, 'proof> {
    fn new(
        plan: &'plan LoweredGraphPlan,
        resources: ProofMap<'proof, RuntimeResourceId, &'plan RuntimeResourceRequest>,
        clear: &RuntimePass,
        root: &RuntimeResourceRequest,
        proof: &'proof ProofContext<'proof>,
    ) -> Result<Self> {
        let bounds = ProofBounds::for_plan(plan, proof.account)?;
        let mut contexts = ProofVec::new(
            proof.account,
            bounds.contexts,
            RenderResource::ScratchCpuBytes,
        )?;
        contexts.push(ExecutableCompositionContext {
            current: root.id,
            producer: clear.id,
            contains_captured_source: false,
        })?;
        let mut expected_resources = ProofSet::new(
            plan.resources.iter().map(|r| r.id),
            plan.resources.len(),
            proof,
        )?;
        expected_resources.insert(root.id);
        Ok(Self {
            plan,
            proof,
            resources,
            contexts,
            captures: ProofVec::new(
                proof.account,
                bounds.captures,
                RenderResource::RetainedCpuBytes,
            )?,
            layer_compositions: ProofVec::new(
                proof.account,
                bounds.layers,
                RenderResource::RetainedCpuBytes,
            )?,
            color_filters: ProofVec::new(
                proof.account,
                bounds.colors,
                RenderResource::RetainedCpuBytes,
            )?,
            blurs: ProofVec::new(
                proof.account,
                bounds.blurs,
                RenderResource::RetainedCpuBytes,
            )?,
            drop_shadows: ProofVec::new(
                proof.account,
                bounds.shadows,
                RenderResource::RetainedCpuBytes,
            )?,
            filter_steps: ProofVec::new(
                proof.account,
                bounds.steps,
                RenderResource::RetainedCpuBytes,
            )?,
            backdrops: ProofVec::new(
                proof.account,
                bounds.backdrops,
                RenderResource::RetainedCpuBytes,
            )?,
            expected_resources,
            cursor: 1,
        })
    }

    fn run(mut self) -> Result<Option<ClosedExecutableGraphFacts>> {
        let valid = (|| -> Option<()> {
            while self.cursor < self.plan.passes.len() {
                self.visit_current_pass()?;
            }
            let clip_coverages_are_exact =
                proof_nodes(&self.layer_compositions, self.proof).all(|layer| {
                    layer_has_exact_clip_coverage_capture(layer, &self.captures, self.proof)
                }) && proof_nodes(&self.captures, self.proof).all(|capture| {
                    capture.work().clip_coverage().is_none()
                        || proof_nodes(&self.layer_compositions, self.proof)
                            .any(|layer| layer.clip_coverage == Some(capture.target()))
                });
            if (self.captures.is_empty() && self.layer_compositions.is_empty())
                || self.contexts.len() != 1
                || !clip_coverages_are_exact
                || self.expected_resources.len() != self.plan.resources.len()
                || self
                    .expected_resources
                    .iter()
                    .any(|resource| !self.resources.contains_key(resource))
            {
                return None;
            }
            Some(())
        })()
        .is_some();
        self.proof.finish()?;
        if !valid || self.proof.invalid.get() {
            return Ok(None);
        }
        Ok(Some(ClosedExecutableGraphFacts {
            working_format: self.plan.working_format,
            output_format: self.plan.output_format,
            captures: self.captures.into_shared(self.proof.account)?,
            layer_compositions: self.layer_compositions.into_shared(self.proof.account)?,
            color_filters: self.color_filters.into_shared(self.proof.account)?,
            blurs: self.blurs.into_shared(self.proof.account)?,
            drop_shadows: self.drop_shadows.into_shared(self.proof.account)?,
            filter_steps: self.filter_steps.into_shared(self.proof.account)?,
            backdrops: self.backdrops.into_shared(self.proof.account)?,
        }))
    }

    fn pass_at(&self, index: usize) -> Option<&'plan RuntimePass> {
        let pass = self.plan.passes.get(index)?;
        self.proof
            .work(RenderWork::GraphNodeVisits, 1)
            .then_some(pass)
    }

    fn visit_current_pass(&mut self) -> Option<()> {
        let pass = self.pass_at(self.cursor)?;
        match &pass.kind {
            RuntimePassKind::ClearRoot {
                initialization: RuntimeInitialization::Transparent,
                color,
            } => self.visit_transparent_clear(pass, *color),
            RuntimePassKind::VelloCapture(Some(work)) if work.span().is_some() => {
                self.visit_span_capture(pass, work)
            }
            RuntimePassKind::VelloCapture(Some(work)) if work.clip_coverage().is_some() => {
                self.visit_clip_coverage_capture(pass)
            }
            RuntimePassKind::ColorFilter(Some(filter)) => self.visit_color_filter(pass, filter),
            RuntimePassKind::BlurHorizontal(Some(blur))
                if blur.axis == RuntimeBlurAxis::Horizontal
                    && blur.input == RuntimeBlurInput::Rgba =>
            {
                self.visit_blur(pass, blur)
            }
            RuntimePassKind::BlurHorizontal(Some(blur))
                if blur.axis == RuntimeBlurAxis::Horizontal
                    && blur.input == RuntimeBlurInput::SourceAlpha =>
            {
                self.visit_drop_shadow(pass, blur)
            }
            RuntimePassKind::CopyBackdrop => self.visit_backdrop(pass),
            RuntimePassKind::Composite(Some(composite))
                if matches!(composite.kind, RuntimeCompositeKind::Layer { .. }) =>
            {
                self.visit_layer_composite(pass)
            }
            RuntimePassKind::Present => self.visit_present(pass),
            RuntimePassKind::ClearRoot {
                initialization: RuntimeInitialization::SurfaceBaseColor,
                ..
            }
            | RuntimePassKind::VelloCapture(None)
            | RuntimePassKind::VelloCapture(Some(_))
            | RuntimePassKind::CanonicalizeCapture
            | RuntimePassKind::ColorFilter(_)
            | RuntimePassKind::BlurHorizontal(_)
            | RuntimePassKind::BlurVertical(_)
            | RuntimePassKind::DropShadowColorize(_)
            | RuntimePassKind::Composite(_) => None,
        }
    }

    fn visit_transparent_clear(&mut self, pass: &RuntimePass, color: Color) -> Option<()> {
        if color != Color::TRANSPARENT
            || !pass.dependencies.is_empty()
            || !pass.reads.is_empty()
            || !pass.releases.is_empty()
            || pass.cache_keys.is_some()
        {
            return None;
        }
        let RuntimeResultBinding::Resource(resource) = pass.result else {
            return None;
        };
        let request = self.resources.get(&resource).copied()?;
        if !base_graph_resource_has_fixed_facts(
            request,
            RuntimeResourceRole::IsolationWorkingImage,
            RuntimeResourceFormat::Working(self.plan.working_format),
            RuntimeResourceProducer::Pass(pass.id),
        ) {
            return None;
        }
        self.expected_resources.insert(resource);
        self.proof
            .capture_storage_write(self.contexts.push(ExecutableCompositionContext {
                current: resource,
                producer: pass.id,
                contains_captured_source: false,
            }))?;
        self.advance(1)
    }

    fn initialized_context_source(
        &self,
        context: ExecutableCompositionContext,
    ) -> Option<&'plan RuntimeResourceRequest> {
        let source = self.resources.get(&context.current).copied()?;
        // The traversal creates an uncaptured source context only after proving
        // a transparent isolation clear. Bind it to that exact resource and
        // producer, rather than treating absence of source ink as absence of work.
        (context.contains_captured_source
            || base_graph_resource_has_fixed_facts(
                source,
                RuntimeResourceRole::IsolationWorkingImage,
                RuntimeResourceFormat::Working(self.plan.working_format),
                RuntimeResourceProducer::Pass(context.producer),
            ))
        .then_some(source)
    }

    fn visit_span_capture(&mut self, pass: &RuntimePass, work: &RuntimeVelloCapture) -> Option<()> {
        let span = work.span()?;
        let canonicalize = self.pass_at(self.cursor.checked_add(1)?)?;
        let after_canonicalize = self.pass_at(self.cursor.checked_add(2)?)?;
        let (coverage_pass, composite, pass_count) = if matches!(
            after_canonicalize.kind,
            RuntimePassKind::VelloCapture(Some(RuntimeVelloCapture::ClipCoverage(_)))
        ) {
            (
                Some(after_canonicalize),
                self.pass_at(self.cursor.checked_add(3)?)?,
                4,
            )
        } else {
            (None, after_canonicalize, 3)
        };
        let (capture_target, capture_resource) =
            self.validate_span_capture_source(pass, canonicalize, span)?;
        let (canonical_target, canonical_resource) = self.validate_canonical_capture(
            pass,
            canonicalize,
            composite,
            capture_target,
            capture_resource,
        )?;
        let capture_facts = self.proof.capture(executable_vello_capture_facts(
            pass.id,
            capture_target,
            work,
            capture_resource.spatial,
            self.proof.account,
        ))??;
        let coverage_facts = match coverage_pass {
            Some(coverage) => Some(validate_closed_clip_coverage_capture(
                coverage,
                composite.id,
                &self.resources,
            )?),
            None => None,
        };
        let parent = *self.contexts.last()?;
        let layer = validate_closed_composite(
            composite,
            parent,
            canonical_resource,
            &self.resources,
            self.plan.working_format,
            false,
        )?;
        let RuntimeResultBinding::Resource(result) = composite.result else {
            return None;
        };
        self.record_span_capture(
            composite,
            [capture_target, canonical_target, result],
            capture_facts,
            coverage_facts,
            layer,
        )?;
        self.advance(pass_count)
    }

    fn validate_span_capture_source(
        &self,
        pass: &RuntimePass,
        canonicalize: &RuntimePass,
        span: &RuntimeVelloSpan,
    ) -> Option<(RuntimeResourceId, &'plan RuntimeResourceRequest)> {
        let RuntimeResultBinding::Resource(capture_target) = pass.result else {
            return None;
        };
        let expected_scope = if self.contexts.len() == 1 {
            RuntimeVelloSpanScope::CurrentParent
        } else {
            RuntimeVelloSpanScope::LayerSource
        };
        if !pass.dependencies.is_empty()
            || !pass.reads.is_empty()
            || !pass.releases.is_empty()
            || pass.cache_keys.is_some()
            || span.scope != expected_scope
        {
            return None;
        }
        let resource = self.resources.get(&capture_target).copied()?;
        if span.source_space == crate::frame::CompositeSourceSpace::SurfaceGrid
            && resource.spatial.raster_scale
                != self
                    .resources
                    .get(&self.plan.root_working_image)?
                    .spatial
                    .raster_scale
        {
            return None;
        }
        base_graph_resource_has_fixed_facts(
            resource,
            RuntimeResourceRole::CaptureWorkingImage,
            RuntimeResourceFormat::VelloCaptureRgba8Unorm,
            RuntimeResourceProducer::Pass(pass.id),
        )
        .then_some(())
        .filter(|()| resource.expected_reads == 1 && resource.last_use == canonicalize.id)?;
        Some((capture_target, resource))
    }

    fn validate_canonical_capture(
        &self,
        capture: &RuntimePass,
        canonicalize: &RuntimePass,
        composite: &RuntimePass,
        capture_target: RuntimeResourceId,
        capture_resource: &RuntimeResourceRequest,
    ) -> Option<(RuntimeResourceId, &'plan RuntimeResourceRequest)> {
        if !matches!(canonicalize.kind, RuntimePassKind::CanonicalizeCapture)
            || !proof_edges_equal(&canonicalize.dependencies, &[capture.id], self.proof)
            || canonicalize.reads.len() != 1
            || !runtime_read_has_exact_facts(
                &canonicalize.reads[0],
                RuntimeReadRole::CaptureSource,
                capture_resource,
                RuntimeSamplingFilter::Linear,
                RuntimeSamplingEdge::ClampToExtent,
                self.proof,
            )
            || !proof_edges_equal(&canonicalize.releases, &[capture_target], self.proof)
        {
            return None;
        }
        let RuntimeResultBinding::Resource(canonical_target) = canonicalize.result else {
            return None;
        };
        let resource = self.resources.get(&canonical_target).copied()?;
        base_graph_resource_has_fixed_facts(
            resource,
            RuntimeResourceRole::FilterIntermediate,
            RuntimeResourceFormat::Working(self.plan.working_format),
            RuntimeResourceProducer::Pass(canonicalize.id),
        )
        .then_some(())
        .filter(|()| {
            resource.expected_reads == 1
                && resource.last_use == composite.id
                && resource.spatial == capture_resource.spatial
        })?;
        Some((canonical_target, resource))
    }

    fn record_span_capture(
        &mut self,
        composite: &RuntimePass,
        resources: [RuntimeResourceId; 3],
        capture: ExecutableVelloCaptureFacts,
        coverage: Option<(RuntimeResourceId, ExecutableVelloCaptureFacts)>,
        layer: Option<ExecutableLayerCompositionFacts>,
    ) -> Option<()> {
        let context = self.contexts.last_mut()?;
        context.current = resources[2];
        context.producer = composite.id;
        context.contains_captured_source = true;
        self.expected_resources.extend(resources);
        self.proof
            .capture_storage_write(self.captures.push(capture))?;
        if let Some((resource, facts)) = coverage {
            self.expected_resources.insert(resource);
            self.proof
                .capture_storage_write(self.captures.push(facts))?;
        }
        if let Some(layer) = layer {
            self.record_layer_resources(&layer);
            self.proof
                .capture_storage_write(self.layer_compositions.push(layer))?;
        }
        Some(())
    }

    fn visit_clip_coverage_capture(&mut self, pass: &RuntimePass) -> Option<()> {
        let composite = self.pass_at(self.cursor.checked_add(1)?)?;
        let (coverage, facts) =
            validate_closed_clip_coverage_capture(pass, composite.id, &self.resources)?;
        self.expected_resources.insert(coverage);
        self.proof
            .capture_storage_write(self.captures.push(facts))?;
        self.advance(1)
    }

    fn visit_color_filter(
        &mut self,
        pass: &RuntimePass,
        filter: &RuntimeColorFilter,
    ) -> Option<()> {
        let context = *self.contexts.last()?;
        if !context.contains_captured_source {
            return None;
        }
        let source = self.resources.get(&context.current).copied()?;
        let color = validate_closed_color_filter(
            pass,
            context,
            source,
            filter,
            &self.resources,
            self.plan.working_format,
        )?;
        let runtime_context = self.contexts.last_mut()?;
        runtime_context.current = color.result;
        runtime_context.producer = pass.id;
        self.expected_resources.insert(color.result);
        self.proof.capture_storage_write(
            self.filter_steps
                .push(ExecutableFilterStepFacts::Color(pass.id)),
        )?;
        self.proof
            .capture_storage_write(self.color_filters.push(color))?;
        self.advance(1)
    }

    fn visit_blur(&mut self, horizontal: &RuntimePass, blur: &RuntimeBlur) -> Option<()> {
        self.visit_blur_with_edge(horizontal, blur, RuntimeSamplingEdge::TransparentBlack)
    }

    fn visit_blur_with_edge(
        &mut self,
        horizontal: &RuntimePass,
        blur: &RuntimeBlur,
        edge: RuntimeSamplingEdge,
    ) -> Option<()> {
        let vertical = self.pass_at(self.cursor.checked_add(1)?)?;
        let context = *self.contexts.last()?;
        if !context.contains_captured_source {
            return None;
        }
        let source = self.resources.get(&context.current).copied()?;
        let validation = ClosedFilterValidation {
            context,
            source,
            resources: &self.resources,
            working_format: self.plan.working_format,
            edge,
        };
        let facts = validate_closed_blur(horizontal, vertical, blur, validation)?;
        let runtime_context = self.contexts.last_mut()?;
        runtime_context.current = facts.result;
        runtime_context.producer = facts.vertical;
        self.expected_resources
            .extend([facts.intermediate, facts.result]);
        self.proof.capture_storage_write(self.filter_steps.push(
            ExecutableFilterStepFacts::Blur {
                horizontal: facts.horizontal,
                vertical: facts.vertical,
            },
        ))?;
        self.proof.capture_storage_write(self.blurs.push(facts))?;
        self.advance(2)
    }

    fn visit_drop_shadow(&mut self, horizontal: &RuntimePass, blur: &RuntimeBlur) -> Option<()> {
        self.visit_drop_shadow_with_edge(horizontal, blur, RuntimeSamplingEdge::TransparentBlack)
    }

    fn visit_drop_shadow_with_edge(
        &mut self,
        horizontal: &RuntimePass,
        blur: &RuntimeBlur,
        edge: RuntimeSamplingEdge,
    ) -> Option<()> {
        let vertical = self.pass_at(self.cursor.checked_add(1)?)?;
        let colorize = self.pass_at(self.cursor.checked_add(2)?)?;
        let merge = self.pass_at(self.cursor.checked_add(3)?)?;
        let context = *self.contexts.last()?;
        if !context.contains_captured_source {
            return None;
        }
        let source = self.resources.get(&context.current).copied()?;
        let validation = ClosedFilterValidation {
            context,
            source,
            resources: &self.resources,
            working_format: self.plan.working_format,
            edge,
        };
        let facts =
            validate_closed_drop_shadow([horizontal, vertical, colorize, merge], blur, validation)?;
        let runtime_context = self.contexts.last_mut()?;
        runtime_context.current = facts.result;
        runtime_context.producer = facts.merge;
        self.expected_resources.extend([
            facts.horizontal_result,
            facts.vertical_result,
            facts.shadow,
            facts.result,
        ]);
        self.proof.capture_storage_write(self.filter_steps.push(
            ExecutableFilterStepFacts::DropShadow {
                horizontal: facts.horizontal,
                vertical: facts.vertical,
                colorize: facts.colorize,
                merge: facts.merge,
            },
        ))?;
        self.proof
            .capture_storage_write(self.drop_shadows.push(facts))?;
        self.advance(4)
    }

    fn visit_backdrop(&mut self, copy: &RuntimePass) -> Option<()> {
        let foreground = match self.contexts.len() {
            1 => None,
            2 => Some(self.contexts.pop()?),
            _ => return None,
        };
        if let Some(context) = foreground {
            self.initialized_context_source(context)?;
        }
        let parent = *self.contexts.first()?;
        let copied = self.validate_backdrop_copy(copy, parent)?;
        let filter_start = self.filter_steps.len();
        self.proof
            .capture_storage_write(self.contexts.push(ExecutableCompositionContext {
                current: copied,
                producer: copy.id,
                contains_captured_source: true,
            }))?;
        self.advance(1)?;
        self.visit_backdrop_filters()?;
        let filtered = self.contexts.pop()?;
        let filter_steps = self
            .proof
            .shared_copy(self.filter_steps.get(filter_start..)?)?;
        let group_clear = self.pass_at(self.cursor)?;
        self.visit_transparent_clear(group_clear, Color::TRANSPARENT)?;
        let backdrop_composite = self.visit_backdrop_source_composite(filtered)?;
        let foreground_composite = match foreground {
            Some(foreground) => Some(self.visit_backdrop_foreground(foreground)?),
            None => None,
        };
        let completed_group = *self.contexts.last()?;
        let outer_composite = self.visit_backdrop_outer_composite(parent, completed_group)?;
        let result = self.contexts.first()?.current;
        self.proof
            .capture_storage_write(self.backdrops.push(ExecutableBackdropFacts {
                copy: copy.id,
                completed_parent: parent.current,
                copied,
                foreground: foreground.map(|context| context.current),
                filter_steps,
                filtered: filtered.current,
                group_clear: group_clear.id,
                backdrop_composite,
                foreground_composite,
                outer_composite,
                completed_group: completed_group.current,
                result,
            }))?;
        Some(())
    }

    fn validate_backdrop_copy(
        &mut self,
        copy: &RuntimePass,
        parent: ExecutableCompositionContext,
    ) -> Option<RuntimeResourceId> {
        let parent_resource = self.resources.get(&parent.current).copied()?;
        if !proof_edges_equal(&copy.dependencies, &[parent.producer], self.proof)
            || copy.reads.len() != 1
            || !runtime_read_has_exact_facts(
                &copy.reads[0],
                RuntimeReadRole::CompletedParent,
                parent_resource,
                RuntimeSamplingFilter::Nearest,
                RuntimeSamplingEdge::TransparentBlack,
                self.proof,
            )
            || !copy.releases.is_empty()
            || copy.cache_keys.is_none()
        {
            return None;
        }
        let RuntimeResultBinding::Resource(copied) = copy.result else {
            return None;
        };
        let resource = self.resources.get(&copied).copied()?;
        if !base_graph_resource_has_fixed_facts(
            resource,
            RuntimeResourceRole::BackdropCopy,
            RuntimeResourceFormat::Working(self.plan.working_format),
            RuntimeResourceProducer::Pass(copy.id),
        ) || resource.expected_reads != 1
        {
            return None;
        }
        self.expected_resources.insert(copied);
        Some(copied)
    }

    fn visit_backdrop_filters(&mut self) -> Option<()> {
        loop {
            let pass = self.pass_at(self.cursor)?;
            match &pass.kind {
                RuntimePassKind::ColorFilter(Some(filter)) => {
                    self.visit_color_filter(pass, filter)?;
                }
                RuntimePassKind::BlurHorizontal(Some(blur))
                    if blur.axis == RuntimeBlurAxis::Horizontal
                        && blur.input == RuntimeBlurInput::Rgba =>
                {
                    self.visit_backdrop_blur(pass, blur)?;
                }
                RuntimePassKind::BlurHorizontal(Some(blur))
                    if blur.axis == RuntimeBlurAxis::Horizontal
                        && blur.input == RuntimeBlurInput::SourceAlpha =>
                {
                    self.visit_backdrop_drop_shadow(pass, blur)?;
                }
                RuntimePassKind::ClearRoot {
                    initialization: RuntimeInitialization::Transparent,
                    color,
                } if *color == Color::TRANSPARENT => return Some(()),
                _ => return None,
            }
        }
    }

    fn visit_backdrop_blur(&mut self, horizontal: &RuntimePass, blur: &RuntimeBlur) -> Option<()> {
        let RuntimeSamplingEdge::SemanticBorderMirror(_) = blur.edge else {
            return None;
        };
        self.visit_blur_with_edge(horizontal, blur, blur.edge)
    }

    fn visit_backdrop_drop_shadow(
        &mut self,
        horizontal: &RuntimePass,
        blur: &RuntimeBlur,
    ) -> Option<()> {
        let RuntimeSamplingEdge::SemanticBorderMirror(_) = blur.edge else {
            return None;
        };
        self.visit_drop_shadow_with_edge(horizontal, blur, blur.edge)
    }

    fn visit_backdrop_source_composite(
        &mut self,
        filtered: ExecutableCompositionContext,
    ) -> Option<RuntimePassId> {
        let pass = self.next_layer_composite_with_optional_coverage()?;
        let parent = *self.contexts.last()?;
        let source = self.resources.get(&filtered.current).copied()?;
        let layer = validate_closed_composite(
            pass,
            parent,
            source,
            &self.resources,
            self.plan.working_format,
            true,
        )??;
        if !runtime_composite_is_backdrop_inner(&layer.composite) {
            return None;
        }
        self.record_backdrop_composite_result(pass, layer)?;
        Some(pass.id)
    }

    fn visit_backdrop_foreground(
        &mut self,
        foreground: ExecutableCompositionContext,
    ) -> Option<RuntimePassId> {
        let pass = self.pass_at(self.cursor)?;
        let parent = *self.contexts.last()?;
        let source = self.resources.get(&foreground.current).copied()?;
        if validate_closed_composite(
            pass,
            parent,
            source,
            &self.resources,
            self.plan.working_format,
            false,
        )?
        .is_some()
        {
            return None;
        }
        self.record_backdrop_span_result(pass)?;
        Some(pass.id)
    }

    fn visit_backdrop_outer_composite(
        &mut self,
        parent: ExecutableCompositionContext,
        group: ExecutableCompositionContext,
    ) -> Option<RuntimePassId> {
        let pass = self.next_layer_composite_with_optional_coverage()?;
        let source = self.resources.get(&group.current).copied()?;
        let layer = validate_closed_composite_with_parent_reads(
            pass,
            parent,
            source,
            &self.resources,
            self.plan.working_format,
            true,
            2,
        )??;
        if !runtime_composite_is_untransformed_outer(&layer.composite) {
            return None;
        }
        let RuntimeResultBinding::Resource(result) = pass.result else {
            return None;
        };
        self.contexts.pop()?;
        let root = self.contexts.first_mut()?;
        root.current = result;
        root.producer = pass.id;
        root.contains_captured_source = true;
        self.expected_resources.insert(result);
        self.record_layer_resources(&layer);
        self.proof
            .capture_storage_write(self.layer_compositions.push(layer))?;
        self.advance(1)?;
        Some(pass.id)
    }

    fn next_layer_composite_with_optional_coverage(&mut self) -> Option<&'plan RuntimePass> {
        let pass = self.pass_at(self.cursor)?;
        if matches!(
            pass.kind,
            RuntimePassKind::VelloCapture(Some(RuntimeVelloCapture::ClipCoverage(_)))
        ) {
            self.visit_clip_coverage_capture(pass)?;
        }
        let composite = self.pass_at(self.cursor)?;
        matches!(
            composite.kind,
            RuntimePassKind::Composite(Some(RuntimeComposite {
                kind: RuntimeCompositeKind::Layer { .. },
                ..
            }))
        )
        .then_some(composite)
    }

    fn record_backdrop_composite_result(
        &mut self,
        pass: &RuntimePass,
        layer: ExecutableLayerCompositionFacts,
    ) -> Option<()> {
        let RuntimeResultBinding::Resource(result) = pass.result else {
            return None;
        };
        let context = self.contexts.last_mut()?;
        context.current = result;
        context.producer = pass.id;
        context.contains_captured_source = true;
        self.expected_resources.insert(result);
        self.record_layer_resources(&layer);
        self.proof
            .capture_storage_write(self.layer_compositions.push(layer))?;
        self.advance(1)
    }

    fn record_backdrop_span_result(&mut self, pass: &RuntimePass) -> Option<()> {
        let RuntimeResultBinding::Resource(result) = pass.result else {
            return None;
        };
        let context = self.contexts.last_mut()?;
        context.current = result;
        context.producer = pass.id;
        context.contains_captured_source = true;
        self.expected_resources.insert(result);
        self.advance(1)
    }

    fn visit_layer_composite(&mut self, pass: &RuntimePass) -> Option<()> {
        if self.contexts.len() < 2 {
            return None;
        }
        let source_context = self.contexts.pop()?;
        let parent = *self.contexts.last()?;
        let source = self.initialized_context_source(source_context)?;
        let layer = validate_closed_composite(
            pass,
            parent,
            source,
            &self.resources,
            self.plan.working_format,
            true,
        )??;
        let RuntimeResultBinding::Resource(result) = pass.result else {
            return None;
        };
        let context = self.contexts.last_mut()?;
        context.current = result;
        context.producer = pass.id;
        context.contains_captured_source = true;
        self.expected_resources.insert(result);
        self.record_layer_resources(&layer);
        self.proof
            .capture_storage_write(self.layer_compositions.push(layer))?;
        self.advance(1)
    }

    fn visit_present(&mut self, pass: &RuntimePass) -> Option<()> {
        if self.cursor.checked_add(1)? != self.plan.passes.len()
            || pass.id != self.plan.final_present
            || self.contexts.len() != 1
        {
            return None;
        }
        let parent = self.contexts[0];
        let resource = self.resources.get(&parent.current).copied()?;
        if !proof_edges_equal(&pass.dependencies, &[parent.producer], self.proof)
            || pass.reads.len() != 1
            || !runtime_read_has_exact_facts(
                &pass.reads[0],
                RuntimeReadRole::FinalWorkingImage,
                resource,
                RuntimeSamplingFilter::Linear,
                RuntimeSamplingEdge::ClampToExtent,
                self.proof,
            )
            || pass.result != RuntimeResultBinding::Output(self.plan.output_format)
            || !proof_edges_equal(&pass.releases, &[parent.current], self.proof)
            || resource.expected_reads != 1
            || resource.last_use != pass.id
        {
            return None;
        }
        self.advance(1)
    }

    fn record_layer_resources(&mut self, layer: &ExecutableLayerCompositionFacts) {
        if let Some(coverage) = layer.clip_coverage {
            self.expected_resources.insert(coverage);
        }
        if let Some(mask) = layer.alpha_mask {
            self.expected_resources.insert(mask);
        }
    }

    fn advance(&mut self, count: usize) -> Option<()> {
        self.cursor = self.cursor.checked_add(count)?;
        Some(())
    }
}
impl LoweredGraphPlan {
    fn closed_executable_graph_facts(
        &self,
        account: &Account,
    ) -> Result<Option<ClosedExecutableGraphFacts>> {
        // Root clear, finite empty-group clear, composition and presentation
        // form the smallest executable graph; every pass is still proved below.
        if !matches!(self.output_format, Format::Rgba8 | Format::Bgra8) || self.passes.len() < 4 {
            return Ok(None);
        }
        let proof = ProofContext::new(account);
        let maps = ClosedGraphMaps::try_new(self, &proof)?;
        let valid = validate_closed_graph_accounting(self, &maps).is_some();
        proof.finish()?;
        if !valid || proof.invalid.get() {
            return Ok(None);
        }
        let Some((clear, root)) = closed_graph_root(self, &maps.resource_by_id) else {
            proof.finish()?;
            return Ok(None);
        };
        ClosedGraphTraversal::new(self, maps.resource_by_id, clear, root, &proof)?.run()
    }
}

fn runtime_read_sampler_is_exact(
    read: &RuntimeReadBinding,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
) -> bool {
    let Some(resource) = resources.get(&read.resource).copied() else {
        return false;
    };
    runtime_read_sampler_for_resource_is_exact(read, resource)
}
fn runtime_read_sampler_for_resource_is_exact(
    read: &RuntimeReadBinding,
    resource: &RuntimeResourceRequest,
) -> bool {
    let resolved_mask = match (&read.role, &resource.import) {
        (RuntimeReadRole::AlphaMask, Some(RuntimeResourceImport::ResolvedAlphaMask(upload))) => {
            Some(ShaderMaskSamplingKey::with_rendering(
                upload.quality(),
                upload.rendering_hint(),
                upload.extend(),
            ))
        }
        (RuntimeReadRole::AlphaMask, None) => return false,
        (RuntimeReadRole::ClipCoverage, Some(_)) => return false,
        (RuntimeReadRole::ClipCoverage, None) => None,
        (_, _) => None,
    };
    read.sampler_key
        == SamplerKey::new(
            shader_binding_role(read.role),
            resource.format.shader_key(),
            match read.sampling_filter {
                RuntimeSamplingFilter::Nearest => ShaderSamplingFilterKey::Nearest,
                RuntimeSamplingFilter::Linear => ShaderSamplingFilterKey::Linear,
            },
            shader_sampling_edge(read.sampling_edge),
            resolved_mask,
        )
}

fn runtime_read_has_exact_facts(
    read: &RuntimeReadBinding,
    role: RuntimeReadRole,
    resource: &RuntimeResourceRequest,
    sampling_filter: RuntimeSamplingFilter,
    sampling_edge: RuntimeSamplingEdge,
    proof: &ProofContext<'_>,
) -> bool {
    if !proof.work(RenderWork::GraphEdgeVisits, 1) {
        return false;
    }
    read.role == role
        && read.resource == resource.id
        && read.sampling_filter == sampling_filter
        && read.sampling_edge == sampling_edge
        && runtime_read_sampler_for_resource_is_exact(read, resource)
}

fn validate_closed_clip_coverage_capture(
    pass: &RuntimePass,
    composite: RuntimePassId,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
) -> Option<(RuntimeResourceId, ExecutableVelloCaptureFacts)> {
    let RuntimePassKind::VelloCapture(Some(work @ RuntimeVelloCapture::ClipCoverage(_))) =
        &pass.kind
    else {
        return None;
    };
    let RuntimeResultBinding::Resource(target) = pass.result else {
        return None;
    };
    if !pass.dependencies.is_empty()
        || !pass.reads.is_empty()
        || !pass.releases.is_empty()
        || pass.cache_keys.is_some()
    {
        return None;
    }
    let resource = resources.get(&target).copied()?;
    if !base_graph_resource_has_fixed_facts(
        resource,
        RuntimeResourceRole::ClipCoverage,
        RuntimeResourceFormat::ClipCoverageRgba8Unorm,
        RuntimeResourceProducer::Pass(pass.id),
    ) || resource.expected_reads != 1
        || resource.last_use != composite
    {
        return None;
    }
    Some((
        target,
        resources.proof.capture(executable_vello_capture_facts(
            pass.id,
            target,
            work,
            resource.spatial,
            resources.proof.account,
        ))??,
    ))
}

fn layer_has_exact_clip_coverage_capture(
    layer: &ExecutableLayerCompositionFacts,
    captures: &[ExecutableVelloCaptureFacts],
    proof: &ProofContext<'_>,
) -> bool {
    let RuntimeCompositeKind::Layer {
        transform,
        clip,
        outer_clips,
        clip_coverage,
        ..
    } = &layer.composite.kind
    else {
        return false;
    };
    match (outer_clips.is_empty() && clip.is_none(), clip_coverage) {
        (true, None) => true,
        (false, Some(coverage)) => {
            let mut matching = captures
                .iter()
                .take_while(|_| proof.work(RenderWork::GraphNodeVisits, 1))
                .filter(|capture| capture.target() == *coverage);
            let exact = matching.next().is_some_and(|capture| {
                capture.work().clip_coverage().is_some_and(|coverage| {
                    proof_relationships_equal(
                        coverage.elements.iter(),
                        outer_clips.iter().map(Ok).chain(clip.as_deref().map(Err)),
                        proof,
                        |element, expected| {
                            let expected = match expected {
                                Ok(outer) => (&outer.clip, outer.transform),
                                Err(clip) => (clip, *transform),
                            };
                            clips_match(&element.clip, expected.0, proof)
                                && element.transform == expected.1
                        },
                    )
                })
            });
            exact && matching.next().is_none() && layer.clip_coverage == Some(*coverage)
        }
        (true, Some(_)) | (false, None) => false,
    }
}

fn validate_closed_color_filter(
    pass: &RuntimePass,
    context: ExecutableCompositionContext,
    source: &RuntimeResourceRequest,
    filter: &RuntimeColorFilter,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
    working_format: WorkingFormat,
) -> Option<ExecutableColorFilterFacts> {
    let RuntimeResourceProducer::Pass(source_producer) = source.producer else {
        return None;
    };
    if source.id != context.current
        || source_producer != context.producer
        || !proof_edges_equal(&pass.dependencies, &[source_producer], resources.proof)
        || pass.reads.len() != 1
        || !runtime_read_has_exact_facts(
            &pass.reads[0],
            RuntimeReadRole::FilterSource,
            source,
            RuntimeSamplingFilter::Nearest,
            RuntimeSamplingEdge::ClampToExtent,
            resources.proof,
        )
        || !proof_edges_equal(&pass.releases, &[source.id], resources.proof)
        || pass.cache_keys.is_none()
        || source.format != RuntimeResourceFormat::Working(working_format)
        || !matches!(
            source.role,
            RuntimeResourceRole::BackdropCopy
                | RuntimeResourceRole::FilterIntermediate
                | RuntimeResourceRole::CompositeResult
        )
        || source.expected_reads != 1
        || source.last_use != pass.id
        || filter.operations.is_empty()
        || filter.edge != RuntimeSamplingEdge::ClampToExtent
        || !filter.operations.iter().all(|operation| {
            resources.proof.work(RenderWork::FilterOperationVisits, 1)
                && runtime_color_operation_is_closed(operation)
        })
    {
        return None;
    }
    let RuntimeResultBinding::Resource(result) = pass.result else {
        return None;
    };
    if result == source.id {
        return None;
    }
    let result_resource = resources.get(&result).copied()?;
    if !base_graph_resource_has_fixed_facts(
        result_resource,
        RuntimeResourceRole::FilterIntermediate,
        RuntimeResourceFormat::Working(working_format),
        RuntimeResourceProducer::Pass(pass.id),
    ) || result_resource.expected_reads != 1
        || source.spatial != result_resource.spatial
        || filter.spatial.source != source.spatial
        || filter.spatial.result != result_resource.spatial
    {
        return None;
    }
    Some(ExecutableColorFilterFacts {
        pass: pass.id,
        source: source.id,
        result,
        filter: filter.clone(),
    })
}

fn runtime_color_operation_is_closed(operation: &RuntimeColorOperation) -> bool {
    if operation.clamp_boundary != RuntimeColorClampBoundary::ClampStraightRgbaToUnitThenPremultiply
    {
        return false;
    }
    match operation.operation {
        RuntimeColorOperationKind::Brightness(amount)
        | RuntimeColorOperationKind::Contrast(amount)
        | RuntimeColorOperationKind::Saturate(amount) => {
            if amount.zero() {
                amount.mantissa() == 0.0 && amount.exponent() == 0
            } else {
                amount.mantissa().is_finite() && (0.5..1.0).contains(&amount.mantissa())
            }
        }
        RuntimeColorOperationKind::Grayscale(amount)
        | RuntimeColorOperationKind::Invert(amount)
        | RuntimeColorOperationKind::Opacity(amount)
        | RuntimeColorOperationKind::Sepia(amount) => {
            amount.value().is_finite() && (0.0..=1.0).contains(&amount.value())
        }
        RuntimeColorOperationKind::HueRotate(angle) => {
            angle.sine().is_finite() && angle.cosine().is_finite()
        }
    }
}

#[derive(Clone, Copy)]
struct ClosedFilterValidation<'plan> {
    context: ExecutableCompositionContext,
    source: &'plan RuntimeResourceRequest,
    resources: &'plan ProofMap<'plan, RuntimeResourceId, &'plan RuntimeResourceRequest>,
    working_format: WorkingFormat,
    edge: RuntimeSamplingEdge,
}

fn validate_closed_blur(
    horizontal: &RuntimePass,
    vertical: &RuntimePass,
    blur: &RuntimeBlur,
    validation: ClosedFilterValidation<'_>,
) -> Option<ExecutableBlurFacts> {
    let ClosedFilterValidation {
        context,
        source,
        resources,
        working_format,
        edge,
    } = validation;
    let RuntimeResourceProducer::Pass(source_producer) = source.producer else {
        return None;
    };
    if source.id != context.current
        || source_producer != context.producer
        || blur.axis != RuntimeBlurAxis::Horizontal
        || blur.input != RuntimeBlurInput::Rgba
        || !runtime_blur_is_closed(blur, true, edge)
        || blur.spatial.source != source.spatial
        || !closed_filter_source_is_exact(source, working_format, 1, horizontal.id)
        || !closed_unary_filter_pass_is_exact(
            horizontal,
            source_producer,
            source,
            RuntimeReadRole::FilterSource,
            true,
            edge,
            resources.proof,
        )
    {
        return None;
    }
    let intermediate = closed_filter_result(
        horizontal,
        resources,
        RuntimeResourceRole::FilterIntermediate,
        blur.spatial.result,
        working_format,
    )?;
    let RuntimePassKind::BlurVertical(Some(vertical_blur)) = &vertical.kind else {
        return None;
    };
    if !runtime_blur_matches_axis(vertical_blur, blur, RuntimeBlurAxis::Vertical)
        || intermediate.expected_reads != 1
        || intermediate.last_use != vertical.id
        || !closed_unary_filter_pass_is_exact(
            vertical,
            horizontal.id,
            intermediate,
            RuntimeReadRole::FilterSource,
            true,
            edge,
            resources.proof,
        )
    {
        return None;
    }
    let result = closed_filter_result(
        vertical,
        resources,
        RuntimeResourceRole::FilterIntermediate,
        blur.spatial.result,
        working_format,
    )?;
    Some(ExecutableBlurFacts {
        horizontal: horizontal.id,
        vertical: vertical.id,
        source: source.id,
        intermediate: intermediate.id,
        result: result.id,
        blur: blur.clone(),
    })
}

fn validate_closed_drop_shadow(
    passes: [&RuntimePass; 4],
    blur: &RuntimeBlur,
    validation: ClosedFilterValidation<'_>,
) -> Option<ExecutableDropShadowFacts> {
    let ClosedFilterValidation {
        context,
        source,
        resources,
        working_format,
        edge,
    } = validation;
    let [horizontal, vertical, colorize, merge] = passes;
    let RuntimeResourceProducer::Pass(source_producer) = source.producer else {
        return None;
    };
    if source.id != context.current
        || source_producer != context.producer
        || blur.axis != RuntimeBlurAxis::Horizontal
        || blur.input != RuntimeBlurInput::SourceAlpha
        || !runtime_blur_is_closed(blur, false, edge)
        || blur.spatial.source != source.spatial
        || !closed_filter_source_is_exact(source, working_format, 2, merge.id)
        || !closed_unary_filter_pass_is_exact(
            horizontal,
            source_producer,
            source,
            RuntimeReadRole::FilterSource,
            false,
            edge,
            resources.proof,
        )
    {
        return None;
    }
    let horizontal_result = closed_filter_result(
        horizontal,
        resources,
        RuntimeResourceRole::FilterIntermediate,
        blur.spatial.result,
        working_format,
    )?;
    let (vertical_result, parameters, shadow) = validate_closed_shadow_tail(
        [vertical, colorize],
        horizontal,
        horizontal_result,
        blur,
        resources,
        working_format,
        edge,
    )?;
    let result = validate_closed_shadow_merge(
        merge,
        ClosedShadowMergeInputs {
            source_producer,
            source,
            colorize,
            shadow,
            result_spatial: parameters.spatial.result,
        },
        resources,
        working_format,
    )?;
    Some(ExecutableDropShadowFacts {
        horizontal: horizontal.id,
        vertical: vertical.id,
        colorize: colorize.id,
        merge: merge.id,
        source: source.id,
        horizontal_result: horizontal_result.id,
        vertical_result: vertical_result.id,
        shadow: shadow.id,
        result: result.id,
        blur: blur.clone(),
        parameters,
    })
}

fn validate_closed_shadow_tail<'plan>(
    passes: [&RuntimePass; 2],
    horizontal: &RuntimePass,
    horizontal_result: &'plan RuntimeResourceRequest,
    blur: &RuntimeBlur,
    resources: &ProofMap<'_, RuntimeResourceId, &'plan RuntimeResourceRequest>,
    working_format: WorkingFormat,
    edge: RuntimeSamplingEdge,
) -> Option<(
    &'plan RuntimeResourceRequest,
    RuntimeDropShadow,
    &'plan RuntimeResourceRequest,
)> {
    let [vertical, colorize] = passes;
    let RuntimePassKind::BlurVertical(Some(vertical_blur)) = &vertical.kind else {
        return None;
    };
    if !runtime_blur_matches_axis(vertical_blur, blur, RuntimeBlurAxis::Vertical)
        || horizontal_result.expected_reads != 1
        || horizontal_result.last_use != vertical.id
        || !closed_unary_filter_pass_is_exact(
            vertical,
            horizontal.id,
            horizontal_result,
            RuntimeReadRole::FilterSource,
            true,
            edge,
            resources.proof,
        )
    {
        return None;
    }
    let vertical_result = closed_filter_result(
        vertical,
        resources,
        RuntimeResourceRole::FilterIntermediate,
        blur.spatial.result,
        working_format,
    )?;
    let RuntimePassKind::DropShadowColorize(Some(parameters)) = &colorize.kind else {
        return None;
    };
    if !runtime_drop_shadow_is_closed(parameters, blur)
        || vertical_result.expected_reads != 1
        || vertical_result.last_use != colorize.id
        || !closed_unary_filter_pass_is_exact(
            colorize,
            vertical.id,
            vertical_result,
            RuntimeReadRole::BlurredSourceAlpha,
            true,
            RuntimeSamplingEdge::TransparentBlack,
            resources.proof,
        )
    {
        return None;
    }
    let shadow = closed_filter_result(
        colorize,
        resources,
        RuntimeResourceRole::ShadowImage,
        parameters.spatial.result,
        working_format,
    )?;
    Some((vertical_result, *parameters, shadow))
}

struct ClosedShadowMergeInputs<'plan> {
    source_producer: RuntimePassId,
    source: &'plan RuntimeResourceRequest,
    colorize: &'plan RuntimePass,
    shadow: &'plan RuntimeResourceRequest,
    result_spatial: RuntimeSpatialDescriptor,
}

fn validate_closed_shadow_merge<'plan>(
    merge: &RuntimePass,
    inputs: ClosedShadowMergeInputs<'plan>,
    resources: &ProofMap<'_, RuntimeResourceId, &'plan RuntimeResourceRequest>,
    working_format: WorkingFormat,
) -> Option<&'plan RuntimeResourceRequest> {
    let ClosedShadowMergeInputs {
        source_producer,
        source,
        colorize,
        shadow,
        result_spatial,
    } = inputs;
    let RuntimePassKind::Composite(Some(composite)) = &merge.kind else {
        return None;
    };
    if !matches!(composite.kind, RuntimeCompositeKind::DropShadow)
        || !composite.source_captured_before_outer_semantics
        || !proof_edges_equal(
            &merge.dependencies,
            &[source_producer, colorize.id],
            resources.proof,
        )
        || merge.reads.len() != 2
        || !runtime_read_has_exact_facts(
            &merge.reads[0],
            RuntimeReadRole::CompositeSource,
            source,
            RuntimeSamplingFilter::Linear,
            RuntimeSamplingEdge::TransparentBlack,
            resources.proof,
        )
        || !runtime_read_has_exact_facts(
            &merge.reads[1],
            RuntimeReadRole::Shadow,
            shadow,
            RuntimeSamplingFilter::Linear,
            RuntimeSamplingEdge::TransparentBlack,
            resources.proof,
        )
        || !same_resource_set(&merge.releases, &[source.id, shadow.id], resources.proof)
        || merge.cache_keys.is_none()
        || shadow.expected_reads != 1
        || shadow.last_use != merge.id
    {
        return None;
    }
    closed_filter_result(
        merge,
        resources,
        RuntimeResourceRole::CompositeResult,
        result_spatial,
        working_format,
    )
}

fn closed_unary_filter_pass_is_exact(
    pass: &RuntimePass,
    producer: RuntimePassId,
    source: &RuntimeResourceRequest,
    role: RuntimeReadRole,
    releases_source: bool,
    edge: RuntimeSamplingEdge,
    proof: &ProofContext<'_>,
) -> bool {
    proof_edges_equal(&pass.dependencies, &[producer], proof)
        && pass.reads.len() == 1
        && runtime_read_has_exact_facts(
            &pass.reads[0],
            role,
            source,
            RuntimeSamplingFilter::Linear,
            edge,
            proof,
        )
        && if releases_source {
            proof_edges_equal(&pass.releases, &[source.id], proof)
        } else {
            pass.releases.is_empty()
        }
        && pass.cache_keys.is_some()
}

fn closed_filter_source_is_exact(
    source: &RuntimeResourceRequest,
    working_format: WorkingFormat,
    expected_reads: u32,
    last_use: RuntimePassId,
) -> bool {
    matches!(
        source.role,
        RuntimeResourceRole::BackdropCopy
            | RuntimeResourceRole::FilterIntermediate
            | RuntimeResourceRole::CompositeResult
    ) && source.format == RuntimeResourceFormat::Working(working_format)
        && source.expected_reads == expected_reads
        && source.last_use == last_use
}

fn closed_filter_result<'plan>(
    pass: &RuntimePass,
    resources: &ProofMap<'_, RuntimeResourceId, &'plan RuntimeResourceRequest>,
    role: RuntimeResourceRole,
    spatial: RuntimeSpatialDescriptor,
    working_format: WorkingFormat,
) -> Option<&'plan RuntimeResourceRequest> {
    let RuntimeResultBinding::Resource(result) = pass.result else {
        return None;
    };
    let resource = resources.get(&result).copied()?;
    base_graph_resource_has_fixed_facts(
        resource,
        role,
        RuntimeResourceFormat::Working(working_format),
        RuntimeResourceProducer::Pass(pass.id),
    )
    .then_some(())
    .filter(|()| resource.spatial == spatial)?;
    Some(resource)
}

fn runtime_blur_is_closed(
    blur: &RuntimeBlur,
    require_nonzero: bool,
    edge: RuntimeSamplingEdge,
) -> bool {
    blur.standard_deviation.is_finite()
        && if require_nonzero {
            blur.standard_deviation > 0.0 && blur.support_radius > 0
        } else {
            blur.standard_deviation >= 0.0
        }
        && blur.edge == edge
        && blur.spatial.source.raster_scale.is_finite()
        && blur.spatial.result.raster_scale.is_finite()
}

fn runtime_drop_shadow_is_closed(shadow: &RuntimeDropShadow, blur: &RuntimeBlur) -> bool {
    shadow.standard_deviation == blur.standard_deviation
        && shadow.support_radius == blur.support_radius
        && shadow.spatial.source == blur.spatial.result
        && shadow.spatial.result == blur.spatial.result
        && shadow.edge == blur.edge
        && shadow.uses_source_alpha
        && shadow.uses_continuous_offset
        && shadow.retains_unchanged_source
        && shadow.offset.x().is_finite()
        && shadow.offset.y().is_finite()
        && [
            shadow.color.r(),
            shadow.color.g(),
            shadow.color.b(),
            shadow.color.a(),
        ]
        .into_iter()
        .all(f32::is_finite)
}

fn same_resource_set(
    actual: &[RuntimeResourceId],
    expected: &[RuntimeResourceId],
    proof: &ProofContext<'_>,
) -> bool {
    let contains = |ids: &[RuntimeResourceId], expected: &RuntimeResourceId| {
        for id in ids {
            if !proof.work(RenderWork::GraphEdgeVisits, 1) {
                return false;
            }
            if id == expected {
                return true;
            }
        }
        false
    };
    actual.len() == expected.len()
        && actual
            .iter()
            .all(|id| proof.work(RenderWork::GraphEdgeVisits, 1) && contains(expected, id))
        && expected
            .iter()
            .all(|id| proof.work(RenderWork::GraphEdgeVisits, 1) && contains(actual, id))
}

#[cfg(test)]
mod layer_composite_read_tests {
    use super::*;
    use crate::{
        BlendMode, Capabilities, Image, ImageId, ImageQuality, Layer, Rect,
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus,
        RenderEvent, RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile, RenderQuantity,
        RenderRefusalReason, ResolvedLayerAlphaMask, Scene, Size,
        backend::DeviceCapabilities,
        frame::{FrameContext, FramePlan},
    };

    fn authored_plan(clip: bool, mask: bool) -> (LoweredGraphPlan, Option<ImageId>) {
        let account = Account::for_test();
        let mut layer = Layer::new().isolated().blend(BlendMode::Multiply);
        let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
        if clip {
            layer = layer.try_clip(bounds.into()).unwrap();
        }
        let mut mask_id = None;
        if mask {
            let image = Image::from_rgba(
                Size::new(2.0, 1.0),
                vec![255, 255, 255, 128, 255, 255, 255, 255],
            )
            .unwrap()
            .quality(ImageQuality::Low);
            mask_id = Some(image.id());
            layer = layer
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(image, bounds).unwrap());
        }
        let mut scene = Scene::new();
        scene.layer(layer, |child| {
            child.fill(bounds, Color::BLACK);
        });
        let commands = scene.normalize(Capabilities::CURRENT, &account).unwrap();
        let context =
            FrameContext::try_new(Size::new(4.0, 3.0), 1.0, Antialiasing::Area, Color::BLACK)
                .unwrap();
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, &account).unwrap() else {
            panic!("the isolated multiply layer must use the GPU graph");
        };
        let plan = LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            WorkingFormat::HighPrecision,
            Format::Rgba8,
            &DeviceCapabilities::from_test_facts(true, true, 4096),
            &account,
        )
        .unwrap();
        (plan, mask_id)
    }

    fn layer_pass(plan: &LoweredGraphPlan) -> &RuntimePass {
        plan.passes
            .iter()
            .find(|pass| {
                matches!(
                    &pass.kind,
                    RuntimePassKind::Composite(Some(RuntimeComposite {
                        kind: RuntimeCompositeKind::Layer { .. },
                        ..
                    }))
                )
            })
            .unwrap()
    }

    fn resource_result(pass: &RuntimePass) -> RuntimeResourceId {
        let RuntimeResultBinding::Resource(resource) = pass.result else {
            panic!("the fixture pass must produce a resource");
        };
        resource
    }

    #[test]
    fn layer_composite_reads_preserve_four_authored_shapes_and_results() {
        use RuntimeReadRole::{AlphaMask, ClipCoverage, CompositeParent, CompositeSource};
        use RuntimeSamplingEdge::{ClampToExtent, TransparentBlack};
        use RuntimeSamplingFilter::{Linear, Nearest};

        for (clip, mask) in [(false, false), (true, false), (false, true), (true, true)] {
            let (plan, mask_id) = authored_plan(clip, mask);
            let closed = ClosedExecutableGraph::try_from_lowered(plan, &Account::for_test())
                .unwrap()
                .unwrap();
            let plan = &closed.lowered;
            let pass = layer_pass(plan);
            let [layer] = closed.facts.layer_compositions.as_slice() else {
                panic!("one authored layer must close as one layer composite");
            };
            let source = plan
                .passes
                .iter()
                .find(|pass| {
                    matches!(
                        &pass.kind,
                        RuntimePassKind::Composite(Some(RuntimeComposite {
                            kind: RuntimeCompositeKind::SpanSourceOver,
                            ..
                        }))
                    )
                })
                .map(resource_result)
                .unwrap();
            let coverage = plan
                .passes
                .iter()
                .find(|pass| {
                    matches!(
                        &pass.kind,
                        RuntimePassKind::VelloCapture(Some(RuntimeVelloCapture::ClipCoverage(_)))
                    )
                })
                .map(resource_result);
            assert_eq!(coverage.is_some(), clip);
            let imported = plan.resources.iter().find_map(|resource| {
                let Some(RuntimeResourceImport::ResolvedAlphaMask(upload)) = &resource.import
                else {
                    return None;
                };
                assert_eq!(Some(upload.image().id()), mask_id);
                assert_eq!(upload.physical_size(), PhysicalSize::new(2, 1));
                assert_eq!(resource.producer, RuntimeResourceProducer::Imported);
                Some(resource.id)
            });
            assert_eq!(imported.is_some(), mask);
            let root = plan.root_working_image;
            let mut expected = vec![
                (CompositeParent, root, Linear, ClampToExtent),
                (CompositeSource, source, Linear, TransparentBlack),
            ];
            if let Some(id) = coverage {
                expected.push((ClipCoverage, id, Linear, TransparentBlack));
            }
            if let Some(id) = imported {
                expected.push((AlphaMask, id, Nearest, ClampToExtent));
            }
            let observed: Vec<_> = pass
                .reads
                .iter()
                .map(|read| {
                    (
                        read.role,
                        read.resource,
                        read.sampling_filter,
                        read.sampling_edge,
                    )
                })
                .collect();
            assert_eq!(observed, expected, "clip {clip}, mask {mask}");

            let resource = |id| {
                plan.resources
                    .iter()
                    .find(|resource| resource.id == id)
                    .unwrap()
            };
            assert_eq!(pass.releases.len(), expected.len());
            for (_, id, _, _) in expected {
                assert!(pass.releases.contains(&id));
                assert_eq!(resource(id).expected_reads, 1);
                assert_eq!(resource(id).last_use, pass.id);
            }
            let result = resource_result(pass);
            assert_eq!(layer.pass, pass.id);
            assert_eq!(
                (layer.parent, layer.source, layer.result),
                (root, source, result)
            );
            assert_eq!(
                (layer.clip_coverage, layer.alpha_mask),
                (coverage, imported)
            );
            assert_eq!(resource(result).role, RuntimeResourceRole::CompositeResult);
            assert_eq!(
                resource(result).producer,
                RuntimeResourceProducer::Pass(pass.id)
            );
            assert_eq!(
                resource(result).format,
                RuntimeResourceFormat::Working(WorkingFormat::HighPrecision)
            );
            assert_eq!(resource(result).spatial, resource(root).spatial);
            assert_eq!(
                resource(result).spatial.device_extent(),
                PhysicalSize::new(4, 3)
            );

            let present = plan.passes.last().unwrap();
            assert_eq!(present.id, plan.final_present);
            assert!(matches!(present.kind, RuntimePassKind::Present));
            assert_eq!(present.result, RuntimeResultBinding::Output(Format::Rgba8));
            assert_eq!(resource(result).last_use, present.id);
            assert_eq!(present.reads.len(), 1);
            assert_eq!(present.reads[0].role, RuntimeReadRole::FinalWorkingImage);
            assert_eq!(present.reads[0].resource, result);
        }
    }

    #[test]
    fn layer_composite_read_budgets_reach_coverage_before_mask() {
        use RenderWork::{GraphEdgeVisits, ResourceProbes};

        let (plan, _) = authored_plan(true, true);
        let pass = layer_pass(&plan);
        let RuntimeResourceProducer::Pass(producer) = plan.resources[0].producer else {
            panic!("the root has a producer pass");
        };
        let parent = ExecutableCompositionContext {
            current: plan.root_working_image,
            producer,
            contains_captured_source: false,
        };
        let source = plan
            .resources
            .iter()
            .find(|resource| resource.id == pass.reads[1].resource)
            .unwrap();
        // The eight declaration-order IDs give four root probes, two coverage
        // probes and three result probes before the optional reads. Coverage
        // needs two more probes; the imported mask needs three. The first five
        // edges check three dependencies and the two mandatory reads.
        for (work, limit, edges, probes, succeeds) in [
            (GraphEdgeVisits, 5, 5, 11, false),
            (GraphEdgeVisits, 6, 6, 14, false),
            (ResourceProbes, 11, 6, 11, false),
            (GraphEdgeVisits, 7, 7, 14, true),
        ] {
            let setup = Account::for_test();
            let proof = ProofContext::new(&setup);
            let maps = ClosedGraphMaps::try_new(&plan, &proof).unwrap();
            let ProofMap { entries, live, .. } = maps.resource_by_id;
            let options = RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_work_limit(work, limit).unwrap())
                .with_profile(RenderProfile::Detailed {
                    timing: false,
                    event_limit: 32,
                    filter: RenderPhaseFilter::Phase(RenderPhase::Lower),
                });
            let mut accounting = RenderAccountingContext::new(options);
            let result = accounting.run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                let proof = ProofContext::new(account);
                // Reuse the real graph resource map. The fresh budget applies
                // to composite validation after the map is established.
                let resources = ProofMap {
                    entries,
                    live,
                    proof: &proof,
                };
                let layer = validate_closed_composite(
                    pass,
                    parent,
                    source,
                    &resources,
                    plan.working_format,
                    true,
                );
                proof.finish()?;
                Ok(layer.flatten().map(|layer| layer.result))
            });
            let attempt = accounting.last_attempt().unwrap().attempt().unwrap();
            let usage = attempt.phase_usage(RenderPhase::Lower);
            assert_eq!(usage.work(GraphEdgeVisits), edges, "{work:?} cap {limit}");
            assert_eq!(usage.work(ResourceProbes), probes, "{work:?} cap {limit}");
            if succeeds {
                assert_eq!(result.unwrap(), Some(resource_result(pass)));
                assert_eq!(attempt.status(), RenderAttemptStatus::Succeeded);
                assert_eq!(attempt.refusal(), None);
            } else {
                let refusal = attempt.refusal().unwrap();
                assert_eq!(attempt.status(), RenderAttemptStatus::Refused);
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
                assert_eq!((refusal.current(), refusal.amount()), (limit, 1));
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
                assert_eq!(
                    result.unwrap_err().accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(refusal))
                );
                assert!(attempt.events().any(|event| matches!(
                    event,
                    RenderEvent::Refused {
                        phase: RenderPhase::Lower,
                        ..
                    }
                )));
            }
        }
    }
}

#[cfg(test)]
mod repeated_proof_accounting_tests {
    use super::*;
    use crate::pass::test_support::{
        lower_capture_proof_fixture, lower_filter_proof_fixture, measure_proof_work,
    };

    fn shared<T: Clone>(values: &[T]) -> SharedValue<Vec<T>> {
        let account = Account::for_test();
        let mut backing = account.retained(values.len()).unwrap();
        backing.extend_from_slice(values).unwrap();
        SharedValue::from_accounted(backing, &account.storage()).unwrap()
    }

    fn empty_source_plan() -> LoweredGraphPlan {
        let account = Account::for_test();
        let mut scene = crate::Scene::new();
        scene.layer(
            crate::Layer::new().with_composition(
                crate::LayerComposition::try_porter_duff(
                    crate::PorterDuffOperator::Clear,
                    crate::BlendMode::Normal,
                )
                .unwrap(),
            ),
            |_| {},
        );
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, &account)
            .unwrap();
        let context = crate::frame::FrameContext::try_new(
            crate::Size::new(2.0, 2.0),
            1.0,
            Antialiasing::Area,
            Color::BLACK,
        )
        .unwrap();
        let crate::frame::FramePlan::GpuGraph(graph) =
            commands.plan_for(context, &account).unwrap()
        else {
            panic!("Clear requires its finite operation graph");
        };
        LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            WorkingFormat::HighPrecision,
            Format::Rgba8,
            &crate::backend::DeviceCapabilities::from_test_facts(true, true, 4096),
            &account,
        )
        .unwrap()
    }

    #[test]
    fn malformed_backdrop_copy_refuses_unreserved_context_storage() {
        use crate::accounting::StorageContractError;

        let mut plan = empty_source_plan();
        let setup = Account::for_test();
        assert!(
            ClosedExecutableGraph::try_from_lowered(plan.clone(), &setup)
                .unwrap()
                .is_ok()
        );
        let mut passes = plan.passes.to_vec();
        let [root_clear, copy, composite, present] = passes.as_mut_slice() else {
            panic!("expected root clear, isolation clear, composite and present");
        };
        assert!(matches!(
            copy.kind,
            RuntimePassKind::ClearRoot {
                initialization: RuntimeInitialization::Transparent,
                ..
            }
        ));
        assert!(matches!(composite.kind, RuntimePassKind::Composite(_)));
        assert!(matches!(present.kind, RuntimePassKind::Present));
        assert!(copy.releases.is_empty());
        let RuntimeResultBinding::Resource(copied) = copy.result else {
            panic!("the original isolation clear must produce a resource");
        };
        let mut resources = plan.resources.to_vec();
        let root = resources
            .iter_mut()
            .find(|resource| resource.id == plan.root_working_image)
            .unwrap();
        root.expected_reads += 1;
        let role = RuntimeReadRole::CompletedParent;
        let edge = RuntimeSamplingEdge::TransparentBlack;
        copy.kind = RuntimePassKind::CopyBackdrop;
        copy.dependencies = shared(&[root_clear.id]);
        copy.reads = shared(&[RuntimeReadBinding {
            role,
            resource: root.id,
            sampling_filter: RuntimeSamplingFilter::Nearest,
            sampling_edge: edge,
            sampler_key: SamplerKey::new(
                shader_binding_role(role),
                root.format.shader_key(),
                ShaderSamplingFilterKey::Nearest,
                shader_sampling_edge(edge),
                None,
            ),
        }]);
        let output = resources
            .iter_mut()
            .find(|resource| resource.id == copied)
            .unwrap();
        assert_eq!(output.role, RuntimeResourceRole::IsolationWorkingImage);
        output.role = RuntimeResourceRole::BackdropCopy;
        copy.cache_keys = runtime_pass_cache_keys(
            &copy.kind,
            &copy.reads,
            copy.result,
            plan.working_format,
            plan.output_format,
            |id| {
                Ok(resources
                    .iter()
                    .find(|resource| resource.id == id)
                    .map(|resource| resource.format))
            },
            &setup,
        )
        .unwrap();
        plan.passes = shared(&passes);
        plan.resources = shared(&resources);

        // The generic accounting proof accepts the copy's exact dependencies,
        // reads and produced resource. The context reservation still has only
        // the root slot because the transparent clear is gone.
        let accounting = Account::for_test();
        let proof = ProofContext::new(&accounting);
        let maps = ClosedGraphMaps::try_new(&plan, &proof).unwrap();
        assert!(validate_closed_graph_accounting(&plan, &maps).is_some());
        proof.finish().unwrap();
        assert!(!proof.invalid.get());
        drop(maps);

        let error = ClosedExecutableGraph::try_from_lowered(plan, &Account::for_test())
            .err()
            .expect("the second context must return a typed storage error");
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(error.accounting_diagnostic(), None);
        assert_eq!(
            std::error::Error::source(&error).and_then(|source| source.downcast_ref()),
            Some(&StorageContractError::ReservedCapacity {
                needed: 2,
                capacity: 1
            }),
        );
    }

    #[test]
    fn empty_source_execution_requires_transparent_initialization_and_exact_custody() {
        let plan = empty_source_plan();
        let account = Account::for_test();
        assert!(
            ClosedExecutableGraph::try_from_lowered(plan.clone(), &account)
                .unwrap()
                .is_ok()
        );
        for alteration in 0..4 {
            let mut changed = plan.clone();
            match alteration {
                0 => {
                    let mut passes = changed.passes.to_vec();
                    let color = passes
                        .iter_mut()
                        .find_map(|pass| match &mut pass.kind {
                            RuntimePassKind::ClearRoot {
                                initialization: RuntimeInitialization::Transparent,
                                color,
                            } => Some(color),
                            _ => None,
                        })
                        .unwrap();
                    *color = Color::BLACK;
                    changed.passes = shared(&passes);
                }
                1 | 2 => {
                    let mut resources = changed.resources.to_vec();
                    let source = resources
                        .iter_mut()
                        .find(|resource| {
                            resource.role == RuntimeResourceRole::IsolationWorkingImage
                        })
                        .unwrap();
                    if alteration == 1 {
                        source.producer = RuntimeResourceProducer::Pass(changed.passes[0].id);
                    } else {
                        source.last_use = changed.final_present;
                    }
                    changed.resources = shared(&resources);
                }
                _ => {
                    let mut passes = changed.passes.to_vec();
                    let composite = passes
                        .iter_mut()
                        .find(|pass| matches!(pass.kind, RuntimePassKind::Composite(_)))
                        .unwrap();
                    let mut reads = composite.reads.to_vec();
                    let source = reads
                        .iter_mut()
                        .find(|read| read.role == RuntimeReadRole::CompositeSource)
                        .unwrap();
                    source.sampling_edge = RuntimeSamplingEdge::ClampToExtent;
                    source.sampler_key = SamplerKey::new(
                        shader_binding_role(source.role),
                        RuntimeResourceFormat::Working(WorkingFormat::HighPrecision).shader_key(),
                        ShaderSamplingFilterKey::Linear,
                        shader_sampling_edge(source.sampling_edge),
                        None,
                    );
                    composite.reads = shared(&reads);
                    changed.passes = shared(&passes);
                }
            }
            assert!(
                ClosedExecutableGraph::try_from_lowered(changed, &account)
                    .unwrap()
                    .is_err(),
                "empty-source alteration {alteration} must fail its execution proof"
            );
        }
    }

    fn resource_entries(
        plan: &LoweredGraphPlan,
    ) -> AccountedVec<(RuntimeResourceId, Option<&RuntimeResourceRequest>)> {
        let account = Account::for_test();
        let mut entries = account.temporary(plan.resources.len()).unwrap();
        for resource in plan.resources.iter() {
            entries.push((resource.id, Some(resource))).unwrap();
        }
        entries.sort_unstable_by_key(|entry| entry.0);
        entries
    }

    fn color_plan() -> LoweredGraphPlan {
        lower_filter_proof_fixture(vec![
            crate::FilterList::try_ops(vec![
                crate::FilterOp::brightness(crate::FilterAmount::try_new(1.25).unwrap()),
                crate::FilterOp::contrast(crate::FilterAmount::try_new(0.75).unwrap()),
                crate::FilterOp::sepia(crate::UnitFilterAmount::try_new(0.5).unwrap()),
            ])
            .unwrap(),
        ])
    }

    fn shadow_plan() -> LoweredGraphPlan {
        lower_filter_proof_fixture(vec![
            crate::FilterList::try_ops(vec![crate::FilterOp::drop_shadow(
                crate::FilterDropShadow::try_new(
                    Point::new(0.0, 0.0),
                    crate::FilterBlur::try_new(0.25).unwrap(),
                    Color::BLACK,
                )
                .unwrap(),
            )])
            .unwrap(),
        ])
    }

    fn assert_refusal(report: &crate::RenderFrameReport, work: RenderWork, current: u64) {
        let refusal = report
            .attempt()
            .unwrap()
            .refusal()
            .expect("accounted proof refusal");
        assert_eq!(refusal.quantity, crate::RenderQuantity::Work(work));
        assert_eq!(refusal.current, current);
        assert_eq!(refusal.amount, 1);
    }

    #[test]
    fn canonical_source_checks_retain_fixed_fields_and_exact_payload_provenance() {
        let plan = color_plan();
        let setup = Account::for_test();
        let work = plan
            .passes
            .iter()
            .find_map(|pass| match &pass.kind {
                RuntimePassKind::VelloCapture(Some(work)) => Some(work),
                _ => None,
            })
            .unwrap();
        let RuntimeVelloCapture::Span(span) = work else {
            unreachable!()
        };
        assert!(same_source_capture(work, &work.clone()));
        assert!(same_source_span(span, &span.clone()));
        let mut changed = span.clone();
        changed.capture_transform = Transform::translation(1.0, 0.0).unwrap();
        assert!(!same_source_span(span, &changed));
        let mut copied = setup.retained(span.commands.len()).unwrap();
        copied.extend_from_slice(&span.commands).unwrap();
        changed = span.clone();
        changed.commands = crate::command::RenderCommands::new_in(copied, &setup).unwrap();
        assert_eq!(
            span.commands, changed.commands,
            "content equality remains unchanged"
        );
        assert!(
            !same_source_span(span, &changed),
            "a canonical source certificate cannot substitute another allocation"
        );

        let filter = plan
            .passes
            .iter()
            .find_map(|pass| match &pass.kind {
                RuntimePassKind::ColorFilter(Some(filter)) => Some(filter),
                _ => None,
            })
            .unwrap();
        assert!(same_source_color_filter(filter, &filter.clone()));
        let mut changed = filter.clone();
        changed.edge = RuntimeSamplingEdge::TransparentBlack;
        assert!(!same_source_color_filter(filter, &changed));
        changed = filter.clone();
        changed.operations = shared(&filter.operations);
        assert_eq!(*filter, changed);
        assert!(!same_source_color_filter(filter, &changed));

        let composite = plan
            .passes
            .iter()
            .find_map(|pass| match &pass.kind {
                RuntimePassKind::Composite(Some(composite))
                    if matches!(composite.kind, RuntimeCompositeKind::Layer { .. }) =>
                {
                    Some(composite)
                }
                _ => None,
            })
            .unwrap();
        assert!(same_source_composite(composite, &composite.clone()));
        let mut changed = composite.clone();
        changed.source_captured_before_outer_semantics =
            !changed.source_captured_before_outer_semantics;
        assert!(!same_source_composite(composite, &changed));
        changed = composite.clone();
        let RuntimeCompositeKind::Layer { parameters, .. } = &mut changed.kind else {
            unreachable!()
        };
        let mut changed_parameters = **parameters;
        changed_parameters.opacity = 0.5;
        *parameters = SharedValue::new(changed_parameters, &setup.storage()).unwrap();
        assert!(!same_source_composite(composite, &changed));
        changed = composite.clone();
        let RuntimeCompositeKind::Layer { outer_clips, .. } = &mut changed.kind else {
            unreachable!()
        };
        *outer_clips = shared(outer_clips);
        assert_eq!(*composite, changed);
        assert!(!same_source_composite(composite, &changed));

        let clipped = lower_capture_proof_fixture(
            1,
            true,
            &crate::backend::DeviceCapabilities::from_test_facts(true, true, 4096),
        );
        let coverage = clipped
            .passes
            .iter()
            .find_map(|pass| match &pass.kind {
                RuntimePassKind::VelloCapture(Some(
                    work @ RuntimeVelloCapture::ClipCoverage(_),
                )) => Some(work),
                _ => None,
            })
            .unwrap();
        assert!(same_source_capture(coverage, &coverage.clone()));
        let mut changed = coverage.clone();
        let RuntimeVelloCapture::ClipCoverage(payload) = &mut changed else {
            unreachable!()
        };
        payload.elements = shared(&payload.elements);
        assert_eq!(*coverage, changed);
        assert!(!same_source_capture(coverage, &changed));
    }

    #[test]
    fn backdrop_step_comparison_admits_exact_pairs_and_stops_at_first_difference() {
        let plan = color_plan();
        let actual = [
            ExecutableFilterStepFacts::Color(plan.passes[0].id),
            ExecutableFilterStepFacts::Color(plan.passes[1].id),
            ExecutableFilterStepFacts::Color(plan.passes[2].id),
        ];
        for mismatch in [false, true] {
            let mut expected = actual;
            if mismatch {
                expected[1] = actual[2];
            }
            let count = if mismatch { 2 } else { 3 };
            for limit in [count, count - 1] {
                let (result, report) =
                    measure_proof_work(RenderWork::GraphNodeVisits, limit, |account| {
                        filter_steps_match(&actual, &expected, account)
                    });
                if limit == count {
                    assert_eq!(result.unwrap(), !mismatch);
                    assert_eq!(
                        report
                            .attempt()
                            .unwrap()
                            .usage()
                            .work(RenderWork::GraphNodeVisits),
                        count
                    );
                } else {
                    assert!(result.is_err());
                    assert_refusal(&report, RenderWork::GraphNodeVisits, limit);
                }
            }
        }
        let (result, report) = measure_proof_work(RenderWork::GraphNodeVisits, 0, |account| {
            filter_steps_match(&actual, &actual[..2], account)
        });
        assert!(!result.unwrap());
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::GraphNodeVisits),
            0
        );
    }

    #[test]
    fn repeated_proof_reconstructed_cache_keys_refuse_before_unadmitted_binding_comparison() {
        let plan = color_plan();
        let first_keyed_pass = plan
            .passes
            .iter()
            .find(|pass| pass.cache_keys.is_some())
            .unwrap();
        assert!(matches!(
            first_keyed_pass.kind,
            RuntimePassKind::CanonicalizeCapture
        ));
        // The first reconstructed program samples its one capture source, so
        // exact cache-key comparison reaches one sampler binding immediately.
        assert_eq!(first_keyed_pass.reads.len(), 1);
        assert_eq!(
            first_keyed_pass
                .cache_keys
                .as_ref()
                .unwrap()
                .samplers()
                .len(),
            1
        );
        let control =
            ClosedExecutableGraph::try_from_lowered(plan.clone(), &Account::for_test()).unwrap();
        assert!(
            control.is_ok(),
            "the uncapped authored color graph remains closed-eligible"
        );
        let (result, report) = measure_proof_work(RenderWork::ShaderBindingVisits, 0, |account| {
            ClosedExecutableGraph::try_from_lowered(plan, account).map(|closed| closed.is_ok())
        });
        assert!(
            result.is_err(),
            "the first reconstructed sampler comparison must be a typed refusal, not graph eligibility"
        );
        assert_eq!(
            report.attempt().unwrap().status(),
            crate::RenderAttemptStatus::Refused
        );
        assert_refusal(&report, RenderWork::ShaderBindingVisits, 0);
    }

    #[test]
    fn repeated_proof_color_operations_are_counted_and_one_under_refuses() {
        let plan = color_plan();
        let pass = plan
            .passes
            .iter()
            .find(|pass| matches!(pass.kind, RuntimePassKind::ColorFilter(Some(_))))
            .unwrap();
        let RuntimePassKind::ColorFilter(Some(filter)) = &pass.kind else {
            unreachable!()
        };
        assert_eq!(filter.operations.len(), 3);
        let source = plan
            .resources
            .iter()
            .find(|resource| resource.id == pass.reads[0].resource)
            .unwrap();
        let RuntimeResourceProducer::Pass(producer) = source.producer else {
            unreachable!()
        };
        for limit in [3, 2] {
            let entries = resource_entries(&plan);
            let (result, report) =
                measure_proof_work(RenderWork::FilterOperationVisits, limit, |account| {
                    let proof = ProofContext::new(account);
                    let resources = ProofMap {
                        live: entries.len(),
                        entries,
                        proof: &proof,
                    };
                    let fact = validate_closed_color_filter(
                        pass,
                        ExecutableCompositionContext {
                            current: source.id,
                            producer,
                            contains_captured_source: true,
                        },
                        source,
                        filter,
                        &resources,
                        plan.working_format,
                    );
                    proof.finish()?;
                    Ok(fact.map(|fact| fact.pass))
                });
            if limit == 3 {
                assert_eq!(result.unwrap(), Some(pass.id));
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::FilterOperationVisits),
                    3
                );
            } else {
                assert!(
                    result.is_err(),
                    "third operation must refuse before returning a closed fact"
                );
                assert_refusal(&report, RenderWork::FilterOperationVisits, 2);
            }
        }
    }

    #[test]
    fn repeated_proof_shadow_release_membership_counts_both_directions() {
        let plan = shadow_plan();
        let merge = plan.passes.iter().find(|pass| matches!(&pass.kind, RuntimePassKind::Composite(Some(composite)) if matches!(composite.kind, RuntimeCompositeKind::DropShadow))).unwrap();
        assert_eq!(merge.releases.len(), 2);
        let source = plan
            .resources
            .iter()
            .find(|resource| resource.id == merge.reads[0].resource)
            .unwrap();
        let shadow = plan
            .resources
            .iter()
            .find(|resource| resource.id == merge.reads[1].resource)
            .unwrap();
        let RuntimeResourceProducer::Pass(source_producer) = source.producer else {
            unreachable!()
        };
        let RuntimeResourceProducer::Pass(colorize_id) = shadow.producer else {
            unreachable!()
        };
        let colorize = plan
            .passes
            .iter()
            .find(|pass| pass.id == colorize_id)
            .unwrap();
        for limit in [14, 13] {
            let entries = resource_entries(&plan);
            let (result, report) =
                measure_proof_work(RenderWork::GraphEdgeVisits, limit, |account| {
                    let proof = ProofContext::new(account);
                    let resources = ProofMap {
                        live: entries.len(),
                        entries,
                        proof: &proof,
                    };
                    let result = validate_closed_shadow_merge(
                        merge,
                        ClosedShadowMergeInputs {
                            source_producer,
                            source,
                            colorize,
                            shadow,
                            result_spatial: shadow.spatial,
                        },
                        &resources,
                        plan.working_format,
                    );
                    proof.finish()?;
                    Ok(result.map(|resource| resource.id))
                });
            if limit == 14 {
                let RuntimeResultBinding::Resource(expected) = merge.result else {
                    unreachable!()
                };
                assert_eq!(result.unwrap(), Some(expected));
                // Two dependency comparisons, two read checks, four outer
                // release relationships, and six membership candidates.
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::GraphEdgeVisits),
                    14
                );
            } else {
                assert!(result.is_err(), "final membership candidate must refuse");
                assert_refusal(&report, RenderWork::GraphEdgeVisits, 13);
            }
        }
    }

    #[test]
    fn repeated_proof_color_filter_counts_dependency_read_and_release_edges() {
        let plan = color_plan();
        let pass = plan
            .passes
            .iter()
            .find(|pass| matches!(pass.kind, RuntimePassKind::ColorFilter(Some(_))))
            .unwrap();
        let RuntimePassKind::ColorFilter(Some(filter)) = &pass.kind else {
            unreachable!()
        };
        let source = plan
            .resources
            .iter()
            .find(|resource| resource.id == pass.reads[0].resource)
            .unwrap();
        let RuntimeResourceProducer::Pass(producer) = source.producer else {
            unreachable!()
        };
        for limit in [3, 2] {
            let entries = resource_entries(&plan);
            let (result, report) =
                measure_proof_work(RenderWork::GraphEdgeVisits, limit, |account| {
                    let proof = ProofContext::new(account);
                    let resources = ProofMap {
                        live: entries.len(),
                        entries,
                        proof: &proof,
                    };
                    let fact = validate_closed_color_filter(
                        pass,
                        ExecutableCompositionContext {
                            current: source.id,
                            producer,
                            contains_captured_source: true,
                        },
                        source,
                        filter,
                        &resources,
                        plan.working_format,
                    );
                    proof.finish()?;
                    Ok(fact.map(|fact| fact.pass))
                });
            if limit == 3 {
                assert_eq!(result.unwrap(), Some(pass.id));
                // The complete routine compares one dependency identity,
                // validates one read, then compares one release identity.
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::GraphEdgeVisits),
                    3
                );
            } else {
                assert!(
                    result.is_err(),
                    "the release identity must refuse before a closed fact returns"
                );
                assert_refusal(&report, RenderWork::GraphEdgeVisits, 2);
            }
        }
    }

    #[test]
    fn repeated_proof_two_key_sort_and_duplicate_classification_are_counted() {
        let plan = color_plan();
        let first = plan.resources[0].id;
        let second = plan.resources[1].id;
        for keys in [[second, first], [first, first]] {
            for limit in [2, 1] {
                let (result, report) =
                    measure_proof_work(RenderWork::ResourceProbes, limit, |account| {
                        let proof = ProofContext::new(account);
                        let map = ProofMap::<_, ()>::new(keys, 2, &proof)?;
                        proof.finish()?;
                        Ok((map.entries[0].0, map.entries[1].0, proof.invalid.get()))
                    });
                if limit == 2 {
                    let (a, b, duplicate) = result.unwrap();
                    assert!(a <= b);
                    assert_eq!(duplicate, keys[0] == keys[1]);
                    // Sorting two keys compares one pair; duplicate detection
                    // then examines that one adjacent pair.
                    assert_eq!(
                        report
                            .attempt()
                            .unwrap()
                            .usage()
                            .work(RenderWork::ResourceProbes),
                        2
                    );
                } else {
                    assert!(
                        result.is_err(),
                        "adjacent-key inspection must refuse before a map returns"
                    );
                    assert_refusal(&report, RenderWork::ResourceProbes, 1);
                }
            }
        }
    }

    #[test]
    fn repeated_proof_clip_coverage_counts_two_paired_relationships() {
        let plan = lower_capture_proof_fixture(
            1,
            true,
            &crate::backend::DeviceCapabilities::from_test_facts(true, true, 4096),
        );
        let closed = ClosedExecutableGraph::try_from_lowered(plan, &Account::for_test())
            .unwrap()
            .unwrap_or_else(|_| panic!("valid clipped composition fixture"));
        let layer = closed
            .facts
            .layer_compositions
            .iter()
            .find(|layer| layer.clip_coverage.is_some())
            .unwrap();
        let capture = closed
            .facts
            .captures
            .iter()
            .find(|capture| Some(capture.target()) == layer.clip_coverage)
            .unwrap();
        assert_eq!(capture.work().clip_coverage().unwrap().elements.len(), 2);
        for limit in [2, 1] {
            let (result, report) =
                measure_proof_work(RenderWork::GraphEdgeVisits, limit, |account| {
                    let proof = ProofContext::new(account);
                    let exact = layer_has_exact_clip_coverage_capture(
                        layer,
                        &closed.facts.captures,
                        &proof,
                    );
                    proof.finish()?;
                    Ok(exact)
                });
            if limit == 2 {
                assert!(result.unwrap());
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::GraphEdgeVisits),
                    2
                );
            } else {
                assert!(result.is_err(), "second clip relationship must refuse");
                assert_refusal(&report, RenderWork::GraphEdgeVisits, 1);
            }
        }
    }

    fn independent_path_clip(second_x: f64) -> RenderClip {
        let setup = Account::for_test();
        let mut path = crate::Path::new();
        path.move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(second_x, 0.0))
            .line_to(Point::new(0.0, 2.0))
            .close();
        let filled = crate::FilledPath::try_new(path, crate::FillRule::NonZero).unwrap();
        let clip = crate::ClipInput::try_filled_path(filled).unwrap();
        let mut scene = crate::Scene::new();
        scene.layer(crate::Layer::new().try_clip_input(clip).unwrap(), |child| {
            child.fill(crate::Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        });
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, &setup)
            .unwrap();
        let crate::command::RenderCommand::Layer { layer, .. } = &commands[0] else {
            unreachable!()
        };
        layer.clip.clone().unwrap()
    }

    #[test]
    fn repeated_proof_independent_equal_clip_paths_count_the_reached_element_pairs() {
        let plan = lower_capture_proof_fixture(
            1,
            true,
            &crate::backend::DeviceCapabilities::from_test_facts(true, true, 4096),
        );
        let closed = ClosedExecutableGraph::try_from_lowered(plan, &Account::for_test())
            .unwrap()
            .unwrap_or_else(|_| panic!("valid clipped composition fixture"));
        let mut layer = closed
            .facts
            .layer_compositions
            .iter()
            .find(|layer| layer.clip_coverage.is_some())
            .unwrap()
            .clone();
        let mut capture = closed
            .facts
            .captures
            .iter()
            .find(|capture| Some(capture.target()) == layer.clip_coverage)
            .unwrap()
            .clone();
        let outer = independent_path_clip(2.0);
        let RuntimeCompositeKind::Layer {
            clip, outer_clips, ..
        } = &mut layer.composite.kind
        else {
            unreachable!()
        };
        *clip = None;
        *outer_clips = shared(&[RuntimeOuterClip {
            clip: outer.clone(),
            transform: Transform::identity(),
        }]);
        for (second_x, expected_visits, expected_equal) in [(2.0, 4, true), (1.5, 2, false)] {
            let element_clip = independent_path_clip(second_x);
            let (
                crate::command::RenderClipGeometry::Path { path: left, .. },
                crate::command::RenderClipGeometry::Path { path: right, .. },
            ) = (outer.geometry(), element_clip.geometry())
            else {
                unreachable!()
            };
            assert!(
                !std::ptr::eq::<crate::Path>(&**left, &**right),
                "the equality fallback compares independent allocations"
            );
            capture.work = RuntimeVelloCapture::ClipCoverage(RuntimeClipCoverage {
                elements: shared(&[super::super::model::RuntimeClipCoverageElement {
                    clip: element_clip,
                    transform: Transform::identity(),
                }]),
                antialiasing: Antialiasing::Area,
            });
            for limit in [expected_visits, expected_visits - 1] {
                let (result, report) =
                    measure_proof_work(RenderWork::PathElementVisits, limit, |account| {
                        let proof = ProofContext::new(account);
                        let exact = layer_has_exact_clip_coverage_capture(
                            &layer,
                            std::slice::from_ref(&capture),
                            &proof,
                        );
                        proof.finish()?;
                        Ok(exact)
                    });
                if limit == expected_visits {
                    assert_eq!(result.unwrap(), expected_equal);
                    assert_eq!(
                        report
                            .attempt()
                            .unwrap()
                            .usage()
                            .work(RenderWork::PathElementVisits),
                        expected_visits
                    );
                } else {
                    assert!(
                        result.is_err(),
                        "the next path pair must refuse before content equality returns"
                    );
                    assert_refusal(&report, RenderWork::PathElementVisits, limit);
                }
            }
        }
    }

    #[test]
    fn repeated_proof_outer_capture_facts_are_distinct_from_inner_pass_visits() {
        let plan = lower_capture_proof_fixture(
            3,
            false,
            &crate::backend::DeviceCapabilities::from_test_facts(true, true, 4096),
        );
        let closed = ClosedExecutableGraph::try_from_lowered(plan, &Account::for_test())
            .unwrap()
            .unwrap_or_else(|_| panic!("valid three-capture composition fixture"));
        assert_eq!(closed.facts.captures.len(), 3);
        let mut facts = (*closed.facts).clone();
        facts.layer_compositions = shared(&[]);
        facts.color_filters = shared(&[]);
        facts.blurs = shared(&[]);
        facts.drop_shadows = shared(&[]);
        facts.filter_steps = shared(&[]);
        facts.backdrops = shared(&[]);
        let mut plan = closed.lowered.clone();
        let passes = plan
            .passes
            .iter()
            .filter(|pass| matches!(pass.kind, RuntimePassKind::VelloCapture(Some(_))))
            .take(2)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(passes.len(), 2);
        plan.passes = shared(&passes);
        for limit in [16, 15] {
            let (result, report) =
                measure_proof_work(RenderWork::GraphNodeVisits, limit, |account| {
                    facts.proves_exact_facts_for(&plan, account)
                });
            if limit == 16 {
                assert!(!result.unwrap(), "the third capture has no matching pass");
                // Three outer capture facts, five inner capture-pass probes
                // (1 + 2 + 2), three two-pass kind censuses, and two ordered passes.
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::GraphNodeVisits),
                    16
                );
            } else {
                assert!(
                    result.is_err(),
                    "last permitted pass inspection must not hide outer fact work"
                );
                assert_refusal(&report, RenderWork::GraphNodeVisits, 15);
            }
        }
    }
}

fn validate_closed_composite(
    pass: &RuntimePass,
    parent: ExecutableCompositionContext,
    source: &RuntimeResourceRequest,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
    working_format: WorkingFormat,
    requires_isolated_source: bool,
) -> Option<Option<ExecutableLayerCompositionFacts>> {
    validate_closed_composite_with_parent_reads(
        pass,
        parent,
        source,
        resources,
        working_format,
        requires_isolated_source,
        1,
    )
}

fn runtime_composite_is_backdrop_inner(composite: &RuntimeComposite) -> bool {
    let RuntimeCompositeKind::Layer {
        transform,
        parameters,
        outer_clips,
        ..
    } = &composite.kind
    else {
        return false;
    };
    *transform == Transform::identity()
        && parameters.destination_to_layer_local().affine() == Transform::identity()
        && parameters.opacity() == 1.0
        && parameters.composition().is_source_over_normal()
        && outer_clips.is_empty()
        && parameters.alpha_mask().is_none()
}

fn runtime_composite_is_untransformed_outer(composite: &RuntimeComposite) -> bool {
    let RuntimeCompositeKind::Layer {
        transform,
        parameters,
        ..
    } = &composite.kind
    else {
        return false;
    };
    *transform == Transform::identity()
        && parameters.destination_to_layer_local().affine() == Transform::identity()
}

fn validate_closed_composite_with_parent_reads(
    pass: &RuntimePass,
    parent: ExecutableCompositionContext,
    source: &RuntimeResourceRequest,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
    working_format: WorkingFormat,
    requires_isolated_source: bool,
    parent_expected_reads: u32,
) -> Option<Option<ExecutableLayerCompositionFacts>> {
    let RuntimePassKind::Composite(Some(composite)) = &pass.kind else {
        return None;
    };
    let result = validate_closed_composite_base(
        pass,
        parent,
        source,
        resources,
        working_format,
        composite,
        parent_expected_reads,
    )?;

    match &composite.kind {
        RuntimeCompositeKind::SpanSourceOver => {
            if requires_isolated_source
                || pass.reads.len() != 2
                || !matches!(
                    source.role,
                    RuntimeResourceRole::FilterIntermediate
                        | RuntimeResourceRole::CompositeResult
                        | RuntimeResourceRole::IsolationWorkingImage
                )
            {
                return None;
            }
            Some(None)
        }
        RuntimeCompositeKind::Layer {
            transform,
            parameters,
            clip,
            outer_clips,
            clip_coverage,
        } => {
            let layer = ClosedLayerCompositeView {
                composite,
                transform,
                parameters,
                clip: clip.as_deref(),
                outer_clips,
                clip_coverage: *clip_coverage,
            };
            validate_closed_layer_composite(
                pass,
                parent,
                source,
                resources,
                requires_isolated_source,
                result,
                layer,
            )
        }
        RuntimeCompositeKind::DropShadow => None,
    }
}

fn validate_closed_composite_base(
    pass: &RuntimePass,
    parent: ExecutableCompositionContext,
    source: &RuntimeResourceRequest,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
    working_format: WorkingFormat,
    composite: &RuntimeComposite,
    parent_expected_reads: u32,
) -> Option<RuntimeResourceId> {
    if !composite.source_captured_before_outer_semantics || source.id == parent.current {
        return None;
    }
    let parent_resource = resources.get(&parent.current).copied()?;
    if let RuntimeCompositeKind::Layer { parameters, .. } = &composite.kind
        && parameters.source_space() == crate::frame::CompositeSourceSpace::SurfaceGrid
        && source.spatial.raster_scale != parent_resource.spatial.raster_scale
    {
        return None;
    }
    let RuntimeResourceProducer::Pass(source_producer) = source.producer else {
        return None;
    };
    let coverage_producer = if let RuntimeCompositeKind::Layer {
        clip_coverage: Some(coverage),
        ..
    } = &composite.kind
    {
        let coverage_resource = resources.get(coverage).copied()?;
        let RuntimeResourceProducer::Pass(producer) = coverage_resource.producer else {
            return None;
        };
        Some(producer)
    } else {
        None
    };
    let expected_dependencies = [
        Some(parent.producer),
        Some(source_producer),
        coverage_producer,
    ]
    .into_iter()
    .flatten();
    if !proof_relationships_equal(
        pass.dependencies.iter(),
        expected_dependencies,
        resources.proof,
        |actual, expected| *actual == expected,
    ) || pass.reads.len() < 2
        || !runtime_read_has_exact_facts(
            &pass.reads[0],
            RuntimeReadRole::CompositeParent,
            parent_resource,
            RuntimeSamplingFilter::Linear,
            RuntimeSamplingEdge::ClampToExtent,
            resources.proof,
        )
        || !runtime_read_has_exact_facts(
            &pass.reads[1],
            RuntimeReadRole::CompositeSource,
            source,
            RuntimeSamplingFilter::Linear,
            RuntimeSamplingEdge::TransparentBlack,
            resources.proof,
        )
        || parent_resource.format != RuntimeResourceFormat::Working(working_format)
        || source.format != RuntimeResourceFormat::Working(working_format)
        || !matches!(
            parent_resource.role,
            RuntimeResourceRole::RootWorkingImage
                | RuntimeResourceRole::IsolationWorkingImage
                | RuntimeResourceRole::CompositeResult
        )
        || parent_resource.expected_reads != parent_expected_reads
        || parent_resource.last_use != pass.id
        || source.expected_reads != 1
        || source.last_use != pass.id
    {
        return None;
    }
    let RuntimeResultBinding::Resource(result) = pass.result else {
        return None;
    };
    let result_resource = resources.get(&result).copied()?;
    base_graph_resource_has_fixed_facts(
        result_resource,
        RuntimeResourceRole::CompositeResult,
        RuntimeResourceFormat::Working(working_format),
        RuntimeResourceProducer::Pass(pass.id),
    )
    .then_some(())
    .filter(|()| result_resource.spatial == parent_resource.spatial)
    .map(|()| result)
}

struct ClosedLayerCompositeView<'a> {
    composite: &'a RuntimeComposite,
    transform: &'a Transform,
    parameters: &'a RuntimeLayerCompositeParameters,
    clip: Option<&'a RenderClip>,
    outer_clips: &'a [RuntimeOuterClip],
    clip_coverage: Option<RuntimeResourceId>,
}

fn validate_closed_layer_composite(
    pass: &RuntimePass,
    parent: ExecutableCompositionContext,
    source: &RuntimeResourceRequest,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
    requires_isolated_source: bool,
    result: RuntimeResourceId,
    layer: ClosedLayerCompositeView<'_>,
) -> Option<Option<ExecutableLayerCompositionFacts>> {
    let destination_to_layer_local: RuntimeDestinationToLayerLocal =
        layer.parameters.destination_to_layer_local();
    let opacity = layer.parameters.opacity();
    let alpha_mask = layer.parameters.alpha_mask();
    if layer
        .transform
        .as_array()
        .iter()
        .any(|value| !value.is_finite())
        || !opacity.is_finite()
        || !(0.0..=1.0).contains(&opacity)
        || !runtime_affine_is_finite_and_non_singular(destination_to_layer_local.affine())
        || !layer.outer_clips.iter().all(|clip| {
            resources.proof.work(RenderWork::GraphEdgeVisits, 1)
                && clip
                    .transform
                    .as_array()
                    .iter()
                    .all(|value| value.is_finite())
        })
        || layer.parameters.has_clip() != (layer.clip.is_some() || !layer.outer_clips.is_empty())
        || layer.parameters.has_clip() != layer.clip_coverage.is_some()
        || !closed_composite_source_is_valid(source, requires_isolated_source, &layer)
    {
        return None;
    }
    match (layer.clip_coverage, alpha_mask, pass.reads.as_slice()) {
        (None, None, [_, _]) => {}
        (Some(coverage), None, [_, _, coverage_read]) => {
            validate_closed_clip_coverage(pass.id, resources, coverage, coverage_read)?;
        }
        (None, Some(mask), [_, _, mask_read]) => {
            validate_closed_alpha_mask(pass.id, resources, mask, mask_read)?;
        }
        (Some(coverage), Some(mask), [_, _, coverage_read, mask_read]) => {
            validate_closed_clip_coverage(pass.id, resources, coverage, coverage_read)?;
            validate_closed_alpha_mask(pass.id, resources, mask, mask_read)?;
        }
        _ => return None,
    }
    Some(Some(ExecutableLayerCompositionFacts {
        pass: pass.id,
        parent: parent.current,
        source: source.id,
        clip_coverage: layer.clip_coverage,
        alpha_mask: alpha_mask.map(RuntimeResolvedAlphaMaskComposition::resource),
        result,
        composite: layer.composite.clone(),
    }))
}

fn closed_composite_source_is_valid(
    source: &RuntimeResourceRequest,
    requires_isolated_source: bool,
    layer: &ClosedLayerCompositeView<'_>,
) -> bool {
    (requires_isolated_source
        && matches!(
            source.role,
            RuntimeResourceRole::CompositeResult
                | RuntimeResourceRole::FilterIntermediate
                | RuntimeResourceRole::IsolationWorkingImage
        ))
        || (!requires_isolated_source
            && source.role == RuntimeResourceRole::FilterIntermediate
            && *layer.transform == Transform::identity()
            && layer.parameters.opacity() == 1.0
            && layer.parameters.composition().is_source_over_normal()
            && layer.clip.is_none()
            && !layer.outer_clips.is_empty()
            && layer.parameters.alpha_mask().is_none())
}

fn validate_closed_clip_coverage(
    pass: RuntimePassId,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
    coverage: RuntimeResourceId,
    read: &RuntimeReadBinding,
) -> Option<()> {
    let coverage_resource = resources.get(&coverage).copied()?;
    if !runtime_read_has_exact_facts(
        read,
        RuntimeReadRole::ClipCoverage,
        coverage_resource,
        RuntimeSamplingFilter::Linear,
        RuntimeSamplingEdge::TransparentBlack,
        resources.proof,
    ) || coverage_resource.role != RuntimeResourceRole::ClipCoverage
        || coverage_resource.format != RuntimeResourceFormat::ClipCoverageRgba8Unorm
        || !matches!(coverage_resource.producer, RuntimeResourceProducer::Pass(_))
        || coverage_resource.import.is_some()
        || coverage_resource.expected_reads != 1
        || coverage_resource.last_use != pass
    {
        return None;
    }
    Some(())
}

fn validate_closed_alpha_mask(
    pass: RuntimePassId,
    resources: &ProofMap<'_, RuntimeResourceId, &RuntimeResourceRequest>,
    mask: RuntimeResolvedAlphaMaskComposition,
    read: &RuntimeReadBinding,
) -> Option<()> {
    let mask_resource = resources.get(&mask.resource()).copied()?;
    let Some(RuntimeResourceImport::ResolvedAlphaMask(upload)) = &mask_resource.import else {
        return None;
    };
    let mask_filter = match mask.sampling().quality() {
        ShaderMaskQualityKey::Low => RuntimeSamplingFilter::Nearest,
        ShaderMaskQualityKey::Medium
        | ShaderMaskQualityKey::High
        | ShaderMaskQualityKey::Pixelated => RuntimeSamplingFilter::Linear,
    };
    (runtime_read_has_exact_facts(
        read,
        RuntimeReadRole::AlphaMask,
        mask_resource,
        mask_filter,
        RuntimeSamplingEdge::ClampToExtent,
        resources.proof,
    ) && mask_resource.role == RuntimeResourceRole::ImportedImage
        && mask_resource.format == RuntimeResourceFormat::ResolvedMaskRgba8Unorm
        && mask_resource.producer == RuntimeResourceProducer::Imported
        && upload.physical_size() == mask.image_dimensions()
        && mask_resource.spatial.device_extent == mask.image_dimensions()
        && mask.sampling()
            == ShaderMaskSamplingKey::with_rendering(
                upload.quality(),
                upload.rendering_hint(),
                upload.extend(),
            )
        && RuntimeMaskTexelCenterFacts::try_new(mask.image_dimensions()).ok()
            == Some(mask.texel_center_facts())
        && mask_resource.expected_reads != 0
        && mask_resource.last_use >= pass)
        .then_some(())
}

pub(super) fn base_graph_resource_has_fixed_facts(
    resource: &RuntimeResourceRequest,
    role: RuntimeResourceRole,
    format: RuntimeResourceFormat,
    producer: RuntimeResourceProducer,
) -> bool {
    resource.role == role
        && resource.format == format
        && resource.producer == producer
        && resource.import.is_none()
}

pub(super) fn executable_vello_capture_facts(
    pass: RuntimePassId,
    target: RuntimeResourceId,
    work: &RuntimeVelloCapture,
    spatial: RuntimeSpatialDescriptor,
    account: &Account,
) -> Result<Option<ExecutableVelloCaptureFacts>> {
    let valid_work = match work {
        RuntimeVelloCapture::Span(span) => {
            !span.commands.commands.is_empty() && span.captured_before_outer_semantics
        }
        RuntimeVelloCapture::ClipCoverage(coverage) => clip_coverage_is_closed(coverage, account)?,
    };
    Ok((|| {
        if !valid_work
            || spatial.device_extent.width() == 0
            || spatial.device_extent.height() == 0
            || !spatial.texel_origin.x().is_finite()
            || !spatial.texel_origin.y().is_finite()
            || !spatial.raster_scale.is_finite()
            || spatial.raster_scale <= 0.0
        {
            return None;
        }
        let expected_device_x = spatial.texel_origin.x() * spatial.raster_scale;
        let expected_device_y = spatial.texel_origin.y() * spatial.raster_scale;
        let tolerance = f64::EPSILON
            * spatial
                .raster_scale
                .abs()
                .max(expected_device_x.abs())
                .max(expected_device_y.abs())
                .max(1.0)
            * 8.0;
        if (expected_device_x - f64::from(spatial.device_origin.0)).abs() > tolerance
            || (expected_device_y - f64::from(spatial.device_origin.1)).abs() > tolerance
        {
            return None;
        }
        let grid_transform =
            Transform::translation(-spatial.texel_origin.x(), -spatial.texel_origin.y())
                .ok()?
                .then(Transform::scale(spatial.raster_scale, spatial.raster_scale).ok()?)
                .ok()?;
        let initial_transform = match work {
            RuntimeVelloCapture::Span(span) => match span.source_space {
                crate::frame::CompositeSourceSpace::SurfaceGrid => {
                    if span.parent_to_surface != Transform::identity() {
                        return None;
                    }
                    span.capture_transform.then(grid_transform).ok()?
                }
                crate::frame::CompositeSourceSpace::LayerLocal => match span.scope {
                    RuntimeVelloSpanScope::CurrentParent => span
                        .capture_transform
                        .then(span.parent_to_surface)
                        .ok()?
                        .then(grid_transform)
                        .ok()?,
                    RuntimeVelloSpanScope::LayerSource => {
                        span.capture_transform.then(grid_transform).ok()?
                    }
                },
            },
            RuntimeVelloCapture::ClipCoverage(_) => grid_transform,
        };
        Some(ExecutableVelloCaptureFacts {
            pass,
            target,
            work: work.clone(),
            initial_transform,
            antialiasing: work.antialiasing(),
            target_extent: spatial.device_extent,
            texel_origin: spatial.texel_origin,
            raster_scale: spatial.raster_scale,
        })
    })())
}

fn clip_coverage_is_closed(coverage: &RuntimeClipCoverage, account: &Account) -> Result<bool> {
    if coverage.elements.is_empty() {
        return Ok(false);
    }
    for element in coverage.elements.iter() {
        account.work(RenderWork::GraphEdgeVisits, 1)?;
        if !element
            .transform
            .as_array()
            .iter()
            .all(|value| value.is_finite())
        {
            return Ok(false);
        }
    }
    Ok(true)
}
pub(super) fn preparation_error(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DispatchPassSemantics {
    ClosedExecutable,
    FuturePass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GraphPreparationIneligibility {
    OutsideClosedExecutableGraph,
}

impl GraphPreparationIneligibility {
    pub(super) fn into_error(self) -> Error {
        match self {
            Self::OutsideClosedExecutableGraph => preparation_error(
                "a graph outside the closed executable subset cannot enter runtime preparation",
            ),
        }
    }
}

pub(super) enum PrePreparationGraphClassification {
    ExactBase(BasePreparableGraph),
    ExactComposition(ClosedExecutableGraph),
    ExactColorFilter(ColorFilterPreparableGraph),
    ExactSpatialFilter(SpatialFilterPreparableGraph),
    ExactBackdrop(BackdropPreparableGraph),
    FuturePasses,
    Ineligible(GraphPreparationIneligibility),
}
impl PrePreparationGraphClassification {
    pub(super) fn classify(lowered: LoweredGraphPlan, account: &Account) -> Result<Self> {
        let closed = match ClosedExecutableGraph::try_from_lowered(lowered, account)? {
            Ok(closed) => closed,
            Err(lowered) => {
                let mut contains_future_passes = false;
                for pass in &lowered.passes {
                    account.work(RenderWork::GraphNodeVisits, 1)?;
                    match dispatch_pass_semantics(&pass.kind, account)? {
                        Some(DispatchPassSemantics::ClosedExecutable) => {}
                        Some(DispatchPassSemantics::FuturePass) => {
                            contains_future_passes = true;
                        }
                        None => {
                            return Ok(Self::Ineligible(
                                GraphPreparationIneligibility::OutsideClosedExecutableGraph,
                            ));
                        }
                    }
                }
                return Ok(if contains_future_passes {
                    Self::FuturePasses
                } else {
                    Self::Ineligible(GraphPreparationIneligibility::OutsideClosedExecutableGraph)
                });
            }
        };
        Ok(
            match BasePreparableGraph::try_from_closed(closed, account)? {
                Ok(preparable) => Self::ExactBase(preparable),
                Err(closed) => match BackdropPreparableGraph::try_from_closed(closed, account)? {
                    Ok(preparable) => Self::ExactBackdrop(preparable),
                    Err(closed) => match SpatialFilterPreparableGraph::try_from_closed(closed) {
                        Ok(preparable) => Self::ExactSpatialFilter(preparable),
                        Err(closed) => match ColorFilterPreparableGraph::try_from_closed(closed) {
                            Ok(preparable) => Self::ExactColorFilter(preparable),
                            Err(closed) => {
                                match CompositionPreparableGraph::try_from_closed(closed) {
                                    Ok(preparable) => {
                                        Self::ExactComposition(preparable.into_closed())
                                    }
                                    Err(_) => Self::Ineligible(
                                        GraphPreparationIneligibility::OutsideClosedExecutableGraph,
                                    ),
                                }
                            }
                        },
                    },
                },
            },
        )
    }
}

fn dispatch_pass_semantics(
    kind: &RuntimePassKind,
    account: &Account,
) -> Result<Option<DispatchPassSemantics>> {
    Ok(match kind {
        RuntimePassKind::ClearRoot {
            initialization: RuntimeInitialization::SurfaceBaseColor,
            ..
        }
        | RuntimePassKind::CanonicalizeCapture
        | RuntimePassKind::Present => Some(DispatchPassSemantics::ClosedExecutable),
        RuntimePassKind::ClearRoot {
            initialization: RuntimeInitialization::Transparent,
            color,
        } if *color == Color::TRANSPARENT => Some(DispatchPassSemantics::ClosedExecutable),
        RuntimePassKind::ClearRoot {
            initialization: RuntimeInitialization::Transparent,
            ..
        } => None,
        RuntimePassKind::VelloCapture(Some(RuntimeVelloCapture::Span(span)))
            if !span.commands.commands.is_empty()
                && span.captured_before_outer_semantics
                && span
                    .capture_transform
                    .as_array()
                    .iter()
                    .all(|value| value.is_finite())
                && span
                    .parent_to_surface
                    .as_array()
                    .iter()
                    .all(|value| value.is_finite()) =>
        {
            Some(DispatchPassSemantics::ClosedExecutable)
        }
        RuntimePassKind::VelloCapture(Some(RuntimeVelloCapture::ClipCoverage(coverage))) => {
            clip_coverage_is_closed(coverage, account)?
                .then_some(DispatchPassSemantics::ClosedExecutable)
        }
        RuntimePassKind::VelloCapture(_) => None,
        RuntimePassKind::CopyBackdrop
        | RuntimePassKind::ColorFilter(Some(_))
        | RuntimePassKind::DropShadowColorize(Some(_)) => Some(DispatchPassSemantics::FuturePass),
        RuntimePassKind::BlurHorizontal(Some(blur)) if blur.axis == RuntimeBlurAxis::Horizontal => {
            Some(DispatchPassSemantics::FuturePass)
        }
        RuntimePassKind::BlurVertical(Some(blur)) if blur.axis == RuntimeBlurAxis::Vertical => {
            Some(DispatchPassSemantics::FuturePass)
        }
        RuntimePassKind::ColorFilter(None)
        | RuntimePassKind::BlurHorizontal(_)
        | RuntimePassKind::BlurVertical(_)
        | RuntimePassKind::DropShadowColorize(None)
        | RuntimePassKind::Composite(None) => None,
        RuntimePassKind::Composite(Some(composite))
            if composite.source_captured_before_outer_semantics =>
        {
            Some(match composite.kind {
                RuntimeCompositeKind::SpanSourceOver | RuntimeCompositeKind::Layer { .. } => {
                    DispatchPassSemantics::ClosedExecutable
                }
                RuntimeCompositeKind::DropShadow => DispatchPassSemantics::FuturePass,
            })
        }
        RuntimePassKind::Composite(Some(_)) => None,
    })
}

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus,
        RenderLimits,
    };

    #[test]
    fn proof_map_refuses_a_key_without_reserved_backing() {
        use crate::accounting::StorageContractError;
        use crate::{RenderAccountingError, RenderPhase, RenderQuantity, RenderRefusalReason};

        for limit in [0, 1] {
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::GraphNodeVisits, limit)
                .unwrap();
            let mut context =
                RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
            let error = context
                .run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                    let proof = ProofContext::new(account);
                    ProofMap::<u32, ()>::new([1], 0, &proof).map(drop)
                })
                .unwrap_err();
            let attempt = context.last_attempt().unwrap().attempt().unwrap();
            if limit == 0 {
                let refusal = attempt.refusal().unwrap();
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(
                    refusal.quantity(),
                    RenderQuantity::Work(RenderWork::GraphNodeVisits)
                );
                assert_eq!((refusal.current(), refusal.amount()), (0, 1));
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(refusal))
                );
            } else {
                assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
                assert_eq!(error.accounting_diagnostic(), None);
                assert_eq!(
                    std::error::Error::source(&error).and_then(|source| source.downcast_ref()),
                    Some(&StorageContractError::ReservedCapacity {
                        needed: 1,
                        capacity: 0
                    }),
                );
                assert_eq!(attempt.refusal(), None);
            }
            assert_eq!(attempt.usage().work(RenderWork::GraphNodeVisits), limit);
            assert_eq!(attempt.usage().work(RenderWork::ResourceProbes), 0);
            assert_eq!(attempt.usage().live(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(attempt.usage().peak(RenderResource::ScratchCpuBytes), 0);
        }
    }

    #[test]
    fn proof_set_epoch_exhaustion_preserves_membership_and_first_error() {
        use crate::{RenderAccountingError, RenderPhase, RenderQuantity, RenderRefusalReason};

        for prior_refusal in [false, true] {
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::ResourceProbes, 1)
                .unwrap();
            let mut context =
                RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
            let error = context
                .run(RenderAttemptKind::Frame, RenderPhase::Lower, |account| {
                    let proof = ProofContext::new(account);
                    let mut set = ProofSet::new([1_u32], 1, &proof)?;
                    set.epoch = u64::MAX;
                    assert!(set.insert(1));
                    if prior_refusal {
                        assert!(!set.contains(&1));
                    }
                    let work = || {
                        [
                            RenderWork::GraphNodeVisits,
                            RenderWork::GraphEdgeVisits,
                            RenderWork::ResourceProbes,
                        ]
                        .map(|dimension| account.work_count(dimension).unwrap())
                    };
                    assert_eq!(work(), [1, 0, 1]);
                    set.clear();
                    assert_eq!(work(), [1, 0, 1]);
                    assert_eq!(set.epoch, u64::MAX);
                    assert_eq!(set.len(), 1);
                    assert_eq!(set.slots.entries.as_slice(), &[(1, Some(u64::MAX))]);
                    assert!(set.iter().copied().eq([1]));
                    assert!(!proof.invalid.get());
                    proof.finish()
                })
                .unwrap_err();
            let attempt = context.last_attempt().unwrap().attempt().unwrap();
            if prior_refusal {
                let refusal = attempt.refusal().unwrap();
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(
                    refusal.quantity(),
                    RenderQuantity::Work(RenderWork::ResourceProbes)
                );
                assert_eq!((refusal.current(), refusal.amount()), (1, 1));
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 1 });
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(refusal))
                );
            } else {
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Invariant)
                );
                assert_eq!(attempt.refusal(), None);
            }
        }
    }

    #[test]
    fn sorted_proof_slots_update_exact_values_without_growth_and_sets_reset_per_pass() {
        let mut context = RenderAccountingContext::default();
        context
            .run(
                RenderAttemptKind::Frame,
                crate::RenderPhase::Lower,
                |account| {
                    let proof = ProofContext::new(account);
                    let mut map = ProofMap::<u32, u64>::new([3, 1, 2], 3, &proof)?;
                    let capacity = map.entries.capacity();
                    assert_eq!(map.insert(2, 20), None);
                    assert_eq!(map.insert(1, 10), None);
                    assert_eq!(map.insert(2, 21), Some(20));
                    assert_eq!(map.get(&1), Some(&10));
                    assert_eq!(map.get(&2), Some(&21));
                    assert_eq!(map.get(&3), None);
                    assert_eq!(map.entries.capacity(), capacity);
                    assert_eq!(map.len(), 2);
                    let mut set = ProofSet::new([3, 1, 2], 3, &proof)?;
                    let set_capacity = set.slots.entries.capacity();
                    for _ in 0..16 {
                        assert!(set.insert(3));
                        assert!(!set.insert(3));
                        assert!(set.insert(1));
                        assert_eq!(set.iter().copied().collect::<Vec<_>>(), [1, 3]);
                        assert_eq!(set.len(), 2);
                        set.clear();
                        assert!(!set.contains(&3));
                        assert_eq!(set.len(), 0);
                    }
                    assert_eq!(set.slots.entries.capacity(), set_capacity);
                    assert!(!proof.invalid.get());
                    proof.finish()
                },
            )
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
    }

    #[test]
    fn proof_lookup_refusal_remains_a_typed_accounting_failure_instead_of_ineligibility() {
        let setup = Account::for_test();
        let setup_proof = ProofContext::new(&setup);
        let entries = ProofMap::<u32, u64>::new([1, 2], 2, &setup_proof)
            .unwrap()
            .entries;
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::ResourceProbes, 0)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::default().with_limits(limits));
        let result = context.run(
            RenderAttemptKind::Frame,
            crate::RenderPhase::Lower,
            |account| {
                let proof = ProofContext::new(account);
                let map = ProofMap {
                    entries,
                    proof: &proof,
                    live: 0,
                };
                let semantic_hit = map.get(&1).is_some();
                assert!(!semantic_hit);
                proof.finish()?;
                Ok(semantic_hit)
            },
        );
        assert!(result.is_err());
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.usage().work(RenderWork::ResourceProbes), 0);
        assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
    }

    #[test]
    fn proof_capacity_refusal_adds_no_accepted_peak_or_source_visit() {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::default()
                .with_limits(RenderLimits::new().with_scratch_bytes(0)),
        );
        let result = context.run(
            RenderAttemptKind::Frame,
            crate::RenderPhase::Lower,
            |account| {
                let proof = ProofContext::new(account);
                let _ = ProofMap::<u32, u64>::new([1, 2], 2, &proof)?;
                Ok(())
            },
        );
        assert!(result.is_err());
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.usage().peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(report.usage().work(RenderWork::GraphNodeVisits), 0);
    }
    #[test]
    fn sorted_lookup_admits_each_actual_candidate_and_stops_before_the_refused_comparison() {
        for (limit, succeeds) in [(3, false), (4, true)] {
            let setup = Account::for_test();
            let setup_proof = ProofContext::new(&setup);
            let entries = ProofMap::<u32, u64>::new([3, 1, 2], 3, &setup_proof)
                .unwrap()
                .entries;
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::ResourceProbes, limit)
                .unwrap();
            let mut context = RenderAccountingContext::new(
                RenderAccountingOptions::default().with_limits(limits),
            );
            let result = context.run(
                RenderAttemptKind::Frame,
                crate::RenderPhase::Lower,
                |account| {
                    let proof = ProofContext::new(account);
                    let map = ProofMap {
                        entries,
                        proof: &proof,
                        live: 0,
                    };
                    assert_eq!(map.position(&3), Some(2));
                    assert_eq!(map.position(&0), None);
                    proof.finish()
                },
            );
            assert_eq!(result.is_ok(), succeeds);
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::ResourceProbes),
                limit
            );
        }
    }
}

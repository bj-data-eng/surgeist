//! Invocation-local universal width-action evidence, without an execution log.
use super::{
    buffer::hb_buffer_t,
    components::{self, ProvenanceSet},
};
use crate::{ComponentCoverage, ComponentPath, FeatureProvenance, ShapingRefusal, Tag};
use alloc::vec::Vec;
use core::mem::size_of;

/// One registered OpenType width feature to observe, independently of enabling it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidthFeature {
    /// The `hwid` feature.
    HalfWidth,
    /// The `twid` feature.
    ThirdWidth,
    /// The `qwid` feature.
    QuarterWidth,
}
impl WidthFeature {
    /// Actual OpenType feature tag; no numeric resulting width is promised.
    pub const fn tag(self) -> Tag {
        match self {
            Self::HalfWidth => Tag::new(b"hwid"),
            Self::ThirdWidth => Tag::new(b"twid"),
            Self::QuarterWidth => Tag::new(b"qwid"),
        }
    }
}
/// An actual path that cannot establish complete width-action evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidthObservationPath {
    /// Native correspondence or positioning already has this qualification limit.
    Native(ComponentPath),
    /// An active positioning value/anchor adjustment could not be qualified.
    ValueRecord,
    /// An effectively reached lookup/reference or demanded lazy datum retained
    /// rejected native content, including recovery consumed by a native cache.
    LookupAdmission,
}
/// Completeness of the requested action observation, separate from shaping status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidthObservationCoverage {
    /// All surviving source paths and actual relevant actions were observed.
    Complete,
    /// The named executed path supplies no complete positive or negative proof.
    Unqualified(WidthObservationPath),
}
/// Diagnostic selected-feature summary for one original input scalar, not a CSS
/// unit. It proves availability/absence only under Complete observation coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidthInputOutcome {
    /// Qualified native removal/residual output leaves no ordinary operand.
    NoSurvivingOperand,
    /// Surviving ordinary ancestry contains no selected-feature action.
    NoAction,
    /// Some surviving paths contain an action and some remain uncovered.
    Partial,
    /// Every surviving ordinary ancestry path contains a successful action.
    Covered,
}
/// Borrowed observation tied to the actual completed output lifecycle.
pub struct WidthFeatureReceipt<'a> {
    buffer: &'a hb_buffer_t,
}
/// Opaque evidence for one original scalar ordinal, independent of cluster values.
pub struct WidthInputEvidence<'a> {
    buffer: &'a hb_buffer_t,
    ordinal: usize,
}
impl WidthFeatureReceipt<'_> {
    /// The observation request; it does not assert feature availability.
    pub fn feature(&self) -> WidthFeature {
        self.buffer
            .components
            .width
            .requested
            .expect("issued width receipt")
    }
    /// Number of original body input scalars, excluding external context.
    pub fn input_len(&self) -> usize {
        self.buffer.components.width.final_inputs.len()
    }
    /// Qualify this receipt before interpreting absence or partial coverage.
    pub fn coverage(&self) -> WidthObservationCoverage {
        if let Some(path) = self.buffer.components.width.unqualified {
            return WidthObservationCoverage::Unqualified(path);
        }
        match self.buffer.components.coverage() {
            ComponentCoverage::Complete => WidthObservationCoverage::Complete,
            ComponentCoverage::Unqualified(path) => {
                WidthObservationCoverage::Unqualified(WidthObservationPath::Native(path))
            }
        }
    }
    /// Evidence for an original input ordinal, or None outside this invocation.
    pub fn input(&self, ordinal: usize) -> Option<WidthInputEvidence<'_>> {
        (ordinal < self.input_len()).then_some(WidthInputEvidence {
            buffer: self.buffer,
            ordinal,
        })
    }
}
impl WidthInputEvidence<'_> {
    /// Universal surviving-path outcome, conditional on complete observation.
    pub fn outcome(&self) -> WidthInputOutcome {
        self.buffer.components.width.final_inputs[self.ordinal].outcome
    }
    /// Actual effective enabling contributions on surviving acted-on paths.
    pub fn provenance(&self) -> &[FeatureProvenance] {
        components::provenance(
            self.buffer,
            self.buffer.components.width.final_inputs[self.ordinal].proof,
        )
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
struct Span {
    start: usize,
    len: usize,
}
#[derive(Clone, Copy)]
struct Root {
    ordinal: u32,
    any: bool,
    all: bool,
    proof: ProvenanceSet,
}
#[derive(Clone, Copy)]
struct Final {
    outcome: WidthInputOutcome,
    proof: ProvenanceSet,
}
#[derive(Clone, Copy)]
struct Carrier {
    width: u32,
    // The actual native range domain at setup_masks, independent of both
    // original input ordinals and subsequent mutable provider clusters.
    mask_cluster: Option<u32>,
    removal_dependency: bool,
}
#[derive(Default)]
pub(crate) struct State {
    pub(crate) requested: Option<WidthFeature>,
    sets: Vec<Span>,
    roots: Vec<Root>,
    carriers: Vec<Carrier>,
    final_inputs: Vec<Final>,
    unqualified: Option<WidthObservationPath>,
    finalized: bool,
}
impl State {
    pub(crate) fn bytes(&self) -> Option<usize> {
        self.sets
            .capacity()
            .checked_mul(size_of::<Span>())?
            .checked_add(self.roots.capacity().checked_mul(size_of::<Root>())?)?
            .checked_add(self.carriers.capacity().checked_mul(size_of::<Carrier>())?)?
            .checked_add(
                self.final_inputs
                    .capacity()
                    .checked_mul(size_of::<Final>())?,
            )
    }
}
fn fail(buffer: &mut hb_buffer_t, reason: ShapingRefusal) -> bool {
    buffer.refuse(reason);
    buffer.successful = false;
    false
}
fn reserve(buffer: &mut hb_buffer_t, adds: [usize; 4]) -> bool {
    if buffer.components.width.requested.is_none() {
        return true;
    }
    let s = &buffer.components.width;
    let lens = [
        s.sets.len(),
        s.roots.len(),
        s.carriers.len(),
        s.final_inputs.len(),
    ];
    let caps = [
        s.sets.capacity(),
        s.roots.capacity(),
        s.carriers.capacity(),
        s.final_inputs.capacity(),
    ];
    let sizes = [
        size_of::<Span>(),
        size_of::<Root>(),
        size_of::<Carrier>(),
        size_of::<Final>(),
    ];
    let mut targets = [0; 4];
    let mut extra = 0usize;
    let mut work = 0usize;
    for i in 0..4 {
        let Some(target) = lens[i].checked_add(adds[i]) else {
            return fail(buffer, ShapingRefusal::GrowthLimit);
        };
        targets[i] = target;
        if target > caps[i] {
            let Some(bytes) = target
                .checked_mul(sizes[i])
                .and_then(|n| extra.checked_add(n))
            else {
                return fail(buffer, ShapingRefusal::GrowthLimit);
            };
            extra = bytes;
            let Some(next) = work.checked_add(lens[i]) else {
                return fail(buffer, ShapingRefusal::WorkLimit);
            };
            work = next;
        }
    }
    if !super::original_joining::room(buffer, buffer.len, extra)
        || !super::original_joining::work(buffer, work)
    {
        return false;
    }
    let s = &mut buffer.components.width;
    if targets[0] > caps[0] {
        s.sets.reserve_exact(targets[0] - lens[0]);
    }
    if targets[1] > caps[1] {
        s.roots.reserve_exact(targets[1] - lens[1]);
    }
    if targets[2] > caps[2] {
        s.carriers.reserve_exact(targets[2] - lens[2]);
    }
    if targets[3] > caps[3] {
        s.final_inputs.reserve_exact(targets[3] - lens[3]);
    }
    true
}
pub(crate) fn begin(buffer: &mut hb_buffer_t) -> bool {
    if buffer.components.width.requested.is_none() {
        return true;
    }
    if !reserve(buffer, [buffer.len, buffer.len, buffer.len, buffer.len])
        || !super::original_joining::work(buffer, buffer.len)
    {
        return false;
    }
    let s = &mut buffer.components.width;
    for i in 0..buffer.len {
        s.sets.push(Span {
            start: s.roots.len(),
            len: 1,
        });
        s.roots.push(Root {
            ordinal: i as u32,
            any: false,
            all: false,
            proof: ProvenanceSet::default(),
        });
        s.carriers.push(Carrier {
            width: (i + 1) as u32,
            mask_cluster: None,
            removal_dependency: false,
        });
        s.final_inputs.push(Final {
            outcome: WidthInputOutcome::NoSurvivingOperand,
            proof: ProvenanceSet::default(),
        });
    }
    true
}
pub(crate) fn carrier(buffer: &hb_buffer_t, component: u32) -> u32 {
    if buffer.components.width.requested.is_none() || component == 0 || component & (1 << 31) != 0 {
        0
    } else {
        buffer.components.width.carriers[component as usize - 1].width
    }
}
pub(crate) fn mask_cluster(buffer: &hb_buffer_t, component: u32) -> Option<u32> {
    buffer.components.width.requested?;
    let context = component & !(1 << 31);
    if context == 0 {
        None
    } else {
        buffer.components.width.carriers[context as usize - 1].mask_cluster
    }
}
pub(crate) fn binding_matches(
    buffer: &hb_buffer_t,
    component: u32,
    width: u32,
    cluster: Option<u32>,
    removal_dependency: bool,
) -> bool {
    carrier(buffer, component) == width
        && mask_cluster(buffer, component) == cluster
        && removal_dependency_failed(buffer, component) == removal_dependency
}
pub(crate) fn removal_dependency_failed(buffer: &hb_buffer_t, component: u32) -> bool {
    if buffer.components.width.requested.is_none() {
        return false;
    }
    let context = component & !(1 << 31);
    context != 0 && buffer.components.width.carriers[context as usize - 1].removal_dependency
}
pub(crate) fn bind(
    buffer: &mut hb_buffer_t,
    width: u32,
    mask_cluster: Option<u32>,
    removal_dependency: bool,
) -> bool {
    if buffer.components.width.requested.is_none() {
        return true;
    }
    if !reserve(buffer, [0, 0, 1, 0]) {
        return false;
    }
    buffer.components.width.carriers.push(Carrier {
        width,
        mask_cluster,
        removal_dependency,
    });
    true
}
pub(crate) fn depend_on_recovery(buffer: &mut hb_buffer_t, index: usize, dependency: bool) -> bool {
    if buffer.components.width.requested.is_none() || !dependency {
        return true;
    }
    let component = buffer.info[index].component_set;
    if removal_dependency_failed(buffer, component) {
        return true;
    }
    let width = carrier(buffer, component & !(1 << 31));
    let Some(component) = components::with_width_dependency(
        buffer,
        component,
        width,
        mask_cluster(buffer, component),
        true,
    ) else {
        return false;
    };
    buffer.info[index].component_set = component;
    true
}
pub(crate) fn discard(buffer: &mut hb_buffer_t, index: usize, dependency: bool) -> bool {
    if buffer.components.width.requested.is_none() {
        return true;
    }
    if !super::original_joining::work(buffer, 1) {
        return false;
    }
    let component = buffer.info[index].component_set;
    if carrier(buffer, component) != 0
        && (dependency || removal_dependency_failed(buffer, component))
    {
        unqualify_lookup(buffer);
    }
    true
}
pub(crate) fn capture_masks(buffer: &mut hb_buffer_t) -> bool {
    if buffer.components.width.requested.is_none() {
        return true;
    }
    if !super::original_joining::work(buffer, buffer.len) {
        return false;
    }
    for index in 0..buffer.len {
        let info = buffer.info[index];
        // Generated carriers retain context custody, never original roots.
        let width = carrier(buffer, info.component_set & !(1 << 31));
        let Some(component) =
            components::with_width_custody(buffer, info.component_set, width, Some(info.cluster))
        else {
            return false;
        };
        buffer.info[index].component_set = component;
    }
    true
}
fn merge_proof(
    buffer: &mut hb_buffer_t,
    a: ProvenanceSet,
    b: ProvenanceSet,
) -> Option<ProvenanceSet> {
    let mut result = a;
    let count = components::provenance(buffer, b).len();
    for i in 0..count {
        let p = components::provenance(buffer, b)[i];
        result = components::collect_provenance(buffer, result, core::iter::once(p))?;
    }
    Some(result)
}
fn intern_tail(buffer: &mut hb_buffer_t, start: usize) -> Option<u32> {
    let len = buffer.components.width.roots.len() - start;
    let count = buffer.components.width.sets.len();
    for i in 0..count {
        let span = buffer.components.width.sets[i];
        let work = span.len.checked_add(1).and_then(|initial| {
            buffer.components.width.roots[span.start..span.start + span.len]
                .iter()
                .try_fold(initial, |cost, root| {
                    cost.checked_add(components::provenance(buffer, root.proof).len())
                })
        });
        let Some(work) = work else {
            fail(buffer, ShapingRefusal::WorkLimit);
            return None;
        };
        if !super::original_joining::work(buffer, work) {
            return None;
        }
        let s = &buffer.components.width;
        if span.len == len
            && s.roots[span.start..span.start + span.len]
                .iter()
                .zip(&s.roots[start..])
                .all(|(a, b)| {
                    a.ordinal == b.ordinal
                        && a.any == b.any
                        && a.all == b.all
                        && components::provenance(buffer, a.proof)
                            == components::provenance(buffer, b.proof)
                })
        {
            buffer.components.width.roots.truncate(start);
            return Some(i as u32 + 1);
        }
    }
    if buffer.components.width.sets.len() >= u32::MAX as usize || !reserve(buffer, [1, 0, 0, 0]) {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return None;
    }
    buffer.components.width.sets.push(Span { start, len });
    Some(buffer.components.width.sets.len() as u32)
}
pub(crate) fn merge(buffer: &mut hb_buffer_t, a: u32, b: u32) -> Option<u32> {
    let a = carrier(buffer, a);
    let b = carrier(buffer, b);
    if a == 0 || a == b {
        return Some(b);
    }
    if b == 0 {
        return Some(a);
    }
    let aa = buffer.components.width.sets[a as usize - 1];
    let bb = buffer.components.width.sets[b as usize - 1];
    let Some(size) = aa.len.checked_add(bb.len) else {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return None;
    };
    if !reserve(buffer, [0, size, 0, 0]) || !super::original_joining::work(buffer, size) {
        return None;
    }
    let start = buffer.components.width.roots.len();
    let mut i = 0;
    let mut j = 0;
    while i < aa.len || j < bb.len {
        let x = (i < aa.len).then(|| buffer.components.width.roots[aa.start + i]);
        let y = (j < bb.len).then(|| buffer.components.width.roots[bb.start + j]);
        let value = match (x, y) {
            (Some(x), Some(y)) if x.ordinal == y.ordinal => {
                i += 1;
                j += 1;
                Root {
                    ordinal: x.ordinal,
                    any: x.any || y.any,
                    all: x.all && y.all,
                    proof: merge_proof(buffer, x.proof, y.proof)?,
                }
            }
            (Some(x), Some(y)) if x.ordinal < y.ordinal => {
                i += 1;
                x
            }
            (_, Some(y)) => {
                j += 1;
                y
            }
            (Some(x), None) => {
                i += 1;
                x
            }
            (None, None) => unreachable!(),
        };
        buffer.components.width.roots.push(value);
    }
    intern_tail(buffer, start)
}
pub(crate) fn act(buffer: &mut hb_buffer_t, index: usize, proof: ProvenanceSet) -> bool {
    if buffer.components.width.requested.is_none()
        || components::provenance(buffer, proof).is_empty()
    {
        return true;
    }
    let component = buffer.info[index].component_set;
    let id = carrier(buffer, component);
    if id == 0 {
        return true;
    }
    let span = buffer.components.width.sets[id as usize - 1];
    if !reserve(buffer, [0, span.len, 0, 0]) || !super::original_joining::work(buffer, span.len) {
        return false;
    }
    let start = buffer.components.width.roots.len();
    for i in 0..span.len {
        let root = buffer.components.width.roots[span.start + i];
        let Some(merged) = merge_proof(buffer, root.proof, proof) else {
            return false;
        };
        buffer.components.width.roots.push(Root {
            ordinal: root.ordinal,
            any: true,
            all: true,
            proof: merged,
        });
    }
    let Some(width) = intern_tail(buffer, start) else {
        return false;
    };
    let Some(component) = components::with_width(buffer, component, width) else {
        return false;
    };
    buffer.info[index].component_set = component;
    true
}
pub(crate) fn unqualify_value(buffer: &mut hb_buffer_t) {
    if buffer.components.width.requested.is_some() && buffer.components.width.unqualified.is_none()
    {
        buffer.components.width.unqualified = Some(WidthObservationPath::ValueRecord);
    }
}
pub(crate) fn unqualify_lookup(buffer: &mut hb_buffer_t) {
    if buffer.components.width.requested.is_some() && buffer.components.width.unqualified.is_none()
    {
        buffer.components.width.unqualified = Some(WidthObservationPath::LookupAdmission);
    }
}
pub(crate) fn finish(buffer: &mut hb_buffer_t) {
    if buffer.components.width.requested.is_none()
        || buffer.shaping_status() != crate::ShapingStatus::Complete
    {
        return;
    }
    for glyph in 0..buffer.len {
        if buffer.info[glyph].di_residual() {
            continue;
        }
        let id = carrier(buffer, buffer.info[glyph].component_set);
        if id == 0 {
            continue;
        }
        let span = buffer.components.width.sets[id as usize - 1];
        if !super::original_joining::work(buffer, span.len) {
            return;
        }
        for i in 0..span.len {
            let root = buffer.components.width.roots[span.start + i];
            let old = buffer.components.width.final_inputs[root.ordinal as usize];
            let any = root.any
                || matches!(
                    old.outcome,
                    WidthInputOutcome::Partial | WidthInputOutcome::Covered
                );
            let all = root.all
                && matches!(
                    old.outcome,
                    WidthInputOutcome::NoSurvivingOperand | WidthInputOutcome::Covered
                );
            let outcome = if all {
                WidthInputOutcome::Covered
            } else if any {
                WidthInputOutcome::Partial
            } else {
                WidthInputOutcome::NoAction
            };
            let Some(proof) = merge_proof(buffer, old.proof, root.proof) else {
                return;
            };
            buffer.components.width.final_inputs[root.ordinal as usize] = Final { outcome, proof };
        }
    }
    buffer.components.width.finalized = true;
}
pub(crate) fn receipt(buffer: &hb_buffer_t) -> Option<WidthFeatureReceipt<'_>> {
    buffer.components.width.requested?;
    debug_assert!(buffer.components.width.finalized);
    Some(WidthFeatureReceipt { buffer })
}

//! Compact invocation-local correspondence and movement constraints.
use super::buffer::{hb_buffer_t, ShapingRefusal};
use crate::{Feature, Tag};
use alloc::vec::Vec;
use core::mem::size_of;

/// Qualification of the actual substitution and positioning path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentCoverage {
    /// All executed transformations preserve correspondence and constraints.
    Complete,
    /// The named active path has no complete observation contract.
    Unqualified(ComponentPath),
}
/// An actual provider path whose constraints are not qualified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentPath {
    /// Apple advanced substitution.
    AatSubstitution,
    /// Apple advanced positioning.
    AatPositioning,
    /// Generated repair output without qualified context correspondence.
    GeneratedRepair,
    /// An actual attachment chain has no valid final parent correspondence.
    AttachmentCorrespondence,
}
/// Effective role of an enabled feature contribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeatureRole {
    /// Required by the selected language system or shaping plan.
    Required,
    /// Affirmatively optional in the selected plan.
    Optional,
    /// No optional or required role is qualified.
    Unclassified,
}
/// Source of an effective feature contribution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FeatureSource {
    /// Native provider feature registration.
    Provider,
    /// An actual caller request, retaining its origin and range.
    Request(Feature),
    /// Required feature selected by the actual OpenType language system.
    LanguageSystem,
    /// A required feature registered by the selected script shaper.
    Script,
}
/// One enabled contribution to an actual applied lookup.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FeatureProvenance {
    pub(crate) tag: Tag,
    pub(crate) role: FeatureRole,
    pub(crate) source: FeatureSource,
}
impl FeatureProvenance {
    /// OpenType feature tag.
    pub fn tag(&self) -> Tag {
        self.tag
    }
    /// Effective selected-plan role.
    pub fn role(&self) -> FeatureRole {
        self.role
    }
    /// Actual enabled source, including caller range where applicable.
    pub fn source(&self) -> FeatureSource {
        self.source
    }
}
/// Admission of a soft optional-default request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefaultFeatureDisposition {
    /// Optional request admitted to ordinary mask compilation.
    Accepted,
    /// Required feature retained unchanged.
    Required,
    /// Unknown role retained unchanged.
    Unclassified,
}
/// Compiled observation of an original optional-default request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DefaultFeature {
    pub(crate) request: Feature,
    pub(crate) disposition: DefaultFeatureDisposition,
}
impl DefaultFeature {
    /// Original request, including exact bounds and origin.
    pub fn request(&self) -> Feature {
        self.request
    }
    /// Actual compiled admission decision.
    pub fn disposition(&self) -> DefaultFeatureDisposition {
        self.disposition
    }
}
/// An actual positional dependence, rather than ordinary kerning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentAttachmentKind {
    /// Mark-to-base, mark-to-ligature, mark-to-mark or fallback mark positioning.
    Mark,
    /// Cursive attachment.
    Cursive,
}
#[derive(Clone, Copy, Default)]
struct Span {
    start: usize,
    len: usize,
}
#[derive(Clone, Copy, Default)]
pub(crate) struct ProvenanceSet(Span);
#[derive(Clone, Copy)]
struct Ligature {
    set: u32,
    provenance: Span,
    glyphs: Span,
}
#[derive(Clone, Copy)]
struct Attachment {
    dependent: u32,
    base: u32,
    kind: ComponentAttachmentKind,
}
/// Borrowed observation tied to one completed glyph output.
pub struct ComponentReceipt<'a> {
    buffer: &'a hb_buffer_t,
}
/// Borrowed actual multi-component ligature constraint.
pub struct ComponentLigature<'a> {
    state: &'a State,
    row: &'a Ligature,
}
impl ComponentLigature<'_> {
    /// Exact consumed body scalar ordinals, excluding skipped marks.
    pub fn components(&self) -> &[u32] {
        self.state.members(self.row.set)
    }
    /// All final descendants sharing any consumed component.
    pub fn glyph_indices(&self) -> &[u32] {
        self.state.slice(self.row.glyphs)
    }
    /// Effective enabled provenance set, including nested parents.
    pub fn provenance(&self) -> &[FeatureProvenance] {
        let s = self.row.provenance;
        &self.state.provenance[s.start..s.start + s.len]
    }
}
/// Borrowed final positional dependence.
pub struct ComponentAttachment<'a> {
    state: &'a State,
    row: &'a Attachment,
}
impl ComponentAttachment<'_> {
    /// Original body components of the dependent output.
    pub fn dependent_components(&self) -> &[u32] {
        self.state.members(self.row.dependent)
    }
    /// Original body components of its actual attachment parent.
    pub fn base_components(&self) -> &[u32] {
        self.state.members(self.row.base)
    }
    /// Actual attachment kind.
    pub fn kind(&self) -> ComponentAttachmentKind {
        self.row.kind
    }
}
impl ComponentReceipt<'_> {
    /// Count of original body scalars, excluding external context.
    pub fn input_len(&self) -> usize {
        self.buffer.components.input_len
    }
    /// Exact input membership of one final glyph; generated output has no invented input.
    pub fn components(&self, glyph_index: usize) -> Option<&[u32]> {
        self.buffer
            .info
            .get(glyph_index)
            .filter(|_| glyph_index < self.buffer.len)
            .map(|g| self.buffer.components.members(g.component_set))
    }
    /// Whether this final glyph was generated by a provider repair.
    pub fn generated(&self, glyph_index: usize) -> Option<bool> {
        self.buffer
            .info
            .get(glyph_index)
            .filter(|_| glyph_index < self.buffer.len)
            .map(|g| g.component_set & GENERATED != 0)
    }
    /// Actual generated output's contextual input association, when present.
    pub fn generated_context(&self, glyph_index: usize) -> Option<&[u32]> {
        let g = self
            .buffer
            .info
            .get(glyph_index)
            .filter(|_| glyph_index < self.buffer.len)?;
        (g.component_set & GENERATED != 0)
            .then(|| self.buffer.components.members(g.component_set & !GENERATED))
    }
    /// Qualification of the actual executed provider path.
    pub fn coverage(&self) -> ComponentCoverage {
        self.buffer.components.coverage
    }
    /// Canonical actual ligature constraints with final descendant indices.
    pub fn ligatures(&self) -> impl Iterator<Item = ComponentLigature<'_>> {
        let state = &self.buffer.components;
        state
            .ligatures
            .iter()
            .map(move |row| ComponentLigature { state, row })
    }
    /// Actual final attachment constraints.
    pub fn attachments(&self) -> impl Iterator<Item = ComponentAttachment<'_>> {
        let state = &self.buffer.components;
        state
            .attachments
            .iter()
            .map(move |row| ComponentAttachment { state, row })
    }
    /// Soft feature admission decisions for this invocation.
    pub fn default_features(&self) -> &[DefaultFeature] {
        &self.buffer.components.defaults
    }
    /// Original inputs with no surviving descendant; consumption remains explicitly accounted for.
    pub fn deleted_input_positions(&self) -> &[u32] {
        self.buffer.components.slice(self.buffer.components.deleted)
    }
}
const GENERATED: u32 = 1 << 31;
pub(crate) struct State {
    pub(crate) requested: bool,
    active: bool,
    pub(crate) generating_repair: bool,
    input_len: usize,
    coverage: ComponentCoverage,
    sets: Vec<Span>,
    members: Vec<u32>,
    ligatures: Vec<Ligature>,
    attachments: Vec<Attachment>,
    provenance: Vec<FeatureProvenance>,
    defaults: Vec<DefaultFeature>,
    indices: Vec<u32>,
    deleted: Span,
    finalized: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            requested: false,
            active: false,
            generating_repair: false,
            input_len: 0,
            coverage: ComponentCoverage::Complete,
            sets: Vec::new(),
            members: Vec::new(),
            ligatures: Vec::new(),
            attachments: Vec::new(),
            provenance: Vec::new(),
            defaults: Vec::new(),
            indices: Vec::new(),
            deleted: Span::default(),
            finalized: false,
        }
    }
}
impl State {
    pub(crate) fn active(&self) -> bool {
        self.active
    }
    pub(crate) fn bytes(&self) -> Option<usize> {
        self.sets
            .capacity()
            .checked_mul(size_of::<Span>())?
            .checked_add(self.members.capacity().checked_mul(4)?)?
            .checked_add(
                self.ligatures
                    .capacity()
                    .checked_mul(size_of::<Ligature>())?,
            )?
            .checked_add(
                self.attachments
                    .capacity()
                    .checked_mul(size_of::<Attachment>())?,
            )?
            .checked_add(
                self.provenance
                    .capacity()
                    .checked_mul(size_of::<FeatureProvenance>())?,
            )?
            .checked_add(
                self.defaults
                    .capacity()
                    .checked_mul(size_of::<DefaultFeature>())?,
            )?
            .checked_add(self.indices.capacity().checked_mul(4)?)
    }
    pub(crate) fn members(&self, id: u32) -> &[u32] {
        if id == 0 || id & GENERATED != 0 {
            return &[];
        }
        let s = self.sets[id as usize - 1];
        &self.members[s.start..s.start + s.len]
    }
    pub(crate) fn input_cluster(&self, ordinal: u32) -> u32 {
        self.indices[ordinal as usize]
    }
    fn slice(&self, s: Span) -> &[u32] {
        &self.indices[s.start..s.start + s.len]
    }
}
fn reserve(buffer: &mut hb_buffer_t, additions: [usize; 7]) -> bool {
    if !buffer.components.active {
        return true;
    }
    let s = &buffer.components;
    let lens = [
        s.sets.len(),
        s.members.len(),
        s.ligatures.len(),
        s.attachments.len(),
        s.provenance.len(),
        s.defaults.len(),
        s.indices.len(),
    ];
    let caps = [
        s.sets.capacity(),
        s.members.capacity(),
        s.ligatures.capacity(),
        s.attachments.capacity(),
        s.provenance.capacity(),
        s.defaults.capacity(),
        s.indices.capacity(),
    ];
    let sizes = [
        size_of::<Span>(),
        4,
        size_of::<Ligature>(),
        size_of::<Attachment>(),
        size_of::<FeatureProvenance>(),
        size_of::<DefaultFeature>(),
        4,
    ];
    let mut targets = [0; 7];
    let mut extra = 0usize;
    let mut work = 0usize;
    for i in 0..7 {
        let Some(target) = lens[i].checked_add(additions[i]) else {
            return fail(buffer, ShapingRefusal::GrowthLimit);
        };
        targets[i] = target;
        if target > caps[i] {
            let Some(n) = target
                .checked_mul(sizes[i])
                .and_then(|n| extra.checked_add(n))
            else {
                return fail(buffer, ShapingRefusal::GrowthLimit);
            };
            extra = n;
            let Some(n) = work.checked_add(lens[i]) else {
                return fail(buffer, ShapingRefusal::WorkLimit);
            };
            work = n;
        }
    }
    if !super::original_joining::room(buffer, buffer.len, extra)
        || !super::original_joining::work(buffer, work)
    {
        return false;
    }
    let s = &mut buffer.components;
    macro_rules! grow {
        ($field:ident,$i:expr) => {
            if targets[$i] > caps[$i] {
                s.$field.reserve_exact(targets[$i] - lens[$i]);
            }
        };
    }
    grow!(sets, 0);
    grow!(members, 1);
    grow!(ligatures, 2);
    grow!(attachments, 3);
    grow!(provenance, 4);
    grow!(defaults, 5);
    grow!(indices, 6);
    true
}
fn fail(buffer: &mut hb_buffer_t, reason: ShapingRefusal) -> bool {
    buffer.refuse(reason);
    buffer.successful = false;
    false
}
pub(crate) fn begin(
    buffer: &mut hb_buffer_t,
    plan: &super::ot_shape_plan::hb_ot_shape_plan_t,
) -> bool {
    if !buffer.components.requested {
        return true;
    }
    buffer.components.active = true;
    buffer.components.input_len = buffer.len;
    if buffer.len >= GENERATED as usize {
        return fail(buffer, ShapingRefusal::GrowthLimit);
    }
    if !reserve(buffer, [buffer.len, buffer.len, 0, 0, 0, 0, buffer.len])
        || !super::original_joining::work(buffer, buffer.len)
    {
        return false;
    }
    for i in 0..buffer.len {
        let s = &mut buffer.components;
        s.sets.push(Span {
            start: s.members.len(),
            len: 1,
        });
        s.members.push(i as u32);
        s.indices.push(buffer.info[i].cluster);
        buffer.info[i].component_set = (i + 1) as u32;
    }
    set_defaults(buffer, plan.ot_map.default_features())
}
pub(crate) fn unqualify(buffer: &mut hb_buffer_t, path: ComponentPath) {
    if buffer.components.active && buffer.components.coverage == ComponentCoverage::Complete {
        buffer.components.coverage = ComponentCoverage::Unqualified(path);
    }
}
pub(crate) fn union(buffer: &mut hb_buffer_t, a: u32, b: u32) -> Option<u32> {
    if !buffer.components.active || a == b {
        return Some(a);
    }
    if a == 0 {
        return Some(b);
    }
    if b == 0 {
        return Some(a);
    }
    if a & GENERATED != 0 || b & GENERATED != 0 {
        unqualify(buffer, ComponentPath::GeneratedRepair);
        return Some(if a & GENERATED != 0 && b & GENERATED == 0 {
            b
        } else {
            a
        });
    }
    let Some(size) = buffer
        .components
        .members(a)
        .len()
        .checked_add(buffer.components.members(b).len())
    else {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return None;
    };
    if !reserve(buffer, [1, size, 0, 0, 0, 0, 0]) || !super::original_joining::work(buffer, size) {
        return None;
    }
    let Some(compare_work) = buffer
        .components
        .sets
        .iter()
        .try_fold(0usize, |n, set| n.checked_add(1)?.checked_add(set.len))
    else {
        fail(buffer, ShapingRefusal::WorkLimit);
        return None;
    };
    if !super::original_joining::work(buffer, compare_work) {
        return None;
    }
    let s = &mut buffer.components;
    let start = s.members.len();
    let mut i = 0;
    let mut j = 0;
    let aa = s.sets[a as usize - 1];
    let bb = s.sets[b as usize - 1];
    while i < aa.len || j < bb.len {
        let x = if j == bb.len || (i < aa.len && s.members[aa.start + i] < s.members[bb.start + j])
        {
            let x = s.members[aa.start + i];
            i += 1;
            x
        } else {
            let x = s.members[bb.start + j];
            j += 1;
            if i < aa.len && s.members[aa.start + i] == x {
                i += 1;
            }
            x
        };
        s.members.push(x);
    }
    let len = s.members.len() - start;
    // Intern exact sets so canonical constraints have stable compact identity.
    for (i, old) in s.sets.iter().enumerate() {
        if old.len == len && s.members[old.start..old.start + len] == s.members[start..] {
            s.members.truncate(start);
            return Some(i as u32 + 1);
        }
    }
    if s.sets.len() >= GENERATED as usize - 1 {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return None;
    }
    s.sets.push(Span { start, len });
    Some(s.sets.len() as u32)
}
pub(crate) fn generated(buffer: &hb_buffer_t, context: u32) -> u32 {
    if buffer.components.active {
        GENERATED | (context & !GENERATED)
    } else {
        0
    }
}
pub(crate) fn collect_provenance(
    buffer: &mut hb_buffer_t,
    inherited: ProvenanceSet,
    contributions: impl Iterator<Item = FeatureProvenance>,
) -> Option<ProvenanceSet> {
    if !buffer.components.active {
        return Some(ProvenanceSet::default());
    }
    let mut span = inherited.0;
    let mut appended = false;
    for p in contributions {
        if !super::original_joining::work(buffer, span.len.saturating_add(1)) {
            return None;
        }
        if buffer.components.provenance[span.start..span.start + span.len].contains(&p) {
            continue;
        }
        if !appended {
            let Some(extra) = span.len.checked_add(1) else {
                fail(buffer, ShapingRefusal::GrowthLimit);
                return None;
            };
            if !reserve(buffer, [0, 0, 0, 0, extra, 0, 0])
                || !super::original_joining::work(buffer, span.len)
            {
                return None;
            }
            let start = buffer.components.provenance.len();
            for i in span.start..span.start + span.len {
                let previous = buffer.components.provenance[i];
                buffer.components.provenance.push(previous);
            }
            span.start = start;
            appended = true;
        } else if !reserve(buffer, [0, 0, 0, 0, 1, 0, 0]) {
            return None;
        }
        buffer.components.provenance.push(p);
        span.len += 1;
    }
    if appended {
        let count = buffer.components.ligatures.len();
        let Some(work) = count
            .checked_mul(span.len.saturating_add(1))
            .and_then(|n| n.checked_mul(span.len.saturating_add(1)))
        else {
            fail(buffer, ShapingRefusal::WorkLimit);
            return None;
        };
        if !super::original_joining::work(buffer, work) {
            return None;
        }
        for row in &buffer.components.ligatures {
            let old = row.provenance;
            if old.len == span.len
                && buffer.components.provenance[old.start..old.start + old.len]
                    .iter()
                    .all(|p| {
                        buffer.components.provenance[span.start..span.start + span.len].contains(p)
                    })
            {
                buffer.components.provenance.truncate(span.start);
                return Some(ProvenanceSet(old));
            }
        }
    }
    Some(ProvenanceSet(span))
}

pub(crate) fn ligature(buffer: &mut hb_buffer_t, set: u32, provenance: ProvenanceSet) -> bool {
    if !buffer.components.active {
        return true;
    }
    if !super::original_joining::work(buffer, buffer.components.ligatures.len()) {
        return false;
    }
    let existing = buffer
        .components
        .ligatures
        .iter()
        .position(|r| r.set == set);
    let merged = if let Some(i) = existing {
        let old = buffer.components.ligatures[i].provenance;
        let mut merged = provenance;
        for j in old.start..old.start + old.len {
            let p = buffer.components.provenance[j];
            let Some(next) = collect_provenance(buffer, merged, core::iter::once(p)) else {
                return false;
            };
            merged = next;
        }
        merged
    } else {
        provenance
    };
    if !reserve(buffer, [0, 0, usize::from(existing.is_none()), 0, 0, 0, 0]) {
        return false;
    }
    let row = Ligature {
        set,
        provenance: merged.0,
        glyphs: Span::default(),
    };
    if let Some(i) = existing {
        buffer.components.ligatures[i] = row;
    } else {
        buffer.components.ligatures.push(row);
    }
    true
}
pub(crate) fn attachment(
    buffer: &mut hb_buffer_t,
    dependent: usize,
    base: usize,
    kind: ComponentAttachmentKind,
) -> bool {
    if !buffer.components.active {
        return true;
    }
    let dependent = buffer.info[dependent].component_set;
    let base = buffer.info[base].component_set;
    if dependent == base {
        return true;
    }
    let row = Attachment {
        dependent,
        base,
        kind,
    };
    if !super::original_joining::work(buffer, buffer.components.attachments.len()) {
        return false;
    }
    if buffer
        .components
        .attachments
        .iter()
        .any(|r| r.dependent == dependent && r.base == base && r.kind == kind)
    {
        return true;
    }
    if !reserve(buffer, [0, 0, 0, 1, 0, 0, 0]) {
        return false;
    }
    buffer.components.attachments.push(row);
    true
}
pub(crate) fn capture_attachments(buffer: &mut hb_buffer_t) -> bool {
    if !buffer.components.active {
        return true;
    }
    if !super::original_joining::work(buffer, buffer.len) {
        return false;
    }
    for i in 0..buffer.len {
        let chain = buffer.pos[i].attach_chain();
        if chain == 0 {
            continue;
        }
        let Some(base) = i
            .checked_add_signed(isize::from(chain))
            .filter(|&j| j < buffer.len)
        else {
            unqualify(buffer, ComponentPath::AttachmentCorrespondence);
            continue;
        };
        let kind =
            if buffer.pos[i].attach_type() & super::ot_layout_gpos_table::attach_type::CURSIVE != 0
            {
                ComponentAttachmentKind::Cursive
            } else {
                ComponentAttachmentKind::Mark
            };
        if !attachment(buffer, i, base, kind) {
            return false;
        }
    }
    true
}
pub(crate) fn set_defaults(buffer: &mut hb_buffer_t, defaults: &[DefaultFeature]) -> bool {
    if !buffer.components.active {
        return true;
    }
    if !reserve(buffer, [0, 0, 0, 0, 0, defaults.len(), 0])
        || !super::original_joining::work(buffer, defaults.len())
    {
        return false;
    }
    buffer.components.defaults.extend_from_slice(defaults);
    true
}
pub(crate) fn finish(buffer: &mut hb_buffer_t) {
    if !buffer.components.active {
        return;
    }
    let rows = buffer.components.ligatures.len();
    for row in 0..rows {
        let set = buffer.components.ligatures[row].set;
        let start = buffer.components.indices.len();
        for i in 0..buffer.len {
            let id = buffer.info[i].component_set;
            let Some(cost) = buffer
                .components
                .members(set)
                .len()
                .checked_mul(buffer.components.members(id).len().saturating_add(1))
            else {
                fail(buffer, ShapingRefusal::WorkLimit);
                return;
            };
            if !super::original_joining::work(buffer, cost) {
                return;
            }
            let shares = buffer
                .components
                .members(set)
                .iter()
                .any(|x| buffer.components.members(id).binary_search(x).is_ok());
            if shares {
                if !reserve(buffer, [0, 0, 0, 0, 0, 0, 1]) {
                    return;
                }
                buffer.components.indices.push(i as u32);
            }
        }
        buffer.components.ligatures[row].glyphs = Span {
            start,
            len: buffer.components.indices.len() - start,
        };
    }
    let start = buffer.components.indices.len();
    let input_len = buffer.components.input_len;
    if !reserve(buffer, [0, 0, 0, 0, 0, 0, input_len])
        || !super::original_joining::work(buffer, input_len)
    {
        return;
    }
    buffer.components.indices.resize(start + input_len, 0);
    for i in 0..buffer.len {
        let id = buffer.info[i].component_set;
        let len = buffer.components.members(id).len();
        if !super::original_joining::work(buffer, len) {
            return;
        }
        for j in 0..len {
            let ordinal = buffer.components.members(id)[j] as usize;
            buffer.components.indices[start + ordinal] = 1;
        }
    }
    if !super::original_joining::work(buffer, input_len) {
        return;
    }
    let mut end = start;
    for ordinal in 0..input_len {
        if buffer.components.indices[start + ordinal] == 0 {
            buffer.components.indices[end] = ordinal as u32;
            end += 1;
        }
    }
    buffer.components.indices.truncate(end);
    buffer.components.deleted = Span {
        start,
        len: end - start,
    };
    buffer.components.finalized = true;
}

pub(crate) fn receipt(buffer: &hb_buffer_t) -> Option<ComponentReceipt<'_>> {
    if buffer.components.requested {
        debug_assert!(buffer.components.finalized);
        Some(ComponentReceipt { buffer })
    } else {
        None
    }
}

pub(crate) fn generated_repairs(buffer: &mut hb_buffer_t, f: fn(&mut hb_buffer_t)) {
    buffer.components.generating_repair = true;
    f(buffer);
    buffer.components.generating_repair = false;
}

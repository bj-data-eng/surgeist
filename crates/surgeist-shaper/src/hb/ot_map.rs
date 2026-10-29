use alloc::vec::Vec;
use core::cmp::Ordering;
use core::ops::Range;

use super::buffer::{glyph_flag, hb_buffer_t};
use super::common::{canonical_features, FeatureOrigin, TagExt};
use super::components::{
    DefaultFeature, DefaultFeatureDisposition, FeatureProvenance, FeatureRole, FeatureSource,
};
use super::ot_layout::TableIndex;
use super::ot_shape_plan::hb_ot_shape_plan_t;
use super::Feature;
use super::{hb_font_t, hb_mask_t, hb_tag_t, tag, Language, Script};

// TODO: Remove once MSRV is 1.80+
use core::mem::{size_of, size_of_val};

pub struct hb_ot_map_t {
    found_script: [bool; 2],
    chosen_script: [Option<hb_tag_t>; 2],
    global_mask: hb_mask_t,
    features: Vec<feature_map_t>,
    lookups: [Vec<lookup_map_t>; 2],
    stages: [Vec<StageMap>; 2],
    feature_variations: [Option<u32>; 2],
    capacity_loss: FeatureCapacityLoss,
    requests: Vec<Feature>,
    default_features: Vec<DefaultFeature>,
    provenance: [Vec<LookupFeature>; 2],
}

struct LookupFeature {
    tag: hb_tag_t,
    stage: usize,
    role: FeatureRole,
    source: FeatureSource,
    mask: hb_mask_t,
    lookups: Vec<u16>,
    width_rejected: Vec<u16>,
    fixed: bool,
}

// Source-owned observations of the existing bit-allocation decisions. These
// do not alter allocation, masks, stages or selection. Qualification happens on
// each actual nonempty invocation, and native masks at their consuming paths.
#[derive(Default)]
struct FeatureCapacityLoss {
    tables: [bool; 2],
    kerning: [bool; 2],
}

impl FeatureCapacityLoss {
    fn record(&mut self, info: &feature_info_t, indices: [Option<u16>; 2]) {
        for side in 0..2 {
            self.tables[side] |= indices[side].is_some();
        }
        if info.flags & F_HAS_FALLBACK != 0 || indices.iter().any(Option::is_some) {
            if info.tag == hb_tag_t::new(b"kern") {
                self.kerning[0] = true;
            } else if info.tag == hb_tag_t::new(b"vkrn") {
                self.kerning[1] = true;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct feature_map_t {
    tag: hb_tag_t,
    // GSUB/GPOS
    index: [Option<u16>; 2],
    stage: [usize; 2],
    shift: u32,
    mask: hb_mask_t,
    // mask for value=1, for quick access
    one_mask: hb_mask_t,
    auto_zwnj: bool,
    auto_zwj: bool,
    random: bool,
    per_syllable: bool,
}

impl Ord for feature_map_t {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tag.cmp(&other.tag)
    }
}

impl PartialOrd for feature_map_t {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.tag.partial_cmp(&other.tag)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct lookup_map_t {
    pub index: u16,
    // TODO: to bitflags
    pub auto_zwnj: bool,
    pub auto_zwj: bool,
    pub random: bool,
    pub mask: hb_mask_t,
    pub per_syllable: bool,
}

#[derive(Clone, Copy)]
pub struct StageMap {
    // Cumulative
    pub last_lookup: usize,
    pub pause_func: Option<pause_func_t>,
}

// Pause functions return true if new glyph indices might have been added to the buffer.
// This is used to update buffer digest.
pub type pause_func_t = fn(&hb_ot_shape_plan_t, &hb_font_t, &mut hb_buffer_t) -> bool;

impl hb_ot_map_t {
    pub(crate) fn default_features(&self) -> &[DefaultFeature] {
        &self.default_features
    }

    pub(crate) fn requests(&self) -> &[Feature] {
        &self.requests
    }

    /// Conservative scan work for one member query, charged by the observer.
    pub(crate) fn provenance_work(&self, table: TableIndex) -> Option<usize> {
        self.provenance[table]
            .iter()
            .try_fold(0usize, |work, feature| {
                work.checked_add(1)?
                    .checked_add(feature.lookups.len())?
                    .checked_add(self.requests.len())
            })
    }

    /// Every effective selected feature contribution, including merged lookups.
    /// Repetition across matched components is intentional: the observation owner
    /// interns the union under its invocation resource budget.
    pub(crate) fn lookup_provenance<'a>(
        &'a self,
        table: TableIndex,
        stage: usize,
        lookup: u16,
        inputs: &'a [(hb_mask_t, u32)],
    ) -> impl Iterator<Item = FeatureProvenance> + 'a {
        self.lookup_provenance_inner(table, stage, lookup, inputs, false)
    }
    pub(crate) fn width_lookup_provenance<'a>(
        &'a self,
        table: TableIndex,
        stage: usize,
        lookup: u16,
        inputs: &'a [(hb_mask_t, u32)],
    ) -> impl Iterator<Item = FeatureProvenance> + 'a {
        self.lookup_provenance_inner(table, stage, lookup, inputs, true)
    }
    fn lookup_provenance_inner<'a>(
        &'a self,
        table: TableIndex,
        stage: usize,
        lookup: u16,
        inputs: &'a [(hb_mask_t, u32)],
        rejected: bool,
    ) -> impl Iterator<Item = FeatureProvenance> + 'a {
        self.provenance[table]
            .iter()
            .filter(move |feature| {
                feature.stage == stage
                    && (feature.lookups.contains(&lookup)
                        || rejected && feature.width_rejected.contains(&lookup))
            })
            .flat_map(move |feature| {
                inputs.iter().filter_map(move |(mask, cluster)| {
                    if mask & feature.mask == 0 {
                        return None;
                    }
                    let request = (!feature.fixed)
                        .then(|| {
                            self.requests.iter().rev().find(|request| {
                                request.tag == feature.tag
                                    && (request.is_global()
                                        || request.start <= *cluster && *cluster < request.end)
                            })
                        })
                        .flatten();
                    if request.is_some_and(|request| request.value == 0) {
                        return None;
                    }
                    Some(FeatureProvenance {
                        tag: feature.tag,
                        role: feature.role,
                        source: request
                            .map_or(feature.source, |request| FeatureSource::Request(*request)),
                    })
                })
            })
    }

    pub(crate) fn width_provenance_work(&self, table: TableIndex) -> Option<usize> {
        self.provenance[table]
            .iter()
            .try_fold(self.provenance_work(table)?, |work, feature| {
                work.checked_add(feature.width_rejected.len())
            })
    }
    pub(crate) fn next_width_lookup(
        &self,
        table: TableIndex,
        stage: usize,
        tag: hb_tag_t,
        after: Option<u16>,
    ) -> Option<u16> {
        self.provenance[table]
            .iter()
            .filter(|f| f.stage == stage && f.tag == tag)
            .flat_map(|f| f.lookups.iter().chain(&f.width_rejected))
            .copied()
            .filter(|index| after.is_none_or(|after| *index > after))
            .min()
    }

    pub const MAX_BITS: u32 = 8;
    pub const MAX_VALUE: u32 = (1 << Self::MAX_BITS) - 1;

    pub(crate) fn selected_capacity_loss(
        &self,
        _requests: &[Feature],
        selected: [bool; 2],
    ) -> bool {
        if (0..2).any(|side| selected[side] && self.capacity_loss.tables[side]) {
            return true;
        }
        self.requests.iter().any(|request| {
            !request.is_global()
                && self
                    .features
                    .binary_search_by_key(&request.tag, |f| f.tag)
                    .is_ok_and(|index| {
                        let feature = &self.features[index];
                        request.value > feature.mask >> feature.shift
                            && (0..2).any(|side| selected[side] && feature.index[side].is_some())
                    })
        })
    }

    pub(crate) fn native_kerning_capacity_loss(
        &self,
        _requests: &[Feature],
        horizontal: bool,
    ) -> bool {
        let side = usize::from(!horizontal);
        if self.capacity_loss.kerning[side] {
            return true;
        }
        let tag = if horizontal {
            hb_tag_t::new(b"kern")
        } else {
            hb_tag_t::new(b"vkrn")
        };
        let (mask, shift) = self.get_mask(tag);
        mask != 0
            && self.requests.iter().any(|request| {
                !request.is_global() && request.tag == tag && request.value > mask >> shift
            })
    }

    #[inline]
    pub fn found_script(&self, table_index: TableIndex) -> bool {
        self.found_script[table_index]
    }

    #[inline]
    pub fn chosen_script(&self, table_index: TableIndex) -> Option<hb_tag_t> {
        self.chosen_script[table_index]
    }

    #[inline]
    pub fn get_global_mask(&self) -> hb_mask_t {
        self.global_mask
    }

    #[inline]
    pub fn get_mask(&self, feature_tag: hb_tag_t) -> (hb_mask_t, u32) {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map_or((0, 0), |idx| {
                (self.features[idx].mask, self.features[idx].shift)
            })
    }

    #[inline]
    pub fn get_1_mask(&self, feature_tag: hb_tag_t) -> hb_mask_t {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map_or(0, |idx| self.features[idx].one_mask)
    }

    #[inline]
    pub fn get_feature_index(&self, table_index: TableIndex, feature_tag: hb_tag_t) -> Option<u16> {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .ok()
            .and_then(|idx| self.features[idx].index[table_index])
    }

    #[inline]
    pub fn get_feature_stage(
        &self,
        table_index: TableIndex,
        feature_tag: hb_tag_t,
    ) -> Option<usize> {
        self.features
            .binary_search_by_key(&feature_tag, |f| f.tag)
            .map(|idx| self.features[idx].stage[table_index])
            .ok()
    }

    #[inline]
    pub fn stages(&self, table_index: TableIndex) -> &[StageMap] {
        &self.stages[table_index]
    }

    #[inline]
    pub fn lookup(&self, table_index: TableIndex, index: usize) -> &lookup_map_t {
        &self.lookups[table_index][index]
    }

    #[inline]
    pub fn stage_lookups(&self, table_index: TableIndex, stage: usize) -> &[lookup_map_t] {
        &self.lookups[table_index][self.stage_lookup_range(table_index, stage)]
    }

    #[inline]
    pub fn stage_lookup_range(&self, table_index: TableIndex, stage: usize) -> Range<usize> {
        let stages = &self.stages[table_index];
        let lookups = &self.lookups[table_index];
        let start = stage
            .checked_sub(1)
            .map_or(0, |prev| stages[prev].last_lookup);
        let end = stages
            .get(stage)
            .map_or(lookups.len(), |curr| curr.last_lookup);
        start..end
    }

    pub fn feature_variations(&self) -> &[Option<u32>; 2] {
        &self.feature_variations
    }
}

pub type hb_ot_map_feature_flags_t = u32;
pub const F_NONE: u32 = 0x0000;
pub const F_GLOBAL: u32 = 0x0001; /* Feature applies to all characters; results in no mask allocated for it. */
pub const F_HAS_FALLBACK: u32 = 0x0002; /* Has fallback implementation, so include mask bit even if feature not found. */
pub const F_MANUAL_ZWNJ: u32 = 0x0004; /* Don't skip over ZWNJ when matching **context**. */
pub const F_MANUAL_ZWJ: u32 = 0x0008; /* Don't skip over ZWJ when matching **input**. */
pub const F_MANUAL_JOINERS: u32 = F_MANUAL_ZWNJ | F_MANUAL_ZWJ;
pub const F_GLOBAL_MANUAL_JOINERS: u32 = F_GLOBAL | F_MANUAL_JOINERS;
pub const F_GLOBAL_HAS_FALLBACK: u32 = F_GLOBAL | F_HAS_FALLBACK;
pub const F_GLOBAL_SEARCH: u32 = 0x0010; /* If feature not found in LangSys, look for it in global feature list and pick one. */
pub const F_RANDOM: u32 = 0x0020; /* Randomly select a glyph from an AlternateSubstFormat1 subtable. */
pub const F_OPTIONAL: u32 = 0x0080; /* Affirmative selected-plan optional role. */
pub const F_REQUIRED: u32 = 0x0100; /* Required shaping role; explicit requests still apply. */
const F_SCRIPT_OVERRIDE: u32 = 0x0400;
pub const F_REQUIRED_ALWAYS: u32 = 0x0200; /* Script override must remain enabled. */
pub const F_PER_SYLLABLE: u32 = 0x0040; /* Contain lookup application to within syllable. */

pub struct hb_ot_map_builder_t<'a> {
    face: &'a hb_font_t<'a>,
    found_script: [bool; 2],
    script_index: [Option<u16>; 2],
    chosen_script: [Option<hb_tag_t>; 2],
    lang_index: [Option<u16>; 2],
    current_stage: [usize; 2],
    feature_infos: Vec<feature_info_t>,
    requests: Vec<Feature>,
    stages: [Vec<stage_info_t>; 2],
    pub(crate) is_simple: bool,
    pub(crate) allow_optional_defaults: bool,
    pub(crate) collecting_overrides: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct feature_info_t {
    tag: hb_tag_t,
    // sequence number, used for stable sorting only
    seq: usize,
    max_value: u32,
    flags: hb_ot_map_feature_flags_t,
    // for non-global features, what should the unset glyphs take
    default_value: u32,
    // GSUB/GPOS
    stage: [usize; 2],
    has_range: bool,
}

#[derive(Clone, Copy)]
struct stage_info_t {
    index: usize,
    pause_func: Option<pause_func_t>,
}

const GLOBAL_BIT_SHIFT: u32 = 8 * size_of::<u32>() as u32 - 1;
const GLOBAL_BIT_MASK: hb_mask_t = 1 << GLOBAL_BIT_SHIFT;

impl<'a> hb_ot_map_builder_t<'a> {
    pub fn new(
        face: &'a hb_font_t<'a>,
        script: Option<Script>,
        language: Option<&Language>,
    ) -> Self {
        // Fetch script/language indices for GSUB/GPOS.  We need these later to skip
        // features not available in either table and not waste precious bits for them.
        let (script_tags, lang_tags) = tag::tags_from_script_and_language(script, language);

        let mut found_script = [false; 2];
        let mut script_index = [None; 2];
        let mut chosen_script = [None; 2];
        let mut lang_index = [None; 2];

        for (table_index, table) in face.layout_tables() {
            if let Some((found, idx, tag)) = table.select_script(&script_tags) {
                chosen_script[table_index] = Some(tag);
                found_script[table_index] = found;
                script_index[table_index] = Some(idx);

                if let Some(idx) = table.select_script_language(idx, &lang_tags) {
                    lang_index[table_index] = Some(idx);
                }
            }
        }

        Self {
            face,
            found_script,
            script_index,
            chosen_script,
            lang_index,
            current_stage: [0, 0],
            feature_infos: Vec::new(),
            requests: Vec::new(),
            stages: [Vec::new(), Vec::new()],
            is_simple: false,
            allow_optional_defaults: true,
            collecting_overrides: false,
        }
    }

    #[inline]
    pub fn chosen_script(&self, table_index: TableIndex) -> Option<hb_tag_t> {
        self.chosen_script[table_index]
    }

    #[inline]
    pub fn has_feature(&self, tag: hb_tag_t) -> bool {
        for (table_index, table) in self.face.layout_tables() {
            if let Some(script_index) = self.script_index[table_index] {
                if table
                    .find_language_feature(script_index, self.lang_index[table_index], tag)
                    .is_some()
                {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    pub fn add_feature(&mut self, tag: hb_tag_t, flags: hb_ot_map_feature_flags_t, value: u32) {
        if !tag.is_null() {
            let seq = self.feature_infos.len();
            self.feature_infos.push(feature_info_t {
                tag,
                seq,
                max_value: value,
                flags: flags
                    | if self.collecting_overrides {
                        F_SCRIPT_OVERRIDE
                    } else {
                        0
                    },
                default_value: if flags & F_GLOBAL != 0 { value } else { 0 },
                stage: self.current_stage,
                has_range: false,
            });
        }
    }

    pub(crate) fn set_user_features(&mut self, features: &[Feature]) {
        self.requests = canonical_features(features).copied().collect();
    }

    #[inline]
    pub fn enable_feature(&mut self, tag: hb_tag_t, flags: hb_ot_map_feature_flags_t, value: u32) {
        self.add_feature(tag, flags | F_GLOBAL, value);
    }

    #[inline]
    pub fn disable_feature(&mut self, tag: hb_tag_t) {
        self.add_feature(tag, F_GLOBAL, 0);
    }

    #[inline]
    pub fn add_gsub_pause(&mut self, pause: Option<pause_func_t>) {
        self.add_pause(TableIndex::GSUB, pause);
    }

    #[inline]
    pub fn add_gpos_pause(&mut self, pause: Option<pause_func_t>) {
        self.add_pause(TableIndex::GPOS, pause);
    }

    fn add_pause(&mut self, table_index: TableIndex, pause: Option<pause_func_t>) {
        self.stages[table_index].push(stage_info_t {
            index: self.current_stage[table_index],
            pause_func: pause,
        });

        self.current_stage[table_index] += 1;
    }

    pub fn compile(&mut self) -> hb_ot_map_t {
        // We default to applying required feature in stage 0.  If the required
        // feature has a tag that is known to the shaper, we apply required feature
        // in the stage for that tag.
        let mut required_index = [None; 2];
        let mut required_tag = [None; 2];

        for (table_index, table) in self.face.layout_tables() {
            if let Some(script) = self.script_index[table_index] {
                let lang = self.lang_index[table_index];
                if let Some((idx, tag)) = table.get_required_language_feature(script, lang) {
                    required_index[table_index] = Some(idx);
                    required_tag[table_index] = Some(tag);
                }
            }
        }

        // Provider registrations and selected language systems establish role;
        // request tags and F_GLOBAL never manufacture optionality.
        let provider_infos = self.feature_infos.clone();
        self.feature_infos
            .retain(|info| info.flags & F_SCRIPT_OVERRIDE == 0);
        let original_requests = core::mem::take(&mut self.requests);
        let mut default_features = Vec::new();
        for request in original_requests {
            if request.origin() == FeatureOrigin::OptionalDefault {
                let disposition = if required_tag.contains(&Some(request.tag))
                    || provider_infos
                        .iter()
                        .any(|info| info.tag == request.tag && info.flags & F_REQUIRED != 0)
                {
                    DefaultFeatureDisposition::Required
                } else if self.allow_optional_defaults
                    && provider_infos.iter().any(|info| {
                        info.tag == request.tag
                            && info.flags & F_OPTIONAL != 0
                            && self.feature_indices(info).iter().any(Option::is_some)
                    })
                {
                    DefaultFeatureDisposition::Accepted
                } else {
                    DefaultFeatureDisposition::Unclassified
                };
                default_features.push(DefaultFeature {
                    request,
                    disposition,
                });
                if disposition != DefaultFeatureDisposition::Accepted {
                    continue;
                }
            }
            let flags = if request.is_global() {
                F_GLOBAL
            } else {
                F_NONE
            };
            if !request.tag.is_null() {
                self.add_feature(request.tag, flags, request.value);
                if let Some(info) = self.feature_infos.last_mut() {
                    info.has_range = !request.is_global();
                }
            }
            self.requests.push(request);
        }
        for info in provider_infos
            .iter()
            .filter(|info| info.flags & F_SCRIPT_OVERRIDE != 0)
        {
            let mut info = *info;
            info.seq = self.feature_infos.len();
            self.feature_infos.push(info);
        }
        let (features, required_stage, global_mask, capacity_loss) =
            self.collect_feature_maps(required_tag);

        self.add_gsub_pause(None);
        self.add_gpos_pause(None);

        let (lookups, stages) =
            self.collect_lookup_stages(&features, required_index, required_stage, &provider_infos);
        let provenance = self.collect_provenance(
            &features,
            required_index,
            required_tag,
            required_stage,
            &provider_infos,
        );

        hb_ot_map_t {
            found_script: self.found_script,
            chosen_script: self.chosen_script,
            global_mask,
            features,
            lookups,
            stages,
            feature_variations: self.face.ot_tables.feature_variations,
            capacity_loss,
            requests: core::mem::take(&mut self.requests),
            default_features,
            provenance,
        }
    }

    fn collect_feature_maps(
        &mut self,
        required_tag: [Option<hb_tag_t>; 2],
    ) -> (
        Vec<feature_map_t>,
        [usize; 2],
        hb_mask_t,
        FeatureCapacityLoss,
    ) {
        let mut capacity_loss = FeatureCapacityLoss::default();
        let mut map_features = Vec::new();
        let mut required_stage = [0; 2];
        let mut global_mask = GLOBAL_BIT_MASK;
        let mut next_bit = glyph_flag::DEFINED.count_ones() + 1;

        // Sort features and merge duplicates.
        self.dedup_feature_infos();

        for info in &self.feature_infos {
            let bits_needed = if info.flags & F_GLOBAL != 0 && info.max_value == 1 {
                // Uses the global bit.
                0
            } else {
                // Limit bits per feature.
                let v = info.max_value;
                let num_bits = 8 * size_of_val(&v) as u32 - v.leading_zeros();
                hb_ot_map_t::MAX_BITS.min(num_bits)
            };

            if info.max_value == 0 {
                continue;
            }
            if next_bit + bits_needed >= GLOBAL_BIT_SHIFT {
                // Preserve the original early allocation refusal and stage state;
                // resolve real feature presence only for diagnostic metadata.
                capacity_loss.record(info, self.feature_indices(info));
                continue;
            }

            for (table_index, _) in self.face.layout_tables() {
                if required_tag[table_index] == Some(info.tag) {
                    required_stage[table_index] = info.stage[table_index];
                }
            }
            let feature_index = self.feature_indices(info);
            let found = feature_index.iter().any(Option::is_some);

            if !found && !info.flags & F_HAS_FALLBACK != 0 {
                continue;
            }

            if info.max_value > hb_ot_map_t::MAX_VALUE {
                capacity_loss.record(info, feature_index);
            }

            let (shift, mask) = if info.flags & F_GLOBAL != 0 && info.max_value == 1 {
                // Uses the global bit
                (GLOBAL_BIT_SHIFT, GLOBAL_BIT_MASK)
            } else {
                let shift = next_bit;
                let mask = (1 << (next_bit + bits_needed)) - (1 << next_bit);
                next_bit += bits_needed;
                global_mask |= (info.default_value << shift) & mask;
                (shift, mask)
            };

            map_features.push(feature_map_t {
                tag: info.tag,
                index: feature_index,
                stage: info.stage,
                shift,
                mask,
                one_mask: (1 << shift) & mask,
                auto_zwnj: info.flags & F_MANUAL_ZWNJ == 0,
                auto_zwj: info.flags & F_MANUAL_ZWJ == 0,
                random: info.flags & F_RANDOM != 0,
                per_syllable: info.flags & F_PER_SYLLABLE != 0,
            });
        }

        if self.is_simple {
            map_features.sort();
        }

        (map_features, required_stage, global_mask, capacity_loss)
    }

    fn feature_indices(&self, info: &feature_info_t) -> [Option<u16>; 2] {
        let mut indices = [None; 2];
        for (side, table) in self.face.layout_tables() {
            if let Some(script) = self.script_index[side] {
                indices[side] =
                    table.find_language_feature(script, self.lang_index[side], info.tag);
            }
        }
        if indices.iter().all(Option::is_none) && info.flags & F_GLOBAL_SEARCH != 0 {
            for (side, table) in self.face.layout_tables() {
                indices[side] = table.feature_index(info.tag);
            }
        }
        indices
    }

    fn dedup_feature_infos(&mut self) {
        let feature_infos = &mut self.feature_infos;
        if feature_infos.is_empty() {
            return;
        }

        if !self.is_simple {
            feature_infos.sort();
        }

        let mut j = 0;
        for i in 1..feature_infos.len() {
            if feature_infos[i].tag != feature_infos[j].tag {
                j += 1;
                feature_infos[j] = feature_infos[i];
            } else {
                if feature_infos[i].flags & F_GLOBAL != 0 {
                    if feature_infos[i].flags & F_SCRIPT_OVERRIDE != 0
                        && feature_infos[i].flags & F_REQUIRED_ALWAYS == 0
                    {
                        feature_infos[j].has_range = false;
                    }
                    if feature_infos[j].has_range {
                        feature_infos[j].max_value =
                            feature_infos[j].max_value.max(feature_infos[i].max_value);
                    } else {
                        feature_infos[j].flags |= F_GLOBAL;
                        feature_infos[j].max_value = feature_infos[i].max_value;
                    }
                    feature_infos[j].default_value = feature_infos[i].default_value;
                } else {
                    if feature_infos[j].flags & F_GLOBAL != 0 {
                        feature_infos[j].flags ^= F_GLOBAL;
                    }
                    feature_infos[j].max_value =
                        feature_infos[j].max_value.max(feature_infos[i].max_value);
                    // Inherit default_value from j
                }
                feature_infos[j].has_range |= feature_infos[i].has_range;
                if feature_infos[j].has_range {
                    feature_infos[j].flags &= !F_GLOBAL;
                }
                let flags = feature_infos[i].flags
                    & (F_HAS_FALLBACK | F_OPTIONAL | F_REQUIRED | F_REQUIRED_ALWAYS);
                feature_infos[j].flags |= flags;
                feature_infos[j].stage[0] =
                    feature_infos[j].stage[0].min(feature_infos[i].stage[0]);
                feature_infos[j].stage[1] =
                    feature_infos[j].stage[1].min(feature_infos[i].stage[1]);
            }
        }

        feature_infos.truncate(j + 1);
    }

    fn collect_lookup_stages(
        &self,
        map_features: &[feature_map_t],
        required_feature_index: [Option<u16>; 2],
        required_feature_stage: [usize; 2],
        provider_infos: &[feature_info_t],
    ) -> ([Vec<lookup_map_t>; 2], [Vec<StageMap>; 2]) {
        let mut map_lookups = [Vec::new(), Vec::new()];
        let mut map_stages = [Vec::new(), Vec::new()];

        for table_index in TableIndex::iter() {
            // Collect lookup indices for features.
            let mut stage_index = 0;
            let mut last_lookup = 0;

            let variation_index = self.face.ot_tables.feature_variations[table_index as usize];

            for stage in 0..self.current_stage[table_index] {
                if let Some(feature_index) = required_feature_index[table_index] {
                    if required_feature_stage[table_index] == stage {
                        self.add_lookups(
                            &mut map_lookups[table_index],
                            table_index,
                            feature_index,
                            variation_index,
                            GLOBAL_BIT_MASK,
                            true,
                            true,
                            false,
                            false,
                        );
                    }
                }

                // Script-required registrations remain a separate enabled
                // contribution, just like the selected LangSys requirement.
                for info in provider_infos
                    .iter()
                    .filter(|info| info.flags & F_REQUIRED_ALWAYS != 0)
                {
                    if info.stage[table_index] == stage {
                        if let Some(index) = self.feature_indices(info)[table_index] {
                            self.add_lookups(
                                &mut map_lookups[table_index],
                                table_index,
                                index,
                                variation_index,
                                GLOBAL_BIT_MASK,
                                info.flags & F_MANUAL_ZWNJ == 0,
                                info.flags & F_MANUAL_ZWJ == 0,
                                false,
                                info.flags & F_PER_SYLLABLE != 0,
                            );
                        }
                    }
                }

                for feature in map_features {
                    if let Some(feature_index) = feature.index[table_index] {
                        if feature.stage[table_index] == stage {
                            self.add_lookups(
                                &mut map_lookups[table_index],
                                table_index,
                                feature_index,
                                variation_index,
                                feature.mask,
                                feature.auto_zwnj,
                                feature.auto_zwj,
                                feature.random,
                                feature.per_syllable,
                            );
                        }
                    }
                }

                // Sort lookups and merge duplicates.
                let lookups = &mut map_lookups[table_index];
                let len = lookups.len();

                if last_lookup + 1 < len {
                    lookups[last_lookup..].sort();

                    let mut j = last_lookup;
                    for i in j + 1..len {
                        if lookups[i].index != lookups[j].index {
                            j += 1;
                            lookups[j] = lookups[i];
                        } else {
                            lookups[j].mask |= lookups[i].mask;
                            lookups[j].auto_zwnj &= lookups[i].auto_zwnj;
                            lookups[j].auto_zwj &= lookups[i].auto_zwj;
                        }
                    }

                    lookups.truncate(j + 1);
                }

                last_lookup = lookups.len();

                if let Some(info) = self.stages[table_index].get(stage_index) {
                    if info.index == stage {
                        map_stages[table_index].push(StageMap {
                            last_lookup,
                            pause_func: info.pause_func,
                        });

                        stage_index += 1;
                    }
                }
            }
        }

        (map_lookups, map_stages)
    }

    fn collect_provenance(
        &self,
        features: &[feature_map_t],
        required_index: [Option<u16>; 2],
        required_tag: [Option<hb_tag_t>; 2],
        required_stage: [usize; 2],
        providers: &[feature_info_t],
    ) -> [Vec<LookupFeature>; 2] {
        let mut result = [Vec::new(), Vec::new()];
        for table in TableIndex::iter() {
            if let (Some(index), Some(tag)) = (required_index[table], required_tag[table]) {
                let (lookups, width_rejected) = self.feature_lookups(table, index);
                result[table].push(LookupFeature {
                    tag,
                    stage: required_stage[table],
                    role: FeatureRole::Required,
                    source: FeatureSource::LanguageSystem,
                    mask: GLOBAL_BIT_MASK,
                    lookups,
                    width_rejected,
                    fixed: true,
                });
            }
            for info in providers
                .iter()
                .filter(|info| info.flags & F_REQUIRED_ALWAYS != 0)
            {
                if let Some(index) = self.feature_indices(info)[table] {
                    let (lookups, width_rejected) = self.feature_lookups(table, index);
                    result[table].push(LookupFeature {
                        tag: info.tag,
                        stage: info.stage[table],
                        role: FeatureRole::Required,
                        source: FeatureSource::Script,
                        mask: GLOBAL_BIT_MASK,
                        lookups,
                        width_rejected,
                        fixed: true,
                    });
                }
            }
            for feature in features {
                if let Some(index) = feature.index[table] {
                    let (lookups, width_rejected) = self.feature_lookups(table, index);
                    let required = required_tag[table] == Some(feature.tag)
                        || providers
                            .iter()
                            .any(|info| info.tag == feature.tag && info.flags & F_REQUIRED != 0);
                    let role = if required {
                        FeatureRole::Required
                    } else if providers
                        .iter()
                        .any(|info| info.tag == feature.tag && info.flags & F_OPTIONAL != 0)
                    {
                        FeatureRole::Optional
                    } else {
                        FeatureRole::Unclassified
                    };
                    result[table].push(LookupFeature {
                        tag: feature.tag,
                        stage: feature.stage[table],
                        role,
                        source: FeatureSource::Provider,
                        mask: feature.mask,
                        lookups,
                        width_rejected,
                        fixed: false,
                    });
                }
            }
        }
        result
    }

    fn feature_lookups(&self, side: TableIndex, index: u16) -> (Vec<u16>, Vec<u16>) {
        let Some(table) = self.face.layout_table(side) else {
            return (Vec::new(), Vec::new());
        };
        let variation = self.face.ot_tables.feature_variations[side as usize];
        let feature = variation
            .and_then(|variation| table.feature_substitution(variation, index))
            .or_else(|| table.feature(index));
        let mut valid = Vec::new();
        let mut rejected = Vec::new();
        if let Some(feature) = feature {
            // Preserve legacy public provenance's admitted references, while a
            // demanded width query can still distinguish rejection from absence.
            let mut count = None;
            for index in feature.lookup_list_indices() {
                let index = index.get();
                let limit = count.get_or_insert_with(|| table.lookup_count_admission().ok());
                if limit.is_some_and(|count| index < count) {
                    valid.push(index);
                } else {
                    rejected.push(index);
                }
            }
        }
        (valid, rejected)
    }

    fn add_lookups(
        &self,
        lookups: &mut Vec<lookup_map_t>,
        table_index: TableIndex,
        feature_index: u16,
        variation_index: Option<u32>,
        mask: hb_mask_t,
        auto_zwnj: bool,
        auto_zwj: bool,
        random: bool,
        per_syllable: bool,
    ) -> Option<()> {
        let table = self.face.layout_table(table_index)?;

        let lookup_count = table.lookup_count();
        let feature = match variation_index {
            Some(idx) => table
                .feature_substitution(idx, feature_index)
                .or_else(|| table.feature(feature_index))?,
            None => table.feature(feature_index)?,
        };

        for index in feature.lookup_list_indices() {
            let index = index.get();
            if index < lookup_count {
                lookups.push(lookup_map_t {
                    index,
                    auto_zwnj,
                    auto_zwj,
                    random,
                    mask,
                    per_syllable,
                });
            }
        }

        Some(())
    }
}

#[cfg(test)]
#[path = "ot_map_completion_tests.rs"]
mod completion_tests;

#[cfg(test)]
#[path = "feature_origin_tests.rs"]
mod feature_origin_tests;

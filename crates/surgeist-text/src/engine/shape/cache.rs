// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Adapted from Parley 0.9.0 shape/cache.rs. Keys retain actual resources and
// catalogue publication; no borrowed query result or bare resource ID escapes.

use crate::FontData;

#[derive(Clone)]
pub(super) struct ResourceKey(pub(super) FontData);

impl PartialEq for ResourceKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.face().is_same_face(&other.0.face())
            && self.0.catalogue_identity() == other.0.catalogue_identity()
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct InstanceKey {
    pub(super) resource: ResourceKey,
    pub(super) synthesis: crate::FontSynthesis,
    pub(super) suggested_variations: Vec<(surgeist_shaper::Tag, f32)>,
    pub(super) variations: Vec<parlance::FontVariation>,
    pub(super) coords: Vec<i16>,
}

#[derive(PartialEq)]
pub(super) struct PlanKey {
    pub(super) instance: InstanceKey,
    pub(super) direction: surgeist_shaper::Direction,
    pub(super) script: surgeist_shaper::Script,
    pub(super) language: Option<surgeist_shaper::Language>,
    pub(super) features: Vec<surgeist_shaper::Feature>,
    pub(super) size_bits: u32,
}

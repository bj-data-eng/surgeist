//! Concrete extension objects over the existing font, counter and readonly owners.
use crate::object_operations::execute;
use crate::*;
use std::vec::IntoIter;
use surgeist_css::{
    CssCounterStyleDescriptorKind, CssFontFeatureValueKind, CssSerializationLimits,
};

/// Ordered keys owned by one immutable map observation.
pub type CssFontFeatureValuesKeys = IntoIter<String>;
/// Ordered projected values; each authored conversion failure remains visible.
pub type CssFontFeatureValuesValues = IntoIter<Result<Vec<u32>, CssomError>>;
/// Ordered projected entries from one immutable map observation.
pub type CssFontFeatureValuesEntries = IntoIter<Result<(String, Vec<u32>), CssomError>>;

// Snapshot map projections allocate only the observed bounded entries, using
// fallible capacity admission. They do not retain a live store borrow.
fn copy_string(value: &str) -> Result<String, CssomError> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| CssomError::Limit {
            resource: "extension string capacity",
            maximum: value.len(),
        })?;
    result.push_str(value);
    Ok(result)
}
fn copy_values(value: &[u32]) -> Result<Vec<u32>, CssomError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| CssomError::Limit {
            resource: "feature values capacity",
            maximum: value.len(),
        })?;
    result.extend_from_slice(value);
    Ok(result)
}
fn projected(value: &CssomFeatureValue) -> Result<Vec<u32>, CssomError> {
    value.projected().and_then(copy_values)
}
fn entries_capacity<T>(count: usize) -> Result<Vec<T>, CssomError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| CssomError::Limit {
            resource: "feature map snapshot capacity",
            maximum: count,
        })?;
    Ok(result)
}

impl CssFontFaceRule {
    pub fn style(&self, store: &CssomStore) -> Result<CssFontFaceDescriptors, CssomError> {
        Self::from_state(&store.state, self.id())?;
        let CssomRuleData::FontFace { block } = store.state.rule(self.id())?.data() else {
            unreachable!()
        };
        CssFontFaceDescriptors::from_state(&store.state, block)
    }
}
impl CssomCapture<CssFontFaceRule> {
    pub fn style(&self) -> Result<CssomCapture<CssFontFaceDescriptors>, CssomError> {
        let CssomRuleData::FontFace { block } = self.snapshot.rule(self.handle.id())?.data() else {
            return Err(CssomError::WrongKind);
        };
        CssFontFaceDescriptors::from_state(&self.snapshot.state, block)?.capture(&self.snapshot)
    }
}

macro_rules! feature_maps {
    ($($method:ident => $kind:ident),+ $(,)?) => {
        impl CssFontFeatureValuesRule { $(
            /// The product-stable map identity, read from the supplied live owner.
            pub fn $method(&self, store: &CssomStore) -> Result<CssFontFeatureValuesMap, CssomError> {
                Self::from_state(&store.state, self.id())?;
                let snapshot = store.snapshot();
                let view = snapshot.inspect_font_feature_values(self.id())?;
                CssFontFeatureValuesMap::from_state(&snapshot.state, view.map(CssFontFeatureValueKind::$kind))
            }
        )+ }
        impl CssomCapture<CssFontFeatureValuesRule> { $(
            /// Child map reads retain this exact owning snapshot.
            pub fn $method(&self) -> Result<CssomCapture<CssFontFeatureValuesMap>, CssomError> {
                let view = self.snapshot.inspect_font_feature_values(self.handle.id())?;
                CssFontFeatureValuesMap::from_state(&self.snapshot.state, view.map(CssFontFeatureValueKind::$kind))?.capture(&self.snapshot)
            }
        )+ }
    };
}
feature_maps! {
    annotation => Annotation, ornaments => Ornaments, stylistic => Stylistic,
    swash => Swash, character_variant => CharacterVariant, styleset => Styleset,
    historical_forms => HistoricalForms,
}
impl CssFontFeatureValuesRule {
    pub fn font_family(
        &self,
        store: &CssomStore,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        Self::from_state(&store.state, self.id())?;
        store
            .snapshot()
            .inspect_font_feature_values(self.id())?
            .font_family(limits)
    }
    pub fn set_font_family(
        &self,
        store: &mut CssomStore,
        source: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            let (_, diagnostics) = batch.set_font_feature_family_operation(self.id(), source)?;
            Ok(CssomStagedOperation::new((), diagnostics))
        })
    }
}
impl CssomCapture<CssFontFeatureValuesRule> {
    pub fn font_family(&self, limits: CssSerializationLimits) -> Result<String, CssomError> {
        self.snapshot
            .inspect_font_feature_values(self.handle.id())?
            .font_family(limits)
    }
}

impl CssomCapture<CssFontFeatureValuesMap> {
    fn data(&self) -> Result<&CssomFeatureMap, CssomError> {
        self.snapshot.feature_map(self.handle.id())
    }
    pub fn size(&self) -> Result<usize, CssomError> {
        Ok(self.data()?.size())
    }
    pub fn has(&self, key: &str) -> Result<bool, CssomError> {
        Ok(self.data()?.has(key))
    }
    pub fn get(&self, key: &str) -> Result<Option<Vec<u32>>, CssomError> {
        self.data()?.get(key)?.map(copy_values).transpose()
    }
    /// Ordered keys copied from this snapshot; later live edits cannot affect iteration.
    pub fn keys(&self) -> Result<CssFontFeatureValuesKeys, CssomError> {
        let data = self.data()?;
        let mut result = entries_capacity(data.size())?;
        for entry in data.raw_entries() {
            result.push(copy_string(&entry.key)?);
        }
        Ok(result.into_iter())
    }
    /// Ordered snapshot values. Nonrepresentable authored entries remain individual errors.
    pub fn values(&self) -> Result<CssFontFeatureValuesValues, CssomError> {
        let data = self.data()?;
        let mut result = entries_capacity(data.size())?;
        for entry in data.raw_entries() {
            result.push(projected(&entry.value));
        }
        Ok(result.into_iter())
    }
    /// Ordered projected snapshot entries, with the same conversion failure as get/values.
    /// Exact authored occurrences remain in snapshot().feature_map(id).authored().
    pub fn entries(&self) -> Result<CssFontFeatureValuesEntries, CssomError> {
        let data = self.data()?;
        let mut result = entries_capacity(data.size())?;
        for entry in data.raw_entries() {
            let key = copy_string(&entry.key)?;
            result.push(projected(&entry.value).map(|value| (key, value)));
        }
        Ok(result.into_iter())
    }
}
impl CssFontFeatureValuesMap {
    pub fn size(&self, store: &CssomStore) -> Result<usize, CssomError> {
        self.capture(&store.snapshot())?.size()
    }
    pub fn has(&self, store: &CssomStore, key: &str) -> Result<bool, CssomError> {
        self.capture(&store.snapshot())?.has(key)
    }
    pub fn get(&self, store: &CssomStore, key: &str) -> Result<Option<Vec<u32>>, CssomError> {
        self.capture(&store.snapshot())?.get(key)
    }
    /// Observe the live owner once and return an owning snapshot iterator.
    pub fn keys(&self, store: &CssomStore) -> Result<CssFontFeatureValuesKeys, CssomError> {
        self.capture(&store.snapshot())?.keys()
    }
    /// Observe the live owner once; per-entry conversion errors are retained.
    pub fn values(&self, store: &CssomStore) -> Result<CssFontFeatureValuesValues, CssomError> {
        self.capture(&store.snapshot())?.values()
    }
    /// Observe the live owner once; this is not a reentrant JavaScript iterator.
    pub fn entries(&self, store: &CssomStore) -> Result<CssFontFeatureValuesEntries, CssomError> {
        self.capture(&store.snapshot())?.entries()
    }
    pub fn set(
        &self,
        store: &mut CssomStore,
        key: &str,
        value: CssomFeatureValues,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            batch.set_feature(self.id(), key, value.as_sequence())?;
            Ok(CssomStagedOperation::new((), Vec::new()))
        })
    }
    pub fn delete(
        &self,
        store: &mut CssomStore,
        key: &str,
    ) -> Result<CssomOperationOutcome<bool>, CssomError> {
        execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            Ok(CssomStagedOperation::new(
                batch.delete_feature(self.id(), key)?,
                Vec::new(),
            ))
        })
    }
    pub fn clear(&self, store: &mut CssomStore) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            batch.clear_feature_map(self.id())?;
            Ok(CssomStagedOperation::new((), Vec::new()))
        })
    }
}

macro_rules! palette_fields {
    ($($method:ident),+) => {
        impl CssFontPaletteValuesRule { $(
            pub fn $method(&self, store: &CssomStore, limits: CssSerializationLimits) -> Result<String, CssomError> {
                Self::from_state(&store.state, self.id())?;
                store.snapshot().palette_values(self.id())?.$method(limits)
            }
        )+ }
        impl CssomCapture<CssFontPaletteValuesRule> { $(
            pub fn $method(&self, limits: CssSerializationLimits) -> Result<String, CssomError> {
                self.snapshot.palette_values(self.handle.id())?.$method(limits)
            }
        )+ }
    };
}
palette_fields!(font_family, base_palette, override_colors);
impl CssFontPaletteValuesRule {
    pub fn name(&self, store: &CssomStore) -> Result<String, CssomError> {
        Self::from_state(&store.state, self.id())?;
        copy_string(store.snapshot().palette_values(self.id())?.name())
    }
}
impl CssomCapture<CssFontPaletteValuesRule> {
    pub fn name(&self) -> Result<String, CssomError> {
        copy_string(self.snapshot.palette_values(self.handle.id())?.name())
    }
}

macro_rules! counter_fields {
    ($($get:ident, $set:ident => $kind:ident),+ $(,)?) => {
        impl CssCounterStyleRule { $(
            pub fn $get(&self, store: &CssomStore, limits: CssSerializationLimits) -> Result<String, CssomError> {
                Self::from_state(&store.state, self.id())?;
                store.snapshot().counter_descriptor(self.id(), CssCounterStyleDescriptorKind::$kind, limits)
            }
            pub fn $set(&self, store: &mut CssomStore, value: &str, limits: CssomDeclarationRequestLimits) -> Result<CssomOperationOutcome<()>, CssomError> {
                execute(store, |batch| {
                    Self::from_state(&batch.staged, self.id())?;
                    let (_, diagnostics) = batch.set_counter_descriptor_operation(self.id(), CssCounterStyleDescriptorKind::$kind, value, limits)?;
                    Ok(CssomStagedOperation::new((), diagnostics))
                })
            }
        )+ }
        impl CssomCapture<CssCounterStyleRule> { $(
            pub fn $get(&self, limits: CssSerializationLimits) -> Result<String, CssomError> {
                self.snapshot.counter_descriptor(self.handle.id(), CssCounterStyleDescriptorKind::$kind, limits)
            }
        )+ }
    };
}
counter_fields! {
    system, set_system => System, symbols, set_symbols => Symbols,
    additive_symbols, set_additive_symbols => AdditiveSymbols,
    negative, set_negative => Negative, prefix, set_prefix => Prefix,
    suffix, set_suffix => Suffix, range, set_range => Range, pad, set_pad => Pad,
    speak_as, set_speak_as => SpeakAs, fallback, set_fallback => Fallback,
}
impl CssCounterStyleRule {
    pub fn name(
        &self,
        store: &CssomStore,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        Self::from_state(&store.state, self.id())?;
        store.snapshot().counter_name(self.id(), limits)
    }
    pub fn set_name(
        &self,
        store: &mut CssomStore,
        name: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            Self::from_state(&batch.staged, self.id())?;
            batch.set_counter_name(self.id(), name)?;
            Ok(CssomStagedOperation::new((), Vec::new()))
        })
    }
}
impl CssomCapture<CssCounterStyleRule> {
    pub fn name(&self, limits: CssSerializationLimits) -> Result<String, CssomError> {
        self.snapshot.counter_name(self.handle.id(), limits)
    }
}

/// A custom-media query selects a boolean or a live typed MediaList, without evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomMediaQuery {
    Boolean(bool),
    MediaList(MediaList),
}
/// The same sum read from one immutable capture. Child lists share that capture.
#[derive(Clone, Debug)]
pub enum CapturedCustomMediaQuery {
    Boolean(bool),
    MediaList(CssomCapture<MediaList>),
}
impl CssCustomMediaRule {
    pub fn name(&self, store: &CssomStore) -> Result<String, CssomError> {
        Self::from_state(&store.state, self.id())?;
        copy_string(store.snapshot().custom_media_name(self.id())?)
    }
    pub fn query(&self, store: &CssomStore) -> Result<CustomMediaQuery, CssomError> {
        Self::from_state(&store.state, self.id())?;
        Ok(match store.snapshot().custom_media_query(self.id())? {
            CssomCustomMediaQuery::Boolean(value) => CustomMediaQuery::Boolean(*value),
            CssomCustomMediaQuery::MediaList(id) => {
                CustomMediaQuery::MediaList(MediaList::from_state(&store.state, id)?)
            }
        })
    }
}
impl CssomCapture<CssCustomMediaRule> {
    pub fn name(&self) -> Result<&str, CssomError> {
        self.snapshot.custom_media_name(self.handle.id())
    }
    pub fn query(&self) -> Result<CapturedCustomMediaQuery, CssomError> {
        Ok(match self.snapshot.custom_media_query(self.handle.id())? {
            CssomCustomMediaQuery::Boolean(value) => CapturedCustomMediaQuery::Boolean(*value),
            CssomCustomMediaQuery::MediaList(id) => CapturedCustomMediaQuery::MediaList(
                MediaList::from_state(&self.snapshot.state, id)?.capture(&self.snapshot)?,
            ),
        })
    }
}
macro_rules! profile_fields {
    ($($method:ident => $helper:ident),+) => {
        impl CssColorProfileRule { $(
            pub fn $method(&self, store: &CssomStore, limits: CssSerializationLimits) -> Result<String, CssomError> {
                Self::from_state(&store.state, self.id())?;
                store.snapshot().$helper(self.id(), limits)
            }
        )+ }
        impl CssomCapture<CssColorProfileRule> { $(
            pub fn $method(&self, limits: CssSerializationLimits) -> Result<String, CssomError> { self.snapshot.$helper(self.handle.id(), limits) }
        )+ }
    };
}
profile_fields! { name => color_profile_name, src => color_profile_src, rendering_intent => color_profile_rendering_intent, components => color_profile_components }

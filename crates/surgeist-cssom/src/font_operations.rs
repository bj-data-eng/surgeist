//! Fonts CSSOM views and source map operations over the shared live owner.
use crate::*;
use surgeist_css::component::CssValueOrigin;
use surgeist_css::diagnostic::CssRecoveryDiagnostic;
use surgeist_css::rule::CssRule;
use surgeist_css::rule::font_face::{
    CssFontFaceDescriptorKind, CssFontFaceFamily, serialize_font_face_family_list,
};
use surgeist_css::rule::font_feature_values::{
    CssFontFeatureDisplayOccurrence, CssFontFeatureValueKind, CssFontFeatureValuesFamilyList,
};
use surgeist_css::rule::font_palette_values::CssFontPaletteDescriptorKind;
use surgeist_css::rule::font_palette_values::CssFontPaletteValuesRule;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::{CssParser, CssParserContext};

/// Genuine checked family admission, independent of the original rule occurrence.
/// Parsed carriers keep their actual whole/member origins; typed carriers retain
/// their own provenance without being reparsed in the current parser context.
#[derive(Clone, Debug, PartialEq)]
pub struct CssomFontFamilyInput {
    value: CssFontFeatureValuesFamilyList,
    parser_context: CssParserContext,
    inputs: Vec<CssomInputVersion>,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}
impl CssomFontFamilyInput {
    pub fn value(&self) -> &CssFontFeatureValuesFamilyList {
        &self.value
    }
    pub fn parser_context(&self) -> CssParserContext {
        self.parser_context
    }
    pub fn inputs(&self) -> &[CssomInputVersion] {
        &self.inputs
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
    pub(crate) fn source_bytes(&self) -> usize {
        match self.value.origin() {
            CssValueOrigin::Parsed(origin) => origin.source().as_str().len(),
            _ => 0,
        }
    }
}

/// The selected map setter's unsigned-long-or-sequence input. Scalar conversion
/// is a singleton; it does not apply authored descriptor index grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomFeatureValues {
    Scalar(u32),
    Sequence(Vec<u32>),
}
impl CssomFeatureValues {
    /// Passes either source input form to the existing `set_feature` owner.
    pub fn as_sequence(&self) -> &[u32] {
        match self {
            Self::Scalar(value) => std::slice::from_ref(value),
            Self::Sequence(values) => values,
        }
    }
}

/// A FontFeatureValuesRule's current families and seven product-stable maps.
/// The selected source does not specify SameObject for its map attributes.
pub struct CssomFontFeatureValuesView<'a> {
    snapshot: &'a CssomSnapshot,
    families: &'a [CssFontFaceFamily],
    input: Option<&'a CssomFontFamilyInput>,
    maps: &'a [CssomFeatureMapId],
    display: &'a [CssFontFeatureDisplayOccurrence],
}
impl<'a> CssomFontFeatureValuesView<'a> {
    pub fn families(&self) -> &'a [CssFontFaceFamily] {
        self.families
    }
    pub fn family_input(&self) -> Option<&'a CssomFontFamilyInput> {
        self.input
    }
    /// One native cumulative list writer over the current literal families.
    pub(crate) fn font_family(&self, limits: CssSerializationLimits) -> Result<String, CssomError> {
        serialize_font_face_family_list(self.families, limits).map_err(CssomError::Value)
    }
    /// Exact authored display occurrences, independently of current maps.
    pub fn font_display(&self) -> &'a [CssFontFeatureDisplayOccurrence] {
        self.display
    }
    pub fn map(&self, kind: CssFontFeatureValueKind) -> &'a CssomFeatureMapId {
        self.maps
            .iter()
            .find(|id| self.snapshot.feature_map(id).expect("owned map").kind() == kind)
            .expect("all seven maps are retained")
    }
}

/// Readonly palette attributes retain the current checked descriptor owner.
pub(crate) struct CssomFontPaletteValuesView<'a> {
    value: &'a CssFontPaletteValuesRule,
}
impl CssomFontPaletteValuesView<'_> {
    pub fn name(&self) -> &str {
        self.value.name().as_str()
    }
    pub fn font_family(&self, limits: CssSerializationLimits) -> Result<String, CssomError> {
        self.descriptor(CssFontPaletteDescriptorKind::FontFamily, limits)
    }
    pub fn base_palette(&self, limits: CssSerializationLimits) -> Result<String, CssomError> {
        self.descriptor(CssFontPaletteDescriptorKind::BasePalette, limits)
    }
    pub fn override_colors(&self, limits: CssSerializationLimits) -> Result<String, CssomError> {
        self.descriptor(CssFontPaletteDescriptorKind::OverrideColors, limits)
    }
    fn descriptor(
        &self,
        kind: CssFontPaletteDescriptorKind,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        self.value
            .descriptors()
            .iter()
            .rev()
            .find(|entry| entry.value().kind() == kind)
            .map(|entry| {
                entry
                    .value()
                    .serialize_specified(limits)
                    .map_err(CssomError::Value)
            })
            .unwrap_or_else(|| Ok(String::new()))
    }
}

impl CssomSnapshot {
    pub fn inspect_font_feature_values(
        &self,
        id: &CssomRuleId,
    ) -> Result<CssomFontFeatureValuesView<'_>, CssomError> {
        let CssomRuleData::FontFeatureValues {
            families,
            family_input,
            maps,
            font_display,
        } = self.rule(id)?.data()
        else {
            return Err(CssomError::WrongKind);
        };
        Ok(CssomFontFeatureValuesView {
            snapshot: self,
            families,
            input: family_input.as_ref(),
            maps,
            display: font_display,
        })
    }
    pub(crate) fn palette_values(
        &self,
        id: &CssomRuleId,
    ) -> Result<CssomFontPaletteValuesView<'_>, CssomError> {
        let CssomRuleData::Leaf { current } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        let value = match current {
            CssomAuthoredRule::Rule(CssRule::FontPaletteValues(value)) => value,
            _ => return Err(CssomError::WrongKind),
        };
        Ok(CssomFontPaletteValuesView { value })
    }
}
impl CssomFeatureValue {
    pub(crate) fn projected(&self) -> Result<&[u32], CssomError> {
        match self {
            Self::Unsigned(values) => Ok(values),
            Self::Authored(_) => Err(CssomError::AuthoredFeatureConversion),
        }
    }
}
impl CssomFeatureMap {
    pub(crate) fn size(&self) -> usize {
        self.raw_entries().len()
    }
    pub(crate) fn has(&self, key: &str) -> bool {
        self.raw_entries().iter().any(|entry| entry.key == key)
    }
}

impl CssomBatch {
    /// Atomic maplike clear; original authored occurrences remain independently retained.
    pub fn clear_feature_map(&mut self, id: &CssomFeatureMapId) -> Result<bool, CssomError> {
        self.apply(|this| {
            if this.staged.feature_map(id)?.entries.is_empty() {
                return Ok(false);
            }
            this.staged
                .maps
                .get_mut(id)
                .expect("owned map")
                .entries
                .clear();
            this.affect(
                CssomObjectId::FeatureMap(id.clone()),
                &[CssomChange::FeatureMaps],
            );
            Ok(true)
        })
    }
    /// Checked typed ingress retains the carrier's honest original provenance.
    pub fn set_font_feature_families(
        &mut self,
        id: &CssomRuleId,
        value: CssFontFeatureValuesFamilyList,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            let CssomRuleData::FontFeatureValues { .. } = this.staged.rule(id)?.data() else {
                return Err(CssomError::WrongKind);
            };
            this.assign_font_families(id, value, Vec::new())
        })
    }
    /// Source fontFamily admission uses the current staged parser facts. Syntax
    /// rejection is a successful no-op; resource failure aborts the atomic batch.
    pub fn set_font_feature_family_text(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<bool, CssomError> {
        self.set_font_feature_family_operation(id, source)
            .map(|(changed, _)| changed)
    }
    pub(crate) fn set_font_feature_family_operation(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<(bool, Vec<CssRecoveryDiagnostic>), CssomError> {
        let mut operation_diagnostics = Vec::new();
        let changed = self.apply(|this| {
            let CssomRuleData::FontFeatureValues { .. } = this.staged.rule(id)?.data() else {
                return Err(CssomError::WrongKind);
            };
            let (value, diagnostics) = CssFontFeatureValuesFamilyList::parse(
                &CssParser::new(
                    this.staged.context.parser_context(),
                    crate::rule_operations::component_limits(&this.limits),
                ),
                source,
            )
            .into_parts();
            crate::parser_resources::check_resource_diagnostics(&diagnostics)?;
            operation_diagnostics = diagnostics;
            let Some(value) = value else {
                return Ok(false);
            };
            let mut retained_diagnostics = Vec::new();
            retained_diagnostics
                .try_reserve_exact(operation_diagnostics.len())
                .map_err(|_| CssomError::Limit {
                    resource: "font family diagnostics capacity",
                    maximum: operation_diagnostics.len(),
                })?;
            retained_diagnostics.extend_from_slice(&operation_diagnostics);
            this.assign_font_families(id, value, retained_diagnostics)
        })?;
        Ok((changed, operation_diagnostics))
    }
    fn assign_font_families(
        &mut self,
        id: &CssomRuleId,
        value: CssFontFeatureValuesFamilyList,
        diagnostics: Vec<CssRecoveryDiagnostic>,
    ) -> Result<bool, CssomError> {
        if value.families().len() > self.limits.max_entries {
            return Err(CssomError::Limit {
                resource: "entries",
                maximum: self.limits.max_entries,
            });
        }
        let mut bytes = 0usize;
        for family in value.families() {
            bytes = bytes
                .checked_add(family.as_str().len())
                .ok_or(CssomError::Limit {
                    resource: "host and live string bytes",
                    maximum: self.limits.max_string_bytes,
                })?;
        }
        if bytes > self.limits.max_string_bytes {
            return Err(CssomError::Limit {
                resource: "host and live string bytes",
                maximum: self.limits.max_string_bytes,
            });
        }
        let input = CssomFontFamilyInput {
            value,
            parser_context: self.staged.context.parser_context(),
            inputs: self
                .staged
                .context
                .inputs
                .iter()
                .map(|v| v.version.clone())
                .chain(self.staged.context.linked.iter().map(|v| v.version.clone()))
                .collect(),
            diagnostics,
        };
        if input.source_bytes() > self.limits.max_input_bytes {
            return Err(CssomError::Limit {
                resource: "retained parse input bytes",
                maximum: self.limits.max_input_bytes,
            });
        }
        let CssomRuleData::FontFeatureValues { families, .. } = self.staged.rule(id)?.data() else {
            unreachable!()
        };
        if families.as_slice() == input.value.families() {
            return Ok(false);
        }
        let CssomRuleData::FontFeatureValues {
            families,
            family_input,
            ..
        } = self
            .staged
            .rules
            .get_mut(id)
            .expect("owned rule")
            .data_mut()
        else {
            unreachable!()
        };
        *families = input.value.families().to_vec();
        *family_input = Some(input);
        self.affect(
            CssomObjectId::Rule(id.clone()),
            &[CssomChange::Descriptors, CssomChange::Provenance],
        );
        Ok(true)
    }
}

impl CssomBatch {
    /// Prepares a reflected descriptor setter using the same support decisions,
    /// source guards, pending selection, request budget and atomic owner as #835.
    /// The source fontStretch and fontWidth attributes both select FontWidth.
    pub fn prepare_font_face_descriptor(
        &mut self,
        id: &CssomRuleId,
        kind: CssFontFaceDescriptorKind,
        value: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomPreparedDeclaration, CssomError> {
        self.apply(|this| {
            let CssomRuleData::FontFace { block } = this.staged.rule(id)?.data() else {
                return Err(CssomError::WrongKind);
            };
            let block = block.clone();
            this.prepare_set_property(&block, kind.as_str(), value, "", limits)
        })
    }
    /// Implements style's PutForwards=cssText through the checked shared owner.
    pub fn prepare_font_face_css_text(
        &mut self,
        id: &CssomRuleId,
        source: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomPreparedDeclaration, CssomError> {
        self.apply(|this| {
            let CssomRuleData::FontFace { block } = this.staged.rule(id)?.data() else {
                return Err(CssomError::WrongKind);
            };
            let block = block.clone();
            this.prepare_css_text(&block, source, limits)
        })
    }
}

impl CssomBatch {
    pub fn set_feature(
        &mut self,
        id: &CssomFeatureMapId,
        key: &str,
        value: &[u32],
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            let map = this.staged.feature_map(id)?;
            let max = match map.kind {
                CssFontFeatureValueKind::HistoricalForms | CssFontFeatureValueKind::Styleset => {
                    usize::MAX
                }
                CssFontFeatureValueKind::CharacterVariant => 2,
                _ => 1,
            };
            if value.len() > max {
                return Err(CssomError::Source(CssomException::InvalidAccess));
            }
            if map.entries.iter().any(|old| {
                old.key == key && matches!(&old.value, CssomFeatureValue::Unsigned(v) if v == value)
            }) {
                return Ok(false);
            }
            let map = this.staged.maps.get_mut(id).expect("map");
            if let Some(old) = map.entries.iter_mut().find(|v| v.key == key) {
                old.value = CssomFeatureValue::Unsigned(value.to_vec());
            } else {
                map.entries.push(CssomFeatureEntry {
                    key: key.to_owned(),
                    value: CssomFeatureValue::Unsigned(value.to_vec()),
                });
            }
            this.affect(
                CssomObjectId::FeatureMap(id.clone()),
                &[CssomChange::FeatureMaps],
            );
            Ok(true)
        })
    }
    pub fn delete_feature(
        &mut self,
        id: &CssomFeatureMapId,
        key: &str,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            let map = this.staged.feature_map(id)?;
            let Some(index) = map.entries.iter().position(|v| v.key == key) else {
                return Ok(false);
            };
            this.staged
                .maps
                .get_mut(id)
                .expect("map")
                .entries
                .remove(index);
            this.affect(
                CssomObjectId::FeatureMap(id.clone()),
                &[CssomChange::FeatureMaps],
            );
            Ok(true)
        })
    }
}

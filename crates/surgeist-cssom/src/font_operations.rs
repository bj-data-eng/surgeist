//! Fonts CSSOM views and source map operations over the shared live owner.
use crate::*;
use surgeist_css::*;

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
    pub fn font_family(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        serialize_font_face_family_list_with_limits(self.families, limits)
            .map_err(CssomError::Value)
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
    pub fn annotation(&self) -> &'a CssomFeatureMapId {
        self.map(CssFontFeatureValueKind::Annotation)
    }
    pub fn ornaments(&self) -> &'a CssomFeatureMapId {
        self.map(CssFontFeatureValueKind::Ornaments)
    }
    pub fn stylistic(&self) -> &'a CssomFeatureMapId {
        self.map(CssFontFeatureValueKind::Stylistic)
    }
    pub fn swash(&self) -> &'a CssomFeatureMapId {
        self.map(CssFontFeatureValueKind::Swash)
    }
    pub fn character_variant(&self) -> &'a CssomFeatureMapId {
        self.map(CssFontFeatureValueKind::CharacterVariant)
    }
    pub fn styleset(&self) -> &'a CssomFeatureMapId {
        self.map(CssFontFeatureValueKind::Styleset)
    }
    pub fn historical_forms(&self) -> &'a CssomFeatureMapId {
        self.map(CssFontFeatureValueKind::HistoricalForms)
    }
}

/// Readonly palette attributes retain the current checked descriptor owner.
pub struct CssomFontPaletteValuesView<'a> {
    value: &'a CssFontPaletteValuesRule,
}
impl CssomFontPaletteValuesView<'_> {
    pub fn name(&self) -> &str {
        self.value.name().as_str()
    }
    pub fn font_family(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        self.descriptor(CssFontPaletteDescriptorKind::FontFamily, limits)
    }
    pub fn base_palette(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        self.descriptor(CssFontPaletteDescriptorKind::BasePalette, limits)
    }
    pub fn override_colors(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        self.descriptor(CssFontPaletteDescriptorKind::OverrideColors, limits)
    }
    fn descriptor(
        &self,
        kind: CssFontPaletteDescriptorKind,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        self.value
            .descriptors()
            .iter()
            .rev()
            .find(|entry| entry.value().kind() == kind)
            .map(|entry| {
                entry
                    .value()
                    .to_specified_css_with_limits(limits)
                    .map_err(CssomError::Value)
            })
            .unwrap_or_else(|| Ok(String::new()))
    }
}

impl CssomSnapshot {
    pub fn font_feature_values(
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
    pub fn font_palette_values(
        &self,
        id: &CssomRuleId,
    ) -> Result<CssomFontPaletteValuesView<'_>, CssomError> {
        let CssomRuleData::Leaf { current } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        let value = match current {
            CssomAuthoredRule::Ordinary(CssRule::FontPaletteValues(value))
            | CssomAuthoredRule::Scoped(CssScopedRule::FontPaletteValues(value)) => value,
            _ => return Err(CssomError::WrongKind),
        };
        Ok(CssomFontPaletteValuesView { value })
    }
}
impl CssomFeatureMap {
    pub fn size(&self) -> usize {
        self.entries().len()
    }
    /// Presence is independent of whether an exact authored value fits u32.
    pub fn has(&self, key: &str) -> bool {
        self.entries().iter().any(|entry| entry.key == key)
    }
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &str> {
        self.entries().iter().map(|entry| entry.key.as_str())
    }
    pub fn values(&self) -> impl ExactSizeIterator<Item = Result<&[u32], CssomError>> {
        self.entries().iter().map(|entry| match &entry.value {
            CssomFeatureValue::Unsigned(values) => Ok(values.as_slice()),
            CssomFeatureValue::Authored(_) => Err(CssomError::AuthoredFeatureConversion),
        })
    }
}
impl CssomBatch {
    /// Atomic maplike clear; original authored occurrences remain independently retained.
    pub fn clear_feature_map(&mut self, id: &CssomFeatureMapId) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.feature_map(id)?;
            let map = this.staged.maps.get_mut(id).expect("owned map");
            if map.entries.is_empty() {
                return Ok(false);
            }
            map.entries.clear();
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
        self.apply(|this| {
            let CssomRuleData::FontFeatureValues { .. } = this.staged.rule(id)?.data() else {
                return Err(CssomError::WrongKind);
            };
            let (value, diagnostics) = this
                .staged
                .context
                .parser_context()
                .parse_font_feature_values_family_list_with_limits(
                    source,
                    crate::rule_operations::component_limits(&this.limits),
                )
                .into_parts();
            crate::rule_operations::check_diagnostics(&diagnostics)?;
            let Some(value) = value else {
                return Ok(false);
            };
            this.assign_font_families(id, value, diagnostics)
        })
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
        let CssomRuleData::FontFeatureValues {
            families,
            family_input,
            ..
        } = &mut self.staged.rules.get_mut(id).expect("owned rule").data
        else {
            unreachable!()
        };
        if families.as_slice() == input.value.families() {
            return Ok(false);
        }
        *families = input.value.families().to_vec();
        *family_input = Some(input);
        self.affect(
            CssomObjectId::Rule(id.clone()),
            &[CssomChange::Descriptors, CssomChange::Provenance],
        );
        Ok(true)
    }
}

/// A font-face rule's same-object declaration carrier and descriptor reflections.
/// All scalar values and complete replacements use the common declaration owner.
pub struct CssomFontFaceView<'a> {
    style_id: &'a CssomBlockId,
    style: CssomDeclarationView<'a>,
}
macro_rules! font_face_attributes {
    ($($method:ident => $kind:ident),+ $(,)?) => {$(
        pub fn $method(&self) -> Result<String, CssomError> {
            self.style.get_property_value(CssFontFaceDescriptorKind::$kind.css_name())
        }
    )+};
}
impl<'a> CssomFontFaceView<'a> {
    pub fn style_id(&self) -> &'a CssomBlockId {
        self.style_id
    }
    pub fn style(&self) -> &CssomDeclarationView<'a> {
        &self.style
    }
    font_face_attributes! {
        src => Src,
        font_family => FontFamily,
        font_style => FontStyle,
        font_weight => FontWeight,
        font_stretch => FontWidth,
        font_width => FontWidth,
        unicode_range => UnicodeRange,
        font_feature_settings => FontFeatureSettings,
        font_variation_settings => FontVariationSettings,
        font_named_instance => FontNamedInstance,
        font_display => FontDisplay,
        font_language_override => FontLanguageOverride,
        ascent_override => AscentOverride,
        descent_override => DescentOverride,
        line_gap_override => LineGapOverride,
    }
}
impl CssomSnapshot {
    pub fn font_face(
        &self,
        id: &CssomRuleId,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomFontFaceView<'_>, CssomError> {
        let CssomRuleData::FontFace { block } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        Ok(CssomFontFaceView {
            style_id: block,
            style: self.declarations(block, limits)?,
        })
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
            this.prepare_set_property(&block, kind.css_name(), value, "", limits)
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

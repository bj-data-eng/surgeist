//! Readonly rule facets over checked current authored data and live media identity.
use crate::*;
use surgeist_css::*;
use surgeist_css::{CssColorProfileRule, CssRule};

/// The authored custom-media branch, without query matching or alias evaluation.
/// A MediaList ID is stable across edits; its contents belong to each snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomCustomMediaQuery {
    Boolean(bool),
    MediaList(CssomMediaListId),
}

impl CssomSnapshot {
    /// Returns the decoded extension name from the associated current rule.
    pub(crate) fn custom_media_name(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        let CssomRuleData::CustomMedia { name, .. } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        Ok(name.as_str())
    }
    /// Returns the authored boolean or associated same-identity MediaList.
    /// Inspect its contents with this snapshot's existing MediaList getters.
    pub(crate) fn custom_media_query(
        &self,
        id: &CssomRuleId,
    ) -> Result<&CssomCustomMediaQuery, CssomError> {
        let CssomRuleData::CustomMedia { query, .. } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        Ok(query)
    }
    /// Serializes the associated current profile identity through the CSS writer.
    pub(crate) fn color_profile_name(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        let name = match self.color_profile(id)?.name() {
            CssColorProfileRuleName::Custom(name) => name.as_str(),
            CssColorProfileRuleName::DeviceCmyk => "device-cmyk",
            _ => return Err(CssomError::InvalidInput("unsupported color-profile name")),
        };
        serialize_css_identifier(name, limits).map_err(CssomError::Value)
    }
    /// Returns serialized src or empty text when the descriptor is absent.
    pub(crate) fn color_profile_src(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        self.color_profile_descriptor(id, CssColorProfileDescriptorKind::Src, limits)
    }
    /// Returns the specified descriptor, without fabricating its initial value.
    pub(crate) fn color_profile_rendering_intent(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        self.color_profile_descriptor(id, CssColorProfileDescriptorKind::RenderingIntent, limits)
    }
    /// Returns serialized channel names or empty text when absent; performs no computation.
    pub(crate) fn color_profile_components(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        self.color_profile_descriptor(id, CssColorProfileDescriptorKind::Components, limits)
    }
    fn color_profile(&self, id: &CssomRuleId) -> Result<&CssColorProfileRule, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Rule(CssRule::ColorProfile(rule)),
            } => Ok(rule),
            _ => Err(CssomError::WrongKind),
        }
    }
    fn color_profile_descriptor(
        &self,
        id: &CssomRuleId,
        kind: CssColorProfileDescriptorKind,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        self.color_profile(id)?
            .effective(kind)
            .map(|value| value.serialize_specified(limits).map_err(CssomError::Value))
            .unwrap_or_else(|| Ok(String::new()))
    }
}

//! Checked authored optimization hints from CSS Will Change 1 §2.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::specified_serialization::serialize_keyword_sequence;
use crate::{
    CssComponentValue, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// A decoded property name in the `will-change` custom-identifier branch.
/// Unknown future and custom property names are retained without resolving them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssWillChangePropertyName(String);

impl CssWillChangePropertyName {
    /// Checks decoded identifier validity and the property-specific exclusions.
    /// `span` is valid here; the grid custom-identifier exclusions do not apply.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if matches!(
            value.to_ascii_lowercase().as_str(),
            "inherit"
                | "initial"
                | "unset"
                | "revert"
                | "revert-layer"
                | "default"
                | "will-change"
                | "none"
                | "all"
                | "auto"
                | "scroll-position"
                | "contents"
        ) {
            return None;
        }
        CssComponentValue::try_ident(value.clone()).ok()?;
        Some(Self(value))
    }

    /// Borrows the decoded, case-preserved name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One authored optimization hint. Keywords are distinct from property names.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWillChangeFeature {
    ScrollPosition,
    Contents,
    Property(CssWillChangePropertyName),
}

/// A nonempty ordered comma list. Repeated hints remain authored entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssWillChangeFeatures(Vec<CssWillChangeFeature>);

impl CssWillChangeFeatures {
    /// Checks the nonempty list invariant without deduplicating or reordering.
    #[must_use]
    pub fn try_new(features: Vec<CssWillChangeFeature>) -> Option<Self> {
        (!features.is_empty()).then_some(Self(features))
    }

    /// Borrows the authored hints in order, including repeats.
    pub fn items(&self) -> &[CssWillChangeFeature] {
        &self.0
    }
}

/// The specified `auto | <animateable-feature>#` value, before optimizations.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWillChange {
    Auto,
    Features(CssWillChangeFeatures),
}

impl CssWillChange {
    /// Emits canonical specified CSS, preserving property-name case and repeats.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses one cumulative budget: one node for `auto`, or one list plus one
    /// node per hint, for both input and projection. All emitted bytes count.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let Self::Features(features) = self else {
            return serialize_keyword_sequence("auto", limits);
        };
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        for (index, feature) in features.items().iter().enumerate() {
            writer.context.charge_input(1)?;
            writer.context.charge_projection(1)?;
            if index != 0 {
                writer.append(", ")?;
            }
            match feature {
                CssWillChangeFeature::ScrollPosition => writer.append("scroll-position")?,
                CssWillChangeFeature::Contents => writer.append("contents")?,
                CssWillChangeFeature::Property(name) => writer.append_identifier(name.as_str())?,
            }
        }
        Ok(writer.css)
    }
}

//! Checked authored optimization hints from CSS Will Change 1 §2.

use crate::CssComponentValue;

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
///
/// # Specified serialization
///
/// Emits canonical specified CSS, preserving property-name case and repeats.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWillChange {
    Auto,
    Features(CssWillChangeFeatures),
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssWillChange {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let Self::Features(features) = self else {
            return writer.keyword("auto");
        };

        writer.node()?;
        for (index, feature) in features.items().iter().enumerate() {
            writer.node()?;
            if index != 0 {
                writer.append(", ")?;
            }
            match feature {
                CssWillChangeFeature::ScrollPosition => writer.append("scroll-position")?,
                CssWillChangeFeature::Contents => writer.append("contents")?,
                CssWillChangeFeature::Property(name) => writer.append_identifier(name.as_str())?,
            }
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssWillChange);

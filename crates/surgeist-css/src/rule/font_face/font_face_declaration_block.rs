//! Checked selected FontFace declarations, independent of whole-rule kind order.
use crate::rule::font_face::{CssFontFaceDescriptor, CssFontFaceDescriptorKind};
use crate::serialization::CssSerializationError;
use crate::serialization::CssSerializationLimits as Limits;
use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind;

/// Why selected membership or its specified text cannot be completed.
///
/// A serialization failure is exposed by [`Self::serialization`] and is the error's
/// source; [`Self::is_resource_limit`] groups the resource refusals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFaceDeclarationBlockError {
    kind: FontFaceDeclarationBlockKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum FontFaceDeclarationBlockKind {
    DuplicateDescriptor { kind: CssFontFaceDescriptorKind },
    Serialization(CssSerializationError),
}
impl CssFontFaceDeclarationBlockError {
    pub(crate) const fn duplicate(kind: CssFontFaceDescriptorKind) -> Self {
        Self {
            kind: FontFaceDeclarationBlockKind::DuplicateDescriptor { kind },
        }
    }
    /// Returns the descriptor kind that occurs more than once in an authored inventory,
    /// which is not selected membership.
    #[must_use]
    pub const fn duplicate_descriptor(&self) -> Option<CssFontFaceDescriptorKind> {
        match self.kind {
            FontFaceDeclarationBlockKind::DuplicateDescriptor { kind } => Some(kind),
            FontFaceDeclarationBlockKind::Serialization(_) => None,
        }
    }
    /// Returns the scalar or punctuation owner's precise resource or capability cause.
    #[must_use]
    pub const fn serialization(&self) -> Option<&CssSerializationError> {
        match &self.kind {
            FontFaceDeclarationBlockKind::Serialization(error) => Some(error),
            FontFaceDeclarationBlockKind::DuplicateDescriptor { .. } => None,
        }
    }
    /// Returns whether the serialization cause is a resource limit.
    #[must_use]
    pub fn is_resource_limit(&self) -> bool {
        self.serialization()
            .is_some_and(CssSerializationError::is_resource_limit)
    }
}
impl std::fmt::Display for CssFontFaceDeclarationBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            FontFaceDeclarationBlockKind::DuplicateDescriptor { kind } => {
                write!(f, "duplicate selected {} descriptor", kind.as_str())
            }
            FontFaceDeclarationBlockKind::Serialization(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for CssFontFaceDeclarationBlockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.serialization()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
impl From<CssSerializationError> for CssFontFaceDeclarationBlockError {
    fn from(error: CssSerializationError) -> Self {
        Self {
            kind: FontFaceDeclarationBlockKind::Serialization(error),
        }
    }
}

/// Exact ordered, unique selected FontFace descriptor occurrences.
///
/// CSSOM owns winner selection and live ordering. This checked boundary rejects
/// raw duplicates instead of silently choosing winners or adopting the whole
/// FontFace rule writer's fixed kind order. Clones retain actual descriptor-name
/// positions and component origins. Empty selected membership is valid.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSpecifiedFontFaceDeclarationBlock {
    entries: Vec<CssFontFaceDescriptor>,
}
impl CssSpecifiedFontFaceDeclarationBlock {
    /// Validates selected membership and traverses every supplied value under
    /// one cumulative input/projection budget. Construction emits no output.
    pub fn try_from_entries(
        entries: &[CssFontFaceDescriptor],
        limits: Limits,
    ) -> Result<Self, CssFontFaceDeclarationBlockError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.node()?;
        let mut admitted = Vec::new();
        for entry in entries {
            writer.node()?;
            let kind = entry.value().kind();
            if admitted
                .iter()
                .any(|entry: &CssFontFaceDescriptor| entry.value().kind() == kind)
            {
                return Err(CssFontFaceDeclarationBlockError::duplicate(kind));
            }
            writer.without_output(|writer| entry.value().append_to_rule_writer(writer))?;
            admitted.try_reserve(1).map_err(|_| {
                CssSerializationError::new(
                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            admitted.push(entry.clone());
        }
        Ok(Self { entries: admitted })
    }
    /// Returns the admitted descriptor occurrences in the supplied order; no two share a descriptor kind.
    #[must_use]
    pub fn entries(&self) -> &[CssFontFaceDescriptor] {
        &self.entries
    }

    /// Emits the exact supplied selected order as an unbraced declaration block.
    /// Every declaration uses canonical name, `: `, scalar text and `;`, separated
    /// by one space. Empty membership emits the empty string successfully.
    ///
    /// Full input, value projection and punctuation/output share one allowance.
    /// No partial text escapes; an unchanged larger-budget retry is independent.
    pub fn serialize_cssom(
        &self,
        limits: Limits,
    ) -> Result<String, CssFontFaceDeclarationBlockError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_cssom(&mut writer)?;
        Ok(writer.css)
    }
    pub(crate) fn append_cssom(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        writer.node()?;
        for (index, entry) in self.entries.iter().enumerate() {
            writer.node()?;
            if index != 0 {
                writer.append(" ")?;
            }
            writer.append(entry.value().kind().as_str())?;
            writer.append(": ")?;
            entry.value().append_to_rule_writer(writer)?;
            writer.append(";")?;
        }
        Ok(())
    }
}

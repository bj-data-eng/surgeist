//! Checked selected FontFace declarations, independent of whole-rule kind order.
use crate::{
    CssFontFaceDescriptor, CssFontFaceDescriptorKind, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits as Limits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

/// Why selected membership or its specified text cannot be completed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDeclarationBlockError {
    /// An authored inventory with duplicate kinds is not selected membership.
    DuplicateDescriptor { kind: CssFontFaceDescriptorKind },
    /// The scalar/punctuation owner retains its precise resource or capability cause.
    Serialization(CssSpecifiedValueSerializationError),
}
impl std::fmt::Display for CssFontFaceDeclarationBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateDescriptor { kind } => {
                write!(f, "duplicate selected {} descriptor", kind.css_name())
            }
            Self::Serialization(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for CssFontFaceDeclarationBlockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DuplicateDescriptor { .. } => None,
            Self::Serialization(error) => Some(error),
        }
    }
}
impl From<CssSpecifiedValueSerializationError> for CssFontFaceDeclarationBlockError {
    fn from(error: CssSpecifiedValueSerializationError) -> Self {
        Self::Serialization(error)
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
    pub fn try_from_entries(
        entries: &[CssFontFaceDescriptor],
    ) -> Result<Self, CssFontFaceDeclarationBlockError> {
        Self::try_from_entries_with_limits(entries, Limits::default())
    }
    /// Validates selected membership and traverses every supplied value under
    /// one cumulative input/projection budget. Construction emits no output.
    pub fn try_from_entries_with_limits(
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
                return Err(CssFontFaceDeclarationBlockError::DuplicateDescriptor { kind });
            }
            writer.without_output(|writer| entry.value().append_to_rule_writer(writer))?;
            admitted.try_reserve(1).map_err(|_| {
                CssSpecifiedValueSerializationError::new(
                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            admitted.push(entry.clone());
        }
        Ok(Self { entries: admitted })
    }
    #[must_use]
    pub fn entries(&self) -> &[CssFontFaceDescriptor] {
        &self.entries
    }

    /// Emits the exact supplied selected order as an unbraced declaration block.
    /// Every declaration uses canonical name, `: `, scalar text and `;`, separated
    /// by one space. Empty membership emits the empty string successfully.
    pub fn serialize_cssom(&self) -> Result<String, CssFontFaceDeclarationBlockError> {
        self.serialize_cssom_with_limits(Limits::default())
    }
    /// Full input, value projection and punctuation/output share one allowance.
    /// No partial text escapes; an unchanged larger-budget retry is independent.
    pub fn serialize_cssom_with_limits(
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
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        writer.node()?;
        for (index, entry) in self.entries.iter().enumerate() {
            writer.node()?;
            if index != 0 {
                writer.append(" ")?;
            }
            writer.append(entry.value().kind().css_name())?;
            writer.append(": ")?;
            entry.value().append_to_rule_writer(writer)?;
            writer.append(";")?;
        }
        Ok(())
    }
}

//! Canonical specified family names, retaining the authored property form.
//! Descriptor families own a decoded literal name rather than an authored form.

use crate::specified_rule_serialization::SpecifiedProvider as _;
use crate::{
    CssFontFaceFamily, CssFontFamilyList, CssFontFamilyName, CssFontFamilyNameKind,
    CssSerializationError, CssSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssFontFamilyName {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        match self.kind() {
            CssFontFamilyNameKind::Quoted => writer.append_string(self.as_str()),
            CssFontFamilyNameKind::Generic => writer.append(self.as_str()),
            CssFontFamilyNameKind::IdentSequence => {
                for (index, identifier) in self
                    .identifier_tokens()
                    .expect("identifier sequence retains tokens")
                    .iter()
                    .enumerate()
                {
                    if index != 0 {
                        writer.append(" ")?;
                    }
                    writer.node()?;
                    writer.append_identifier(identifier)?;
                }
                Ok(())
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFamilyName {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontFamilyName);

impl CssFontFamilyList {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.comma_separated(self.families(), |writer, family| {
            family.append_specified(writer)
        })?;
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFamilyList {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontFamilyList);

impl CssFontFaceFamily {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        let name = self.as_str();
        if can_serialize_family_unquoted(name) {
            for (index, word) in name.split(' ').enumerate() {
                if index != 0 {
                    writer.append(" ")?;
                }
                writer.append_identifier(word)?;
            }
            Ok(())
        } else {
            writer.append_string(name)
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFaceFamily {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontFaceFamily);

fn can_serialize_family_unquoted(name: &str) -> bool {
    if name.is_empty() || name.starts_with(' ') || name.ends_with(' ') || name.contains("  ") {
        return false;
    }
    if CssFontFamilyName::try_ident_sequence(name.split(' ').map(str::to_owned).collect()).is_none()
    {
        return false;
    }
    name.split(' ').all(|word| {
        let mut chars = word.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !(first.is_ascii_alphabetic() || first == '_' || first == '-' || first as u32 >= 0x80) {
            return false;
        }
        if first == '-' && word.len() == 1 {
            return false;
        }
        if first == '-' && chars.clone().next().is_some_and(|c| c.is_ascii_digit()) {
            return false;
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c as u32 >= 0x80)
    })
}

/// Emits actual decoded literal families in order, separated by comma-space.
/// An empty slice emits empty text; feature-value nonemptiness is checked by
/// [`crate::CssFontFeatureValuesFamilyList`], independently of this formatter.
///
/// Uses one native writer and cumulative input, projection and UTF-8 output
/// budget. Each literal costs one input and one projection node; the container
/// adds no charge. Failure returns no partial text and leaves the slice unchanged.
pub fn serialize_font_face_family_list(
    families: &[CssFontFaceFamily],
    limits: CssSerializationLimits,
) -> Result<String> {
    let mut writer = SpecifiedRuleWriter::new(limits);
    writer.comma_separated(families, |writer, family| {
        family.append_to_rule_writer(writer)
    })?;
    Ok(writer.css)
}

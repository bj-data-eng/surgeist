//! The check that the attribute names of a token are all different.
use super::*;
use crate::parser::tokenizer::{Names, token::Attribute as TokenAttribute};
use storage_holds::NAME_INDEX_MIN_ENTRIES;

/// Compares each attribute name with the earlier ones: pair by pair for a short list, through a
/// set of the names recorded so far for a long one, so that the work of checking `n` names is
/// linear in `n` and their length.
pub(super) enum DistinctNames {
    Pairs,
    Set(Names),
}

impl DistinctNames {
    pub(super) fn begin(count: usize) -> Result<Self, SessionError> {
        if count < NAME_INDEX_MIN_ENTRIES {
            return Ok(Self::Pairs);
        }
        Ok(Self::Set(Names::new()?))
    }

    /// Requires the name of `attributes[index]` to differ from every earlier name, and records
    /// it. The attributes are checked in order, so the first repeat is the one refused.
    pub(super) fn check(
        &mut self,
        control: &mut ParseControl,
        attributes: &[TokenAttribute],
        index: usize,
    ) -> Result<(), SessionError> {
        let Some(attribute) = attributes.get(index) else {
            return Err(latch_registry_row(control));
        };
        match self {
            Self::Pairs => {
                for earlier in attributes.iter().take(index) {
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if exact_string_equal(
                        control,
                        earlier.name.value.text(),
                        attribute.name.value.text(),
                    )? {
                        return Err(SessionError::DuplicateAttribute);
                    }
                }
                Ok(())
            }
            Self::Set(names) => {
                if names.insert(
                    attribute.name.value.text().code_units(),
                    attributes,
                    index,
                    control,
                )? {
                    return Err(SessionError::DuplicateAttribute);
                }
                Ok(())
            }
        }
    }

    /// Credits the storage of the recorded names.
    pub(super) fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        match self {
            Self::Pairs => Ok(()),
            Self::Set(names) => Ok(names.release(control)?),
        }
    }
}

/// Runs `check` over every attribute of `attributes` in order and releases the set whatever the
/// outcome. The first refusal is reported.
pub(super) fn require_distinct_names(
    control: &mut ParseControl,
    attributes: &[TokenAttribute],
    mut each: impl FnMut(&mut ParseControl, usize, &TokenAttribute) -> Result<(), SessionError>,
) -> Result<(), SessionError> {
    let mut distinct = DistinctNames::begin(attributes.len())?;
    let checked = (|| {
        for (index, attribute) in attributes.iter().enumerate() {
            each(control, index, attribute)?;
            distinct.check(control, attributes, index)?;
        }
        Ok(())
    })();
    let released = distinct.release(control);
    checked.and(released)
}

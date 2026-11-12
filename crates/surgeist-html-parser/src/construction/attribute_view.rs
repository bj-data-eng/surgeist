//! Reads of the attributes of a record of HTML's construction storage, whose refusals are session
//! errors.
//!
//! A build view reports an element whose run its storage does not hold, and an attribute whose
//! value its text arena does not hold, as typed refusals. A storage never makes such a record, so
//! the refusal is a defect, which the session reports as it reports any refused construction
//! operation.

use super::*;
use surgeist_html::construction::{BuildAttribute, BuildAttributeIter, BuildAttributes, BuildText};

/// The attributes of an element record, read for a session.
pub(super) trait AttributesIn<'a>: Sized {
    /// The number of attributes.
    fn count(self, control: &mut ParseControl) -> Result<usize, SessionError>;
    /// The attribute at `index`, or `None` at or past the last.
    fn at(
        self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<Option<BuildAttribute<'a>>, SessionError>;
    /// The attributes in producer order.
    fn each(self, control: &mut ParseControl) -> Result<BuildAttributeIter<'a>, SessionError>;
}
impl<'a> AttributesIn<'a> for BuildAttributes<'a> {
    fn count(self, control: &mut ParseControl) -> Result<usize, SessionError> {
        self.len()
            .map_err(|error| construction_error(control, error))
    }
    fn at(
        self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<Option<BuildAttribute<'a>>, SessionError> {
        self.get(index)
            .map_err(|error| construction_error(control, error))
    }
    fn each(self, control: &mut ParseControl) -> Result<BuildAttributeIter<'a>, SessionError> {
        self.iter()
            .map_err(|error| construction_error(control, error))
    }
}

/// The units of a value of a record, read for a session.
pub(super) trait UnitsIn<'a>: Sized {
    /// The exact code units.
    fn units(self, control: &mut ParseControl) -> Result<&'a [u16], SessionError>;
}
impl<'a> UnitsIn<'a> for BuildText<'a> {
    fn units(self, control: &mut ParseControl) -> Result<&'a [u16], SessionError> {
        self.code_units()
            .map_err(|error| construction_error(control, error))
    }
}

/// The attributes of an element record in producer order, each as its name and the exact units of
/// its value, which are borrowed from the storage and copied by no one.
pub(crate) struct ElementAttributes<'a>(BuildAttributeIter<'a>);
impl<'a> ElementAttributes<'a> {
    pub(super) fn new(rows: BuildAttributeIter<'a>) -> Self {
        Self(rows)
    }
    /// The next attribute, or `None` after the last. A row whose value its arena does not hold is
    /// the session's defect refusal.
    pub(crate) fn next(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<(&'a surgeist_html::values::QualifiedName, &'a [u16])>, SessionError> {
        match self.0.next() {
            None => Ok(None),
            Some(attribute) => {
                let units = attribute.value().units(control)?;
                Ok(Some((attribute.name(), units)))
            }
        }
    }
}

/// An owned string of the `units`, for the one caller whose consumer holds the string: the `meta`
/// values that `meta.rs` reads across later mutations of the storage that holds the attribute.
/// Rules that only read a value take its units. The copy is charged before it is made, holds no
/// more units than the attribute it reads, and is dropped by its caller.
pub(super) fn transient_string(
    control: &mut ParseControl,
    units: &[u16],
) -> Result<DocumentString, SessionError> {
    let bytes = units.len().saturating_mul(size_of::<u16>()) as u64;
    control.admit_atomic_layout(units.len().saturating_mul(size_of::<u16>()))?;
    control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
    control.charge(HtmlParseWorkCategory::CopyByte, bytes)?;
    Ok(DocumentString::from_utf16(units.to_vec()))
}

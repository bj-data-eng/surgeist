//! Open borrowed inputs become privately owned, validated element values.

use super::*;
use std::mem::size_of;
use surgeist_html::bounded::{
    HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationError,
};
use surgeist_html::{Attribute, HtmlAttributeInput, HtmlAttributesInput, HtmlNameInput};

/// Refusal before an element's values or subject are committed to the builder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCaptureElementError<E> {
    Capture(HtmlCaptureError),
    Attributes(HtmlAttributeValidationError),
    /// The open input did not supply exactly its announced attribute count.
    InconsistentAttributes {
        index: usize,
    },
    /// The original work/cancellation refusal, without translation.
    Interrupted(E),
}
impl<E> From<HtmlCaptureError> for HtmlCaptureElementError<E> {
    fn from(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
}
impl<E: fmt::Display> fmt::Display for HtmlCaptureElementError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capture(error) => error.fmt(f),
            Self::Attributes(error) => error.fmt(f),
            Self::InconsistentAttributes { index } => {
                write!(f, "HTML capture attribute input disagrees at index {index}")
            }
            Self::Interrupted(error) => write!(f, "HTML element capture interrupted: {error}"),
        }
    }
}
impl<E: Error + 'static> Error for HtmlCaptureElementError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Capture(error) => Some(error),
            Self::Attributes(error) => Some(error),
            Self::Interrupted(error) => Some(error),
            Self::InconsistentAttributes { .. } => None,
        }
    }
}

#[derive(Default)]
struct Extent {
    bytes: usize,
    units: usize,
    retained_units: usize,
}
impl Extent {
    fn string<E>(
        &mut self,
        value: &DocumentString,
        step: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(), HtmlCaptureElementError<E>> {
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        self.units = add(self.units, value.len())?;
        self.retained_units = add(self.retained_units, value.shared_code_units().capacity())?;
        let bytes = value
            .shared_code_units()
            .capacity()
            .checked_mul(size_of::<u16>())
            .ok_or(HtmlCaptureError::RepresentationExhausted)?;
        self.bytes = add(self.bytes, add(size_of::<Vec<u16>>(), bytes)?)?;
        Ok(())
    }
    fn name<E>(
        &mut self,
        name: &(impl HtmlNameInput + ?Sized),
        step: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(), HtmlCaptureElementError<E>> {
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        if let Some(uri) = name.namespace_uri() {
            // Namespace::from_uri maps a supplied empty URI to absence.
            if !uri.is_empty() {
                self.string(uri, step)?;
            }
        }
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        if let Some(prefix) = name.prefix() {
            self.string(prefix, step)?;
        }
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        self.string(name.local_name(), step)
    }
}

struct BorrowedName<'a> {
    namespace: Option<&'a DocumentString>,
    prefix: Option<&'a DocumentString>,
    local: &'a DocumentString,
}
impl HtmlNameInput for BorrowedName<'_> {
    fn namespace_uri(&self) -> Option<&DocumentString> {
        self.namespace
    }
    fn prefix(&self) -> Option<&DocumentString> {
        self.prefix
    }
    fn local_name(&self) -> &DocumentString {
        self.local
    }
}
fn borrow_name<'a, E>(
    name: &'a (impl HtmlNameInput + ?Sized),
    step: &mut impl FnMut(preflight::Action) -> Result<(), E>,
) -> Result<BorrowedName<'a>, HtmlCaptureElementError<E>> {
    step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
    let namespace = name.namespace_uri();
    step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
    let prefix = name.prefix();
    step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
    let local = name.local_name();
    Ok(BorrowedName {
        namespace,
        prefix,
        local,
    })
}
impl BorrowedName<'_> {
    fn capture<E>(
        &self,
        step: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<QualifiedName, HtmlCaptureElementError<E>> {
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        Ok(QualifiedName::new(
            self.namespace
                .cloned()
                .map_or_else(Namespace::none, Namespace::from_uri),
            self.prefix.cloned(),
            self.local.clone(),
        ))
    }
}

impl HtmlCaptureBuilder {
    pub(super) fn admit_element_value_bytes(
        &self,
        pending: usize,
    ) -> Result<usize, HtmlCaptureError> {
        let required = add(self.data.usage.element_value_bytes, pending)?;
        if required > self.limits.element_value_bytes {
            return Err(HtmlCaptureError::LimitExceeded {
                resource: HtmlCaptureResource::ElementValueBytes,
                limit: self.limits.element_value_bytes,
                required,
            });
        }
        Ok(required)
    }

    /// Forms one owned element from open borrowed operands, preserving their
    /// exact text and order. Unlike certified `OrderedAttributes`, these inputs
    /// are not a uniqueness proof: their actual destination is validated with
    /// HTML's controlled expanded-name validator before publication.
    ///
    /// `step` precedes input reads, growth and each validator comparison. Its
    /// first error is preserved. Logical counts and the cumulative
    /// `element_value_bytes` allowance admit pending storage before cloning or
    /// reserving; actual destination capacity and final logical usage are
    /// checked again before committing. The allowance includes name/attribute/
    /// saved-is string headers and capacities and the destination attribute Vec
    /// header/capacity, per occurrence. It excludes record/relationship banks,
    /// later nonce/hyperlink facts, opaque Arc headers and allocator bookkeeping.
    /// Refusal commits no subject, value receipt or logical usage.
    pub fn add_element_from_inputs<E>(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        name: &(impl HtmlNameInput + ?Sized),
        attributes: &(impl HtmlAttributesInput + ?Sized),
        is_value: HtmlCapturedIsValue,
        registry: HtmlCaptureRegistryInput,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureElementError<E>> {
        let mut admit = |action| {
            if let preflight::Action::Work(units) = action {
                for _ in 0..units {
                    step()?;
                }
            }
            Ok(())
        };
        let saved_is = match &is_value {
            HtmlCapturedIsValue::Value(value) => Some(value),
            HtmlCapturedIsValue::Absent | HtmlCapturedIsValue::Unavailable => None,
        };
        let formed = self
            .form_element_inputs_controlled(
                document,
                name,
                attributes,
                saved_is,
                registry.clone(),
                &mut admit,
            )
            .map_err(|error| match error {
                FormFailure::Input(error) => error,
                FormFailure::AllocationFailed => {
                    HtmlCaptureElementError::Capture(HtmlCaptureError::RepresentationExhausted)
                }
            })?;
        self.add_element_controlled(
            document,
            formed.name,
            formed.attributes,
            is_value,
            registry,
            &mut admit,
        )
        .map_err(|error| match error {
            preflight::Failure::Admission(error) => HtmlCaptureElementError::Interrupted(error),
            preflight::Failure::Capture(error) => HtmlCaptureElementError::Capture(error),
            preflight::Failure::AllocationFailed => {
                HtmlCaptureElementError::Capture(HtmlCaptureError::RepresentationExhausted)
            }
        })
    }

    pub(crate) fn form_element_inputs_controlled<E>(
        &self,
        document: &HtmlCaptureBuilderSubject,
        name: &(impl HtmlNameInput + ?Sized),
        attributes: &(impl HtmlAttributesInput + ?Sized),
        saved_is: Option<&DocumentString>,
        registry: HtmlCaptureRegistryInput,
        step: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<FormedElement, FormFailure<E>> {
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        self.validate_controlled_document(document)?;
        let registry_value = self.registry(registry.clone())?;
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        let count = attributes.len();
        let relationships = 1 + usize::from(matches!(registry_value, Registry::Index(_)));
        self.admit(1, relationships, count, 0, 0, 0)?;
        let row_bytes = count
            .checked_mul(size_of::<Attribute>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(HtmlCaptureError::RepresentationExhausted)?;
        let mut extent = Extent {
            bytes: add(size_of::<Vec<Attribute>>(), row_bytes)?,
            units: 0,
            retained_units: 0,
        };
        extent.name(name, step)?;
        for index in 0..count {
            step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            let attribute = attributes
                .get(index)
                .ok_or(HtmlCaptureElementError::InconsistentAttributes { index })?;
            step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            extent.name(attribute.name(), step)?;
            step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            extent.string(attribute.value(), step)?;
            self.admit_element_value_bytes(extent.bytes)?;
            self.admit(1, relationships, count, extent.units, 0, 0)?;
        }
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        if attributes.get(count).is_some() {
            return Err(HtmlCaptureElementError::InconsistentAttributes { index: count }.into());
        }
        if let Some(value) = saved_is {
            extent.string(value, step)?;
        }
        self.admit(1, relationships, count, extent.units, 0, 0)?;
        self.admit_element_value_bytes(extent.bytes)?;
        step(preflight::Action::ElementValues {
            bytes: extent.bytes,
            units: extent.retained_units,
            observed: false,
        })
        .map_err(HtmlCaptureElementError::Interrupted)?;

        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        step(preflight::Action::Reserve {
            entries: count,
            record_bytes: size_of::<Attribute>(),
        })
        .map_err(HtmlCaptureElementError::Interrupted)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| FormFailure::AllocationFailed)?;
        let actual_rows = values
            .capacity()
            .checked_mul(size_of::<Attribute>())
            .ok_or(HtmlCaptureError::RepresentationExhausted)?;
        let pending = add(
            extent.bytes,
            actual_rows
                .checked_sub(row_bytes)
                .expect("reserved announced attributes"),
        )?;
        step(preflight::Action::ElementValues {
            bytes: pending,
            units: extent.retained_units,
            observed: true,
        })
        .map_err(HtmlCaptureElementError::Interrupted)?;
        self.admit_element_value_bytes(pending)?;
        let mut actual = Extent {
            bytes: add(size_of::<Vec<Attribute>>(), actual_rows)?,
            units: 0,
            retained_units: 0,
        };
        // Freeze only borrowed component references, not an owned or validated
        // name. Admission and cloning now use these exact immutable values even
        // when the open provider chooses a different value on its next call.
        let name = borrow_name(name, step)?;
        actual.name(&name, step)?;
        self.admit_element_value_bytes(actual.bytes)?;
        self.admit(1, relationships, count, actual.units, 0, 0)?;
        step(preflight::Action::ElementValues {
            bytes: actual.bytes,
            units: actual.retained_units,
            observed: false,
        })
        .map_err(HtmlCaptureElementError::Interrupted)?;
        let name = name.capture(step)?;
        for index in 0..count {
            step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            let attribute = attributes
                .get(index)
                .ok_or(HtmlCaptureElementError::InconsistentAttributes { index })?;
            step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            let attribute_name = borrow_name(attribute.name(), step)?;
            actual.name(&attribute_name, step)?;
            step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            let attribute_value = attribute.value();
            actual.string(attribute_value, step)?;
            self.admit_element_value_bytes(actual.bytes)?;
            self.admit(1, relationships, count, actual.units, 0, 0)?;
            step(preflight::Action::ElementValues {
                bytes: actual.bytes,
                units: actual.retained_units,
                observed: false,
            })
            .map_err(HtmlCaptureElementError::Interrupted)?;
            step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            let name = attribute_name.capture(step)?;
            step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            let value = attribute_value.clone();
            values.push(Attribute::new(name, value));
        }
        if let Some(value) = saved_is {
            actual.string(value, step)?;
        }
        self.admit_element_value_bytes(actual.bytes)?;
        self.admit(1, relationships, count, actual.units, 0, 0)?;
        step(preflight::Action::ElementValues {
            bytes: actual.bytes,
            units: actual.retained_units,
            observed: false,
        })
        .map_err(HtmlCaptureElementError::Interrupted)?;
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        if attributes.len() != count || attributes.get(count).is_some() {
            return Err(HtmlCaptureElementError::InconsistentAttributes { index: count }.into());
        }
        // Arc header storage is admitted above; validation owns this exact
        // destination throughout every fallible comparison, with no second Vec.
        step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
        let mut validation =
            Some(HtmlAttributeStorage::from_shared_attributes(Arc::new(values)).validate());
        while let Some(action) = validation
            .as_mut()
            .expect("owned validation")
            .next_action()
            .map_err(HtmlCaptureElementError::Attributes)?
        {
            let layout = action.layout();
            let work = layout
                .record_visits()
                .checked_add(layout.inspected_bytes() / size_of::<u16>() as u64)
                .ok_or(HtmlCaptureError::RepresentationExhausted)?;
            for _ in 0..work {
                step(preflight::Action::Work(1)).map_err(HtmlCaptureElementError::Interrupted)?;
            }
            action
                .advance()
                .map_err(HtmlCaptureElementError::Attributes)?;
        }
        let attributes = HtmlAttributeValidation::try_finish(&mut validation)
            .map_err(HtmlCaptureElementError::Attributes)?
            .into_attributes();
        let usage = storage::option_value_element(&name, &attributes, saved_is, &mut || {
            step(preflight::Action::Work(1))
        })
        .map_err(|error| match error {
            HtmlCaptureStorageError::RepresentationExhausted => {
                HtmlCaptureElementError::Capture(HtmlCaptureError::RepresentationExhausted)
            }
            HtmlCaptureStorageError::Interrupted(error) => {
                HtmlCaptureElementError::Interrupted(error)
            }
        })?;
        Ok(FormedElement {
            name,
            attributes,
            usage,
        })
    }
}

/// Only fully validated destination storage crosses this private owner seam.
pub(crate) struct FormedElement {
    pub(crate) name: QualifiedName,
    pub(crate) attributes: OrderedAttributes,
    pub(crate) usage: HtmlCaptureRetainedStorageUsage,
}

pub(crate) enum FormFailure<E> {
    Input(HtmlCaptureElementError<E>),
    AllocationFailed,
}
impl<E> From<HtmlCaptureElementError<E>> for FormFailure<E> {
    fn from(error: HtmlCaptureElementError<E>) -> Self {
        Self::Input(error)
    }
}
impl<E> From<HtmlCaptureError> for FormFailure<E> {
    fn from(error: HtmlCaptureError) -> Self {
        Self::Input(error.into())
    }
}

#[cfg(test)]
mod tests;

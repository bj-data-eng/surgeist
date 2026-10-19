//! Canonical name binding at DOM-owned ingress. Language validation and case
//! processing finish before immutable owner-qualified values enter a node.
use super::super::*;
use crate::name::{AttributeStorage, AttributeValues};
use crate::{DomQualifiedName, Namespace, QualifiedName};
use surgeist_html::HtmlNameInput;

pub(in crate::store) struct AttributeInput {
    pub entries: Vec<crate::DomAttribute>,
    global: Option<custody::Debit>,
    local: Option<name_registry::Attribution>,
}
pub(in crate::store) struct FormedAttributes {
    pub value: AttributeStorage,
    pub custody: Option<bank::ObjectCustody>,
}
// Name conversion creates at most namespace, prefix and local buffers.
// These fixed stack slots are not an allocated directory. Each actual new
// string dies before its measured receipt; shared borrowed inputs add no target.
struct NormalizedExtent {
    local: name_registry::Attribution,
    _global: Option<custody::Debit>,
}
pub(in crate::store) struct Normalized<'a> {
    operation: &'a name_registry::Operation,
    extents: [Option<NormalizedExtent>; 3],
    len: usize,
}
impl<'a> Normalized<'a> {
    pub(in crate::store) fn new(operation: &'a name_registry::Operation) -> Self {
        Self {
            operation,
            extents: [None, None, None],
            len: 0,
        }
    }
    fn before(
        &self,
        units: usize,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<NormalizedExtent, DomError> {
        let bytes = units
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<u16>>()))
            .ok_or(DomError::RepresentationExhausted)?;
        let local = self.operation.attribute(bytes)?;
        let global = if budget.is_none() {
            let ledger = self.operation.owner.lock().registry.ledger().clone();
            Some(ledger.retain(custody::Usage::new(bytes, units), |_| check_cancel(cancel))?)
        } else {
            None
        };
        Ok(NormalizedExtent {
            local,
            _global: global,
        })
    }
    fn finish(
        &mut self,
        value: DocumentString,
        extent: NormalizedExtent,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        struct Formed {
            value: DocumentString,
            extent: NormalizedExtent,
        }
        let mut formed = Formed { value, extent };
        let capacity = formed.value.shared_code_units().capacity();
        let actual = capacity
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<u16>>()))
            .ok_or(DomError::RepresentationExhausted)?;
        let extra = actual
            .checked_sub(formed.extent.local.bytes)
            .expect("requested normalized text extent");
        formed.extent.local.grow(extra)?;
        if let Some(global) = &mut formed.extent._global {
            let units = capacity
                .checked_sub(global.usage().units())
                .expect("normalized unit capacity");
            global.grow(custody::Usage::new(extra, units), |_| check_cancel(cancel))?;
        }
        // Source Formation already owns its exact new buffer globally.
        assert!(
            self.len < self.extents.len(),
            "one QName has at most three new text fields"
        );
        self.extents[self.len] = Some(formed.extent);
        self.len += 1;
        Ok(formed.value)
    }
    pub(in crate::store) fn units(
        &mut self,
        units: &[u16],
        lower: bool,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        self.copy(units.len(), units.iter().copied(), lower, budget, cancel)
    }
    fn string(
        &mut self,
        text: &str,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        self.copy(
            text.encode_utf16().count(),
            text.encode_utf16(),
            false,
            budget,
            cancel,
        )
    }
    fn copy(
        &mut self,
        length: usize,
        input: impl Iterator<Item = u16>,
        lower: bool,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        let extent = self.before(length, budget, cancel)?;
        if let Some(budget) = budget {
            budget.retain_payload(
                source::storage::Payload {
                    bytes: size_of::<Vec<u16>>(),
                    units: length,
                },
                cancel,
            )?;
        }
        let mut output = Vec::new();
        source::formation::reserve(&mut output, length, budget, cancel)?;
        if let Some(budget) = budget {
            budget.retain_payload(
                source::storage::Payload {
                    bytes: 0,
                    units: output
                        .capacity()
                        .checked_sub(length)
                        .expect("requested conversion extent"),
                },
                cancel,
            )?;
        }
        #[cfg(test)]
        let mut copied = CopyObservation::default();
        for unit in input {
            self.operation
                .step(&mut || source::formation::step(budget, cancel))?;
            output.push(if lower {
                crate::names::ascii_lower(unit)
            } else {
                unit
            });
            #[cfg(test)]
            copied.pushed::<u16>();
        }
        let value = DocumentString::from_utf16(output);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: size_of::<Vec<u16>>(),
            });
        }
        self.finish(value, extent, cancel)
    }
}
impl Draft {
    pub(in crate::store) fn name_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.names.step(&mut || self.source_step(cancel))
    }
    pub(in crate::store) fn bind_name(
        &self,
        raw: &(impl HtmlNameInput + ?Sized),
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomQualifiedName, DomError> {
        let mut admission = name_registry::Admission {
            global: &mut |next| self.admit_custody_usage(next, cancel),
            step: &mut || self.source_step(cancel),
        };
        let namespace = raw
            .namespace_uri()
            .filter(|text| !text.is_empty())
            .map(|text| self.names.retain(text.clone(), &mut admission))
            .transpose()?;
        let local = self
            .names
            .retain(raw.local_name().clone(), &mut admission)?;
        if let Some(budget) = budget {
            // The selected atom may retain a different, larger capacity than
            // the incoming equal text. Admit before constructing the occurrence.
            let mut payload = source::storage::Payload::default();
            for text in namespace
                .as_ref()
                .map(crate::DomNameAtom::text)
                .into_iter()
                .chain(raw.prefix())
                .chain([local.text()])
            {
                payload.string(text)?;
            }
            budget.retain_payload(payload, cancel)?;
        }
        DomQualifiedName::from_atoms(namespace, raw.prefix().cloned(), local)
    }

    pub(in crate::store) fn bind_canonical_name(
        &self,
        raw: &DomQualifiedName,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomQualifiedName, DomError> {
        let mut admission = name_registry::Admission {
            global: &mut |next| self.admit_custody_usage(next, cancel),
            step: &mut || self.source_step(cancel),
        };
        let namespace = raw
            .namespace()
            .map(|atom| self.names.retain_atom(atom, &mut admission))
            .transpose()?;
        let local = self
            .names
            .retain_atom(raw.local_name_atom(), &mut admission)?;
        if let Some(budget) = budget {
            let mut payload = source::storage::Payload::default();
            for text in namespace
                .as_ref()
                .map(crate::DomNameAtom::text)
                .into_iter()
                .chain(raw.prefix())
                .chain([local.text()])
            {
                payload.string(text)?;
            }
            budget.retain_payload(payload, cancel)?;
        }
        DomQualifiedName::from_atoms(namespace, raw.prefix().cloned(), local)
    }

    pub(in crate::store) fn attribute_input<E: From<DomError>>(
        &self,
        capacity: usize,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
        allocate: impl FnOnce(&mut Vec<crate::DomAttribute>) -> Result<(), E>,
    ) -> Result<AttributeInput, E> {
        let body = if capacity == 0 {
            0
        } else {
            size_of::<payload::Payload<AttributeValues>>()
        };
        let bytes = capacity
            .checked_mul(size_of::<crate::DomAttribute>())
            .and_then(|bytes| bytes.checked_add(body))
            .ok_or(DomError::RepresentationExhausted)?;
        let receipt = if capacity == 0 {
            0
        } else {
            custody::SharedDebit::administration_usage().bytes()
        };
        let local = if bytes == 0 {
            None
        } else {
            Some(
                self.names.attribute(
                    bytes
                        .checked_add(receipt)
                        .ok_or(DomError::RepresentationExhausted)?,
                )?,
            )
        };
        let global = if let Some(budget) = budget {
            budget.retain_bytes(body, cancel)?;
            None
        } else {
            Some(
                self.custody_ledger()
                    .retain(custody::Usage::new(bytes, 0), |next| {
                        self.admit_custody_usage(next, cancel)
                    })?,
            )
        };
        let mut input = AttributeInput {
            entries: Vec::new(),
            global,
            local,
        };
        allocate(&mut input.entries)?;
        let actual = if input.entries.capacity() == 0 {
            0
        } else {
            input
                .entries
                .capacity()
                .checked_mul(size_of::<crate::DomAttribute>())
                .and_then(|bytes| bytes.checked_add(size_of::<payload::Payload<AttributeValues>>()))
                .ok_or(DomError::RepresentationExhausted)?
        };
        let extra = actual
            .checked_sub(bytes)
            .expect("requested attribute capacity");
        if let Some(local) = &mut input.local {
            local.grow(extra)?;
        }
        if let Some(global) = &mut input.global {
            global.grow(custody::Usage::new(extra, 0), |next| {
                self.admit_custody_usage(next, cancel)
            })?;
        }
        Ok(input)
    }

    pub(in crate::store) fn form_attributes(
        &self,
        input: AttributeInput,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<FormedAttributes, DomError> {
        if input.entries.capacity() == 0 {
            return Ok(FormedAttributes {
                value: AttributeStorage::default(),
                custody: None,
            });
        }
        let reserved = payload::Reserved::new()?;
        let body = size_of::<payload::Payload<AttributeValues>>();
        let bytes = input
            .entries
            .capacity()
            .checked_mul(size_of::<crate::DomAttribute>())
            .and_then(|bytes| bytes.checked_add(body))
            .ok_or(DomError::RepresentationExhausted)?;
        let AttributeInput {
            entries,
            global,
            local,
        } = input;
        let debit = match budget {
            Some(budget) => budget
                .split_payload(source::storage::Payload { bytes, units: 0 })
                .into_debit()
                .expect("present attribute target has a receipt"),
            None => global.expect("ordinary attribute input owns original credit"),
        };
        let values = AttributeValues::new(entries, || self.source_step(cancel))?;
        let shallow = values.shallow_usage()?;
        struct Forming {
            values: custody::Held<AttributeValues>,
            local: Option<name_registry::Attribution>,
        }
        let mut forming = Forming {
            values: custody::Held::new(values, debit),
            local,
        };
        let mut dependency = payload::attribute_dependency(
            forming.values.value(),
            self.custody_registry(),
            Some(&self.names),
            true,
            |next| self.admit_custody_usage(next, cancel),
        )?;
        payload::attach_attribute_extent(
            dependency
                .as_mut()
                .expect("present attribute bank has children custody"),
            forming
                .local
                .take()
                .expect("present bank owns local extent"),
        );
        let sealed = reserved
            .materialize(forming.values, shallow, |next| {
                self.admit_custody_usage(next, cancel)
            })?
            .seal_with_children(self.custody_registry(), dependency, |next| {
                self.admit_custody_usage(next, cancel)
            })?;
        Ok(FormedAttributes {
            value: AttributeStorage::from_payload(sealed.value),
            custody: Some(sealed.custody),
        })
    }

    pub(in crate::store) fn native_element(
        &self,
        local: &DocumentString,
        document: &DomDocumentMetadata,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomElement, DomError> {
        if !crate::names::valid_element_local_name(local, &mut || self.name_step(cancel))? {
            return Err(DomError::InvalidCharacter);
        }
        let input = source::formation::Formation::new(self, cancel)?;
        let mut normalized = Normalized::new(&self.names);
        let html = document.class() == crate::DomDocumentClass::Html;
        let local = if html {
            crate::names::ascii_lowercase(
                local,
                &mut || self.name_step(cancel),
                &mut |units, lower| normalized.units(units, lower, input.as_ref(), cancel),
                &mut |text| source::formation::share(text, input.as_ref(), cancel),
            )?
        } else {
            source::formation::share(local, input.as_ref(), cancel)?
        };
        let namespace = if html
            || document
                .content_type()
                .code_units()
                .iter()
                .copied()
                .eq("application/xhtml+xml".encode_utf16())
        {
            Namespace::from_uri(normalized.string(
                "http://www.w3.org/1999/xhtml",
                input.as_ref(),
                cancel,
            )?)
        } else {
            Namespace::none()
        };
        let raw = QualifiedName::new(namespace, None, local);
        let name = self.bind_name(&raw, budget, cancel)?;
        drop(raw);
        drop(normalized);
        drop(input);
        DomElement::native(name, document.document_environment(), cancel, budget)
    }

    pub(in crate::store) fn native_element_ns(
        &self,
        namespace: &Namespace,
        qualified: &DocumentString,
        environment: crate::DomDocumentEnvironment,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomElement, DomError> {
        let input = source::formation::Formation::new(self, cancel)?;
        let mut normalized = Normalized::new(&self.names);
        let raw = crate::names::element_name(
            namespace,
            qualified,
            &mut || self.name_step(cancel),
            &mut |units, lower| normalized.units(units, lower, input.as_ref(), cancel),
            &mut |text| source::formation::share(text, input.as_ref(), cancel),
        )?;
        let name = self.bind_name(&raw, budget, cancel)?;
        drop(raw);
        drop(normalized);
        drop(input);
        DomElement::native(name, environment, cancel, budget)
    }

    pub(in crate::store) fn native_element_components(
        &self,
        raw: &(impl HtmlNameInput + ?Sized),
        environment: crate::DomDocumentEnvironment,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomElement, DomError> {
        crate::names::validate_element_components(raw, &mut || self.name_step(cancel))?;
        DomElement::native(
            self.bind_name(raw, budget, cancel)?,
            environment,
            cancel,
            budget,
        )
    }
}

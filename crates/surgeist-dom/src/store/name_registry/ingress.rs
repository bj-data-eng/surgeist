//! Canonical name binding at DOM-owned ingress. Language validation and case
//! processing finish before immutable owner-qualified values enter a node.
use super::super::*;
use crate::DomQualifiedName;
use surgeist_html::values::HtmlNameInput;
use surgeist_html::values::{Namespace, QualifiedName};

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
pub(in crate::store) struct NormalizedCustody {
    _extents: [Option<NormalizedExtent>; 3],
}
impl<'a> Normalized<'a> {
    pub(in crate::store) fn new(operation: &'a name_registry::Operation) -> Self {
        Self {
            operation,
            extents: [None, None, None],
            len: 0,
        }
    }
    /// Keep the actual normalized text extent after the name operation borrow ends.
    pub(in crate::store) fn into_custody(self) -> NormalizedCustody {
        NormalizedCustody {
            _extents: self.extents,
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
        let capacity = formed.value.retained_units();
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
    /// Form this Element's name occurrence. The shared atoms remain their
    /// original children; this occurrence retains its separate text receipt.
    pub(in crate::store) fn form_element_name(
        &self,
        name: DomQualifiedName,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<payload::Sealed<crate::element::ElementName>, DomError> {
        let reserved = payload::Reserved::new()?;
        let original = name.retained_payload()?;
        let debit = match budget {
            Some(budget) => budget.split_payload(original).into_debit(),
            None => None,
        }
        .map_or_else(
            || {
                self.custody_ledger().retain(
                    custody::Usage::new(original.bytes, original.units),
                    |next| self.admit_custody_usage(next, cancel),
                )
            },
            Ok,
        )?;
        let incoming = custody::Held::new(name, debit);
        let original = incoming.value().retained_payload()?;
        let (name, debit) = incoming.into_parts();
        let incoming = custody::Held::new(crate::element::ElementName { value: name }, debit);
        let usage = custody::Usage::new(
            size_of::<payload::Payload<crate::element::ElementName>>(),
            0,
        )
        .checked_add(custody::Usage::new(original.bytes, original.units))?;
        // This reserves the dedicated name target and child-capsule local
        // extent before materialization, and owns the actual atom controls.
        let children = payload::element_name_dependency(
            incoming.value(),
            self.custody_registry(),
            Some(&self.names),
            |next| self.admit_custody_usage(next, cancel),
        )?;
        reserved
            .materialize(incoming, usage, |next| {
                self.admit_custody_usage(next, cancel)
            })?
            .seal_with_children(self.custody_registry(), children, |next| {
                self.admit_custody_usage(next, cancel)
            })
    }

    /// Admit one name operation step under both name and Source work policy.
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

    pub(in crate::store) fn native_element(
        &self,
        name: &crate::DomElementCreationName,
        options: &crate::DomElementCreationOptions,
        document: &DomDocumentMetadata,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<crate::element::FormedElement, DomError> {
        let name = self.native_element_name(name, options, document, budget, cancel)?;
        self.form_native_element(
            name,
            options.is.as_ref(),
            document.document_environment(),
            budget,
            cancel,
        )
    }

    pub(in crate::store) fn native_element_name(
        &self,
        name: &crate::DomElementCreationName,
        options: &crate::DomElementCreationOptions,
        document: &DomDocumentMetadata,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomQualifiedName, DomError> {
        let name = match name {
            crate::DomElementCreationName::Local(name) => {
                self.native_local_name(name, document, budget, cancel)?
            }
            crate::DomElementCreationName::Qualified(namespace, name) => {
                self.native_qualified_name(namespace, name, budget, cancel)?
            }
        };
        if options.registry.is_some() && options.is.is_some() {
            return Err(DomError::NotSupported);
        }
        if let Some(crate::DomRegistryBinding::Registry(id)) = options.registry {
            self.source_step(cancel)?;
            let record = self
                .registries
                .as_ref()
                .ok_or(DomError::HostedSourceUnavailable)?
                .get_with(id, &mut || self.source_step(cancel))?;
            let scope = record
                .capture
                .registry_scope(&record.local)
                .map_err(DomError::SelectCapture)?;
            use surgeist_html_semantics::HtmlRegistryScope;
            match scope {
                HtmlRegistryScope::Global
                    if document.custom_element_registry()
                        != crate::DomRegistryBinding::Registry(id) =>
                {
                    return Err(DomError::NotSupported);
                }
                HtmlRegistryScope::Unavailable => return Err(DomError::HostedSourceUnavailable),
                HtmlRegistryScope::Global | HtmlRegistryScope::Scoped => (),
            }
        }
        Ok(name)
    }

    fn native_local_name(
        &self,
        local: &DocumentString,
        document: &DomDocumentMetadata,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomQualifiedName, DomError> {
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
        Ok(name)
    }

    fn native_qualified_name(
        &self,
        namespace: &Namespace,
        qualified: &DocumentString,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomQualifiedName, DomError> {
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
        Ok(name)
    }

    pub(in crate::store) fn native_element_components(
        &self,
        raw: &(impl HtmlNameInput + ?Sized),
        environment: crate::DomDocumentEnvironment,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<crate::element::FormedElement, DomError> {
        crate::names::validate_element_components(raw, &mut || self.name_step(cancel))?;
        self.form_native_element(
            self.bind_name(raw, budget, cancel)?,
            None,
            environment,
            budget,
            cancel,
        )
    }

    pub(in crate::store) fn form_native_element(
        &self,
        name: DomQualifiedName,
        saved_is: Option<&DocumentString>,
        environment: crate::DomDocumentEnvironment,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<crate::element::FormedElement, DomError> {
        let payload::Sealed {
            value: name,
            custody,
        } = self.form_element_name(name, budget, cancel)?;
        let mut admit = |next| self.admit_custody_usage(next, cancel);
        let formation = crate::element::ElementFormation::new(
            self.custody_registry(),
            &self.blocks,
            budget,
            cancel,
            &mut admit,
        );
        let mut formed = DomElement::native(name, saved_is, environment, formation)?;
        formed.children.name = Some(custody);
        Ok(formed)
    }
}

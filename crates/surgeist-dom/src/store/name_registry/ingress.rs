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
    selected: Option<custody::Debit>,
}
pub(in crate::store) struct FormedAttributes {
    pub value: AttributeStorage,
    pub custody: Option<bank::ObjectCustody>,
    // Ordinary candidate occurrences remain guarded until the receiving Slot
    // has adopted them. Source occurrences move to their original Formation.
    pub selected: Option<custody::Debit>,
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
        let incoming = self.share_held_name_prefix(custody::Held::new(name, debit), cancel)?;
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

    /// Independent Elements retain independent attribute occurrences even
    /// when their immutable bank is shared. Unrelated edits alias this child.
    pub(in crate::store) fn form_attribute_binding(
        &self,
        attributes: FormedAttributes,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Option<payload::Sealed<crate::element::AttributeBinding>>, DomError> {
        if attributes.value.payload().is_none() {
            debug_assert!(attributes.custody.is_none() && attributes.selected.is_none());
            return Ok(None);
        }
        let reserved = payload::Reserved::new()?;
        let FormedAttributes {
            value,
            custody: _bank_custody,
            selected,
        } = attributes;
        let text = value.text_payload();
        let debit = match budget {
            Some(budget) => {
                assert!(
                    selected.is_none(),
                    "Source text credit belongs to its Formation"
                );
                budget.split_payload(text).into_debit()
            }
            None => selected,
        }
        .map_or_else(
            || {
                self.custody_ledger().retain(custody::Usage::ZERO, |next| {
                    self.admit_custody_usage(next, cancel)
                })
            },
            Ok,
        )?;
        let incoming =
            custody::Held::new(crate::element::AttributeBinding { storage: value }, debit);
        let usage = custody::Usage::new(
            size_of::<payload::Payload<crate::element::AttributeBinding>>(),
            0,
        )
        .checked_add(custody::Usage::new(text.bytes, text.units))?;
        let children = payload::attribute_binding_dependency(
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
            .map(Some)
    }

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
            selected: None,
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

    fn selected_attribute_credit(
        &self,
        input: &mut AttributeInput,
        original: source::storage::Payload,
        selected: custody::Debit,
        budget: Option<&source::formation::Formation>,
    ) {
        if let Some(budget) = budget {
            // The old field was destroyed before this exact occurrence ends.
            budget.release_payload(original);
            budget.absorb_payload_debit(selected);
        } else if let Some(credit) = &mut input.selected {
            credit
                .absorb(selected)
                .expect("one original DOM custody domain");
        } else {
            input.selected = Some(selected);
        }
    }

    fn share_attribute_strings(
        &self,
        input: &mut AttributeInput,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(session) = self.sharing_session() else {
            return Ok(());
        };
        for index in 0..input.entries.len() {
            self.source_step(cancel)?;
            self.seed_name_atoms(input.entries[index].name(), cancel)?;
            if let Some(prefix) = input.entries[index].name().prefix() {
                let selected = self.with_sharing_admission(cancel, |admission| {
                    session.share_string(prefix, admission)
                })?;
                if let Some(selected) = selected
                    && !Arc::ptr_eq(
                        prefix.shared_code_units(),
                        selected.value().shared_code_units(),
                    )
                {
                    let mut original = source::storage::Payload::default();
                    original.string(prefix)?;
                    let (value, credit) = selected.into_held().into_parts();
                    drop(input.entries[index].replace_prefix(Some(value)));
                    self.selected_attribute_credit(input, original, credit, budget);
                }
            }
            let value = input.entries[index].value();
            let selected = self.with_sharing_admission(cancel, |admission| {
                session.share_string(value, admission)
            })?;
            if let Some(selected) = selected
                && !Arc::ptr_eq(
                    value.shared_code_units(),
                    selected.value().shared_code_units(),
                )
            {
                let mut original = source::storage::Payload::default();
                original.string(value)?;
                let (value, credit) = selected.into_held().into_parts();
                drop(input.entries[index].replace_value(value));
                self.selected_attribute_credit(input, original, credit, budget);
            }
        }
        Ok(())
    }

    fn reuse_attributes(
        &self,
        input: AttributeInput,
        selected: value_sharing::SelectedBank,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<FormedAttributes, DomError> {
        let custody = payload::adopt_attributes(
            selected.storage(),
            self.custody_registry(),
            Some(&self.names),
            |next| self.admit_custody_usage(next, cancel),
        )?;
        let mut original = source::storage::Payload::default();
        original.add_bytes(
            input
                .entries
                .capacity()
                .checked_mul(size_of::<crate::DomAttribute>())
                .and_then(|bytes| bytes.checked_add(size_of::<payload::Payload<AttributeValues>>()))
                .ok_or(DomError::RepresentationExhausted)?,
        )?;
        for entry in &input.entries {
            self.source_step(cancel)?;
            original.append(entry.retained_payload()?)?;
        }
        let text = selected.storage().text_payload();
        let mut selected = selected.into_held();
        let text_credit = selected
            .debit_mut()
            .split(custody::Usage::new(text.bytes, text.units))?;
        let (value, selected_credit) = selected.into_parts();
        // The selected target and mandatory name graph now have actual custody.
        // Its temporary strong revival receipt can end after that adoption.
        drop(selected_credit);
        // Destroy the unused canonical input array and its fields before their
        // original array and string occurrence credits are released.
        drop(input);
        let selected = if let Some(budget) = budget {
            budget.release_payload(original);
            budget.absorb_payload_debit(text_credit);
            None
        } else {
            Some(text_credit)
        };
        Ok(FormedAttributes {
            value,
            custody,
            selected,
        })
    }

    pub(in crate::store) fn form_attributes(
        &self,
        mut input: AttributeInput,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<FormedAttributes, DomError> {
        if input.entries.capacity() == 0 {
            return Ok(FormedAttributes {
                value: AttributeStorage::default(),
                custody: None,
                selected: None,
            });
        }
        // Names are already owner-resolved. Whole-bank reuse precedes any
        // new materialization identity, target or optional per-string work.
        let miss = if let Some(session) = self.sharing_session() {
            match self.with_sharing_admission(cancel, |admission| {
                session.probe_bank(&input.entries, admission)
            })? {
                value_sharing::BankProbe::Hit(selected) => {
                    return self.reuse_attributes(input, selected, budget, cancel);
                }
                value_sharing::BankProbe::Miss(miss) => Some(miss),
                value_sharing::BankProbe::Bypassed => None,
            }
        } else {
            None
        };
        self.share_attribute_strings(&mut input, budget, cancel)?;
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
            selected,
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
        let formed = FormedAttributes {
            value: AttributeStorage::from_payload(sealed.value),
            custody: Some(sealed.custody),
            selected,
        };
        if let Some(session) = self.sharing_session() {
            self.with_sharing_admission(cancel, |admission| {
                session.offer_bank(&formed.value, miss, admission)
            })?;
        }
        Ok(formed)
    }

    pub(in crate::store) fn native_element(
        &self,
        local: &DocumentString,
        document: &DomDocumentMetadata,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<crate::element::FormedElement, DomError> {
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
        self.form_native_element(name, document.document_environment(), budget, cancel)
    }

    pub(in crate::store) fn native_element_ns(
        &self,
        namespace: &Namespace,
        qualified: &DocumentString,
        environment: crate::DomDocumentEnvironment,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<crate::element::FormedElement, DomError> {
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
        self.form_native_element(name, environment, budget, cancel)
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
            environment,
            budget,
            cancel,
        )
    }

    fn form_native_element(
        &self,
        name: DomQualifiedName,
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
            budget,
            cancel,
            &mut admit,
        );
        let mut formed = DomElement::native(name, environment, formation)?;
        formed.children.name = Some(custody);
        Ok(formed)
    }
}

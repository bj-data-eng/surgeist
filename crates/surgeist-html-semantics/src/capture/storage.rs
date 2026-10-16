//! Actual immutable capture payload, observed without allocation or caching.
use super::*;
use std::mem::size_of;

/// Actual capture-owned payload, independent of logical formation usage.
///
/// Includes the private data header, actual bank capacities and nested payloads.
/// Shared strings, attributes and URL records count conservatively per retained
/// occurrence. This is not a unique physical-allocation total. A consumer can
/// count a whole aliased capture once using its existing `HtmlCaptureId`.
/// Excludes the public `HtmlCapture` handle, opaque Arc control blocks and
/// allocator bookkeeping. Separately supplied captured Base facts are not owned
/// by a capture and are not included.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCaptureRetainedStorageUsage {
    bytes: usize,
    units: usize,
}
impl HtmlCaptureRetainedStorageUsage {
    /// Capture-owned headers and actual retained backing capacities, in bytes.
    pub fn retained_bytes(self) -> usize {
        self.bytes
    }
    /// Sum of actual retained UTF-16 capacities, including spare capacity.
    pub fn utf16_extent(self) -> usize {
        self.units
    }
    fn add_bytes<E>(&mut self, bytes: usize) -> Result<(), HtmlCaptureStorageError<E>> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(HtmlCaptureStorageError::RepresentationExhausted)?;
        Ok(())
    }
    fn add_units<E>(&mut self, units: usize) -> Result<(), HtmlCaptureStorageError<E>> {
        self.units = self
            .units
            .checked_add(units)
            .ok_or(HtmlCaptureStorageError::RepresentationExhausted)?;
        Ok(())
    }
    fn bank<T, E>(&mut self, capacity: usize) -> Result<(), HtmlCaptureStorageError<E>> {
        self.add_bytes(
            capacity
                .checked_mul(size_of::<T>())
                .ok_or(HtmlCaptureStorageError::RepresentationExhausted)?,
        )
    }
    fn string<E>(
        &mut self,
        value: &DocumentString,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureStorageError<E>> {
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        let capacity = value.shared_code_units().capacity();
        self.add_units(capacity)?;
        self.add_bytes(size_of::<Vec<u16>>())?;
        self.bank::<u16, E>(capacity)
    }
    fn name<E>(
        &mut self,
        name: &QualifiedName,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureStorageError<E>> {
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        if let Some(namespace) = name.namespace().uri() {
            self.string(namespace, step)?;
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        if let Some(prefix) = name.prefix() {
            self.string(prefix, step)?;
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        self.string(name.local_name(), step)
    }
    fn element<E>(
        &mut self,
        element: &HtmlCapturedElement,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureStorageError<E>> {
        let HtmlCapturedElement {
            name,
            attributes,
            is_value,
            registry: _,
            template: _,
            shadow: _,
            custom_context,
            nonce,
            link_history: _,
            hyperlink_url,
        } = element;
        self.name(name, step)?;
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        let attributes = attributes.shared_attributes();
        self.add_bytes(size_of::<Vec<surgeist_html::Attribute>>())?;
        self.bank::<surgeist_html::Attribute, E>(attributes.capacity())?;
        for attribute in attributes.iter() {
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            self.name(attribute.name(), step)?;
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            self.string(attribute.value(), step)?;
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        match is_value {
            HtmlCapturedIsValue::Value(value) => self.string(value, step)?,
            HtmlCapturedIsValue::Absent | HtmlCapturedIsValue::Unavailable => (),
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        match nonce {
            HtmlCapturedNonce::Value(value) => self.string(value, step)?,
            HtmlCapturedNonce::Unavailable => (),
        }
        // These private fields contain only inline state and opaque identities.
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        if let Some(CustomElementContext {
            state: _,
            definition,
        }) = custom_context
        {
            match definition {
                HtmlCapturedCustomElementDefinition::Definition(_)
                | HtmlCapturedCustomElementDefinition::Null
                | HtmlCapturedCustomElementDefinition::Unavailable => (),
            }
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        match hyperlink_url {
            Some(crate::HtmlCapturedHyperlinkUrl::Observed {
                document: _,
                snapshot,
            }) => {
                step().map_err(HtmlCaptureStorageError::Interrupted)?;
                let nested = snapshot.retained_storage_usage();
                // The Record bank already contains this inline snapshot header.
                self.add_bytes(
                    nested
                        .retained_bytes()
                        .checked_sub(size_of::<crate::HtmlHyperlinkUrlSnapshot>())
                        .ok_or(HtmlCaptureStorageError::RepresentationExhausted)?,
                )?;
                self.add_units(nested.utf16_extent())?;
            }
            Some(crate::HtmlCapturedHyperlinkUrl::Null) | None => (),
        }
        Ok(())
    }
    fn kind<E>(
        &mut self,
        kind: &HtmlCapturedNodeKind,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureStorageError<E>> {
        match kind {
            HtmlCapturedNodeKind::Element(element) => self.element(element, step)?,
            HtmlCapturedNodeKind::Text(value) | HtmlCapturedNodeKind::Comment(value) => {
                self.string(value, step)?;
            }
            HtmlCapturedNodeKind::ProcessingInstruction(HtmlCapturedProcessingInstruction {
                target,
                data,
            }) => {
                self.string(target, step)?;
                self.string(data, step)?;
            }
            HtmlCapturedNodeKind::Doctype(doctype) => {
                step().map_err(HtmlCaptureStorageError::Interrupted)?;
                self.string(doctype.name(), step)?;
                step().map_err(HtmlCaptureStorageError::Interrupted)?;
                if let Some(public) = doctype.public_id() {
                    self.string(public, step)?;
                }
                step().map_err(HtmlCaptureStorageError::Interrupted)?;
                if let Some(system) = doctype.system_id() {
                    self.string(system, step)?;
                }
            }
            HtmlCapturedNodeKind::Document(HtmlCapturedDocument {
                mode: _,
                scripting: _,
                image_support: _,
                parser_creation_image_support: _,
                registry: _,
                class: _,
                template_role: _,
                template_cache: _,
                template_facts_established: _,
                nonce_environment: _,
                nonce_environment_established: _,
            }) => (),
            HtmlCapturedNodeKind::Fragment(HtmlCapturedFragment {
                kind,
                host: _,
                registry: _,
            }) => match kind {
                HtmlCapturedFragmentKind::Plain => (),
                HtmlCapturedFragmentKind::ShadowRoot(HtmlCapturedShadowProperties {
                    properties: _,
                    declarative: _,
                    available_to_internals: _,
                    keep_registry_null: _,
                }) => (),
            },
        }
        Ok(())
    }
}

pub(crate) fn option_value_string<E>(
    value: &DocumentString,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<HtmlCaptureRetainedStorageUsage, HtmlCaptureStorageError<E>> {
    let mut usage = HtmlCaptureRetainedStorageUsage { bytes: 0, units: 0 };
    usage.string(value, step)?;
    Ok(usage)
}

pub(crate) fn option_value_element<E>(
    name: &QualifiedName,
    attributes: &OrderedAttributes,
    saved_is: Option<&DocumentString>,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<HtmlCaptureRetainedStorageUsage, HtmlCaptureStorageError<E>> {
    let mut usage = HtmlCaptureRetainedStorageUsage { bytes: 0, units: 0 };
    usage.name(name, step)?;
    step().map_err(HtmlCaptureStorageError::Interrupted)?;
    let values = attributes.shared_attributes();
    usage.add_bytes(size_of::<Vec<surgeist_html::Attribute>>())?;
    usage.bank::<surgeist_html::Attribute, E>(values.capacity())?;
    for attribute in values.iter() {
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        usage.name(attribute.name(), step)?;
        usage.string(attribute.value(), step)?;
    }
    if let Some(value) = saved_is {
        usage.string(value, step)?;
    }
    Ok(usage)
}

/// Checked arithmetic failure or the caller's unchanged first interruption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCaptureStorageError<E> {
    RepresentationExhausted,
    Interrupted(E),
}
impl<E: fmt::Display> fmt::Display for HtmlCaptureStorageError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepresentationExhausted => f.write_str("HTML capture storage extent exhausted"),
            Self::Interrupted(error) => {
                write!(f, "HTML capture storage observation interrupted: {error}")
            }
        }
    }
}
impl<E: Error + 'static> Error for HtmlCaptureStorageError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Interrupted(error) => Some(error),
            Self::RepresentationExhausted => None,
        }
    }
}

impl HtmlCapture {
    /// Observes this complete immutable capture's retained payload without
    /// allocating, copying, caching or changing logical `usage()`/identity.
    ///
    /// `step` is called before the data header, each bank header, each initialized
    /// row/attribute/definition and each nested provider/header observation.
    /// Capacity observations are constant work: UTF-16 contents and fixed-index
    /// relationship entries need no scan. The first callback failure is returned
    /// unchanged; all bank multiplication and total arithmetic is checked.
    /// The caller supplies its cumulative work/cancellation admission and must
    /// admit the returned extent before retaining or publishing its own result.
    /// See `HtmlCaptureRetainedStorageUsage` for the explicit alias domain.
    pub fn retained_storage_usage_with<E>(
        &self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlCaptureRetainedStorageUsage, HtmlCaptureStorageError<E>> {
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        let CaptureData {
            identity: _,
            host_revision: _,
            records,
            registries,
            usage: _,
        } = &*self.0;
        let mut usage = HtmlCaptureRetainedStorageUsage {
            bytes: size_of::<CaptureData>(),
            units: 0,
        };
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        usage.bank::<Record, E>(records.capacity())?;
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        usage.bank::<RegistryRecord, E>(registries.capacity())?;
        for record in records {
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            let Record {
                kind,
                document: _,
                parent: _,
                children,
                dependents,
                depth: _,
            } = record;
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            if let Some(children) = children {
                usage.bank::<usize, E>(children.capacity())?;
            }
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            usage.bank::<usize, E>(dependents.capacity())?;
            usage.kind(kind, &mut step)?;
        }
        for registry in registries {
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            let RegistryRecord {
                definitions,
                complete: _,
                scope: _,
                scope_established: _,
            } = registry;
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            usage.bank::<HtmlCustomElementDefinition, E>(definitions.capacity())?;
            for definition in definitions {
                step().map_err(HtmlCaptureStorageError::Interrupted)?;
                let HtmlCustomElementDefinition {
                    name,
                    local_name,
                    form_associated: _,
                    disable_shadow: _,
                } = definition;
                usage.string(name, &mut step)?;
                usage.string(local_name, &mut step)?;
            }
        }
        Ok(usage)
    }
}

#[cfg(test)]
mod tests;

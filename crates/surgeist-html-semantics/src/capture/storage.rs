//! Actual immutable capture payload, observed without allocation or caching.
use super::*;
use std::mem::size_of;
use surgeist_html::{
    security::nonce::HtmlCapturedNonce,
    values::{
        DocumentString, OrderedAttributes, QualifiedName, TextBlock, TextBlockTable,
        TextBlockTableError,
    },
};

/// How a string is counted. A string of a record is counted by its occurrence, because the capture
/// lists its block and counts the block once; any other string is counted by the capacity of its
/// block, since nothing lists the block.
#[derive(Clone, Copy)]
enum Count {
    Listed,
    Retained,
}

/// The blocks that borrowed strings of one capture index. Each entry holds the bytes and the units
/// its block keeps alive, which the capture accounts for once however many strings borrow it, and
/// the number of the listing that made it, so that a rollback drops the entries made after a
/// checkpoint.
pub(super) type BlockTable = TextBlockTable<(usize, usize, usize)>;

/// Visits every string of `kind` that a record of the capture holds.
pub(super) fn kind_strings(kind: &HtmlCapturedNodeKind, visit: &mut dyn FnMut(&DocumentString)) {
    match kind {
        HtmlCapturedNodeKind::Element(element) => {
            let name = |name: &QualifiedName, visit: &mut dyn FnMut(&DocumentString)| {
                if let Some(namespace) = name.namespace().uri() {
                    visit(namespace);
                }
                if let Some(prefix) = name.prefix() {
                    visit(prefix);
                }
                visit(name.local_name());
            };
            name(element.name(), visit);
            let attributes = element.attributes().shared_attributes();
            for attribute in attributes.iter() {
                name(attribute.name(), visit);
                visit(attribute.value());
            }
            if let HtmlCapturedIsValue::Value(value) = &element.is_value {
                visit(value);
            }
            if let HtmlCapturedNonce::Value(value) = &element.nonce {
                visit(value);
            }
        }
        HtmlCapturedNodeKind::Text(value) | HtmlCapturedNodeKind::Comment(value) => visit(value),
        HtmlCapturedNodeKind::ProcessingInstruction(HtmlCapturedProcessingInstruction {
            target,
            data,
        }) => {
            visit(target);
            visit(data);
        }
        HtmlCapturedNodeKind::Doctype(doctype) => {
            visit(doctype.name());
            if let Some(public) = doctype.public_id() {
                visit(public);
            }
            if let Some(system) = doctype.system_id() {
                visit(system);
            }
        }
        HtmlCapturedNodeKind::Document(_) | HtmlCapturedNodeKind::Fragment(_) => (),
    }
}

/// The blocks one operation listed in a [`BlockTable`], which the operation takes back if it
/// does not complete.
pub(super) struct Listing(Vec<TextBlock>);

impl Listing {
    /// Removes the entries this operation made, leaving the table as it was.
    pub(super) fn undo(self, table: &mut BlockTable) {
        for block in &self.0 {
            table.remove(block);
        }
    }
}

/// Drops the entries of every listing numbered `checkpoint` or later, so that a table returns to
/// the blocks the records it keeps list.
pub(super) fn roll_back_blocks(table: &mut BlockTable, checkpoint: usize) {
    table.retain(|&(_, _, number)| number < checkpoint);
}

/// Lists the blocks of the strings `strings` visits that the table does not list yet. The room
/// for every entry is reserved before the first is listed, so a refusal lists nothing.
pub(super) fn list_blocks(
    table: &mut BlockTable,
    sequence: &mut usize,
    strings: impl Fn(&mut dyn FnMut(&DocumentString)),
) -> Result<Listing, TextBlockTableError> {
    let mut missing = 0usize;
    strings(&mut |string| {
        if let Some(block) = string.borrowed_block()
            && table.find(block).is_none()
        {
            missing = missing.saturating_add(1);
        }
    });
    table.reserve(missing)?;
    let mut listed = Vec::new();
    listed
        .try_reserve_exact(missing)
        .map_err(|_| TextBlockTableError::AllocationFailed)?;
    let mut refusal = None;
    strings(&mut |string| {
        if refusal.is_some() {
            return;
        }
        let Some(block) = string.borrowed_block() else {
            return;
        };
        if table.find(block).is_some() {
            return;
        }
        let number = *sequence;
        let Some(next) = number.checked_add(1) else {
            // A sequence of listings that long cannot be numbered, and is refused as an exhausted
            // table is.
            refusal = Some(TextBlockTableError::AllocationFailed);
            return;
        };
        match table.insert(block, (block.retained_bytes(), block.capacity(), number)) {
            // The room for the entry and for the record of it was reserved above.
            Ok(()) => {
                listed.push(block.clone());
                *sequence = next;
            }
            Err(error) => refusal = Some(error),
        }
    });
    let listing = Listing(listed);
    match refusal {
        Some(error) => {
            listing.undo(table);
            Err(error)
        }
        None => Ok(listing),
    }
}

/// Actual capture-owned payload, independent of logical formation usage.
///
/// Includes the private data header, actual bank capacities and nested payloads.
/// Shared strings, attributes and URL records count conservatively per retained
/// occurrence. A string that is a range of a text block counts its own length per occurrence, and
/// each such block counts once for the capture, however many strings borrow it. This is not a
/// unique physical-allocation total. A consumer can
/// count a whole aliased capture once using its existing `HtmlCaptureId`.
/// Excludes the public `HtmlCapture` handle, opaque Arc control blocks and
/// allocator bookkeeping. Separately supplied captured Base facts are not owned
/// by a capture and are not included.
///
/// This is a measurement of actual capacities with checked sums, not a counter: the consumer
/// that retains the value charges it to its own accounting.
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
        count: Count,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureStorageError<E>> {
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        let capacity = match count {
            Count::Listed => value.occurrence_units(),
            Count::Retained => value.retained_units(),
        };
        self.add_units(capacity)?;
        self.add_bytes(size_of::<Vec<u16>>())?;
        self.bank::<u16, E>(capacity)
    }
    fn name<E>(
        &mut self,
        name: &QualifiedName,
        count: Count,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureStorageError<E>> {
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        if let Some(namespace) = name.namespace().uri() {
            self.string(namespace, count, step)?;
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        if let Some(prefix) = name.prefix() {
            self.string(prefix, count, step)?;
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        self.string(name.local_name(), count, step)
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
        self.name(name, Count::Listed, step)?;
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        let attributes = attributes.shared_attributes();
        self.add_bytes(size_of::<Vec<surgeist_html::values::Attribute>>())?;
        self.bank::<surgeist_html::values::Attribute, E>(attributes.capacity())?;
        for attribute in attributes.iter() {
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            self.name(attribute.name(), Count::Listed, step)?;
            step().map_err(HtmlCaptureStorageError::Interrupted)?;
            self.string(attribute.value(), Count::Listed, step)?;
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        match is_value {
            HtmlCapturedIsValue::Value(value) => self.string(value, Count::Listed, step)?,
            HtmlCapturedIsValue::Absent | HtmlCapturedIsValue::Unavailable => (),
        }
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        match nonce {
            HtmlCapturedNonce::Value(value) => self.string(value, Count::Listed, step)?,
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
                self.string(value, Count::Listed, step)?;
            }
            HtmlCapturedNodeKind::ProcessingInstruction(HtmlCapturedProcessingInstruction {
                target,
                data,
            }) => {
                self.string(target, Count::Listed, step)?;
                self.string(data, Count::Listed, step)?;
            }
            HtmlCapturedNodeKind::Doctype(doctype) => {
                step().map_err(HtmlCaptureStorageError::Interrupted)?;
                self.string(doctype.name(), Count::Listed, step)?;
                step().map_err(HtmlCaptureStorageError::Interrupted)?;
                if let Some(public) = doctype.public_id() {
                    self.string(public, Count::Listed, step)?;
                }
                step().map_err(HtmlCaptureStorageError::Interrupted)?;
                if let Some(system) = doctype.system_id() {
                    self.string(system, Count::Listed, step)?;
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
    usage.string(value, Count::Retained, step)?;
    Ok(usage)
}

pub(crate) fn option_value_element<E>(
    name: &QualifiedName,
    attributes: &OrderedAttributes,
    saved_is: Option<&DocumentString>,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<HtmlCaptureRetainedStorageUsage, HtmlCaptureStorageError<E>> {
    let mut usage = HtmlCaptureRetainedStorageUsage { bytes: 0, units: 0 };
    usage.name(name, Count::Retained, step)?;
    step().map_err(HtmlCaptureStorageError::Interrupted)?;
    let values = attributes.shared_attributes();
    usage.add_bytes(size_of::<Vec<surgeist_html::values::Attribute>>())?;
    usage.bank::<surgeist_html::values::Attribute, E>(values.capacity())?;
    for attribute in values.iter() {
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        usage.name(attribute.name(), Count::Retained, step)?;
        usage.string(attribute.value(), Count::Retained, step)?;
    }
    if let Some(value) = saved_is {
        usage.string(value, Count::Retained, step)?;
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
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlCaptureRetainedStorageUsage, HtmlCaptureStorageError<E>> {
        retained_storage_usage_with(&self.0, step)
    }
}

// Same observer before publication: no temporary capture/Arc is needed to price
// the actual original builder data. Public observation keeps its exact tariff.
pub(super) fn retained_storage_usage_with<E>(
    data: &CaptureData,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<HtmlCaptureRetainedStorageUsage, HtmlCaptureStorageError<E>> {
    step().map_err(HtmlCaptureStorageError::Interrupted)?;
    let CaptureData {
        identity: _,
        host_revision: _,
        records,
        registries,
        blocks,
        block_sequence: _,
        usage: _,
    } = data;
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
    step().map_err(HtmlCaptureStorageError::Interrupted)?;
    usage.add_bytes(
        blocks
            .capacity()
            .checked_mul(BlockTable::ENTRY_BYTES)
            .ok_or(HtmlCaptureStorageError::RepresentationExhausted)?,
    )?;
    for &(bytes, units, _) in blocks.values() {
        step().map_err(HtmlCaptureStorageError::Interrupted)?;
        usage.add_bytes(bytes)?;
        usage.add_units(units)?;
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
            usage.string(name, Count::Listed, &mut step)?;
            usage.string(local_name, Count::Listed, &mut step)?;
        }
    }
    Ok(usage)
}

#[cfg(test)]
mod tests;

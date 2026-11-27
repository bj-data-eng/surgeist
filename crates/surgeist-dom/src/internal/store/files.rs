//! Immutable File and FileList objects used by native File Upload controls.

use super::{
    DomError, bank, custody, payload,
    source::{formation::Formation, storage},
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use surgeist_html::{elements::input::HTML_INPUT_FILENAME_PREFIX, values::DocumentString};

static NEXT_OBJECT: AtomicU64 = AtomicU64::new(1);

fn object_identity() -> Result<u64, DomError> {
    NEXT_OBJECT
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            next.checked_add(1)
        })
        .map_err(|_| DomError::RepresentationExhausted)
}

/// Stable identity of a native File object.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomFileId(u64);

/// Stable identity of a native FileList object.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomFileListId(u64);

struct FileData {
    identity: DomFileId,
    name: DocumentString,
    media_type: DocumentString,
    bytes: Vec<u8>,
}

/// A native File with a path-free selected name and an immutable body.
#[derive(Clone)]
pub struct DomFile(Arc<payload::Payload<FileData>>);

impl DomFile {
    /// Form an independent File from already owned native data. No byte
    /// or string backing is copied. Canonical admission occurs when Source
    /// consumes a FileList containing this object.
    pub fn from_owned_parts(
        name: DocumentString,
        media_type: DocumentString,
        bytes: Vec<u8>,
    ) -> Result<Self, DomError> {
        if name.code_units().contains(&u16::from(b'\\')) {
            return Err(DomError::InputFileNameContainsPath);
        }
        let mut extent = storage::Payload::default();
        extent.string(&name)?;
        extent.string(&media_type)?;
        extent.bank::<u8>(bytes.capacity())?;
        let own_usage = storage::typed::<FileData>(extent)?;
        let identity = DomFileId(object_identity()?);
        Ok(Self(Arc::new(payload::Payload {
            identity: custody::MaterializationId::fresh()?,
            own_usage,
            value: FileData {
                identity,
                name,
                media_type,
                bytes,
            },
        })))
    }

    /// Identity of this File, independent of payload equality and alias count.
    pub fn id(&self) -> DomFileId {
        self.0.value.identity
    }

    /// The selected filename with no path components.
    pub fn name(&self) -> &DocumentString {
        &self.0.value.name
    }

    /// The supplied media-type spelling.
    pub fn media_type(&self) -> &DocumentString {
        &self.0.value.media_type
    }

    /// Exact immutable File body.
    pub fn bytes(&self) -> &[u8] {
        &self.0.value.bytes
    }
}

impl std::fmt::Debug for DomFile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DomFile")
            .field("id", &self.id())
            .field("byte_length", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for DomFile {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for DomFile {}

struct ListData {
    identity: DomFileListId,
    files: Vec<DomFile>,
    filename_value: Option<DocumentString>,
}

/// An ordered, immutable, identity-bearing list of native File objects.
#[derive(Clone)]
pub struct DomFileList(Arc<payload::Payload<ListData>>);

impl DomFileList {
    /// Form a distinct List from already owned ordered File handles.
    /// Original File identities and duplicate occurrences are preserved.
    pub fn from_files(files: Vec<DomFile>) -> Result<Self, DomError> {
        let filename_value = files
            .first()
            .map(|first| {
                let name = first.name().code_units();
                let length = HTML_INPUT_FILENAME_PREFIX
                    .len()
                    .checked_add(name.len())
                    .ok_or(DomError::RepresentationExhausted)?;
                let mut units = Vec::new();
                units
                    .try_reserve_exact(length)
                    .map_err(|_| DomError::AllocationFailed)?;
                units.extend_from_slice(HTML_INPUT_FILENAME_PREFIX);
                units.extend_from_slice(name);
                Ok(DocumentString::from_utf16(units))
            })
            .transpose()?;
        let mut extent = storage::Payload::default();
        extent.bank::<DomFile>(files.capacity())?;
        if let Some(value) = &filename_value {
            extent.string(value)?;
        }
        let own_usage = storage::typed::<ListData>(extent)?;
        let identity = DomFileListId(object_identity()?);
        Ok(Self(Arc::new(payload::Payload {
            identity: custody::MaterializationId::fresh()?,
            own_usage,
            value: ListData {
                identity,
                files,
                filename_value,
            },
        })))
    }

    /// Identity of this List, even when it has no members.
    pub fn id(&self) -> DomFileListId {
        self.0.value.identity
    }

    /// Ordered File handles, including duplicate occurrences.
    pub fn files(&self) -> &[DomFile] {
        &self.0.value.files
    }

    /// The number of ordered File occurrences.
    pub fn len(&self) -> usize {
        self.files().len()
    }

    /// Whether this List has any File occurrences.
    pub fn is_empty(&self) -> bool {
        self.files().is_empty()
    }

    /// The corresponding HTML Input Filename-mode value.
    pub fn filename_value(&self) -> &[u16] {
        self.0
            .value
            .filename_value
            .as_ref()
            .map_or(&[], DocumentString::code_units)
    }

    pub(crate) fn controlled_empty(
        registry: &bank::CustodyRegistry,
        budget: Option<&Formation>,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<(Self, bank::ObjectCustody), DomError> {
        let empty = storage::Payload::default();
        let formed = payload::form_group(
            registry,
            budget,
            empty,
            storage::typed::<ListData>(empty)?,
            admit,
            || {
                Ok(ListData {
                    identity: DomFileListId(object_identity()?),
                    files: Vec::new(),
                    filename_value: None,
                })
            },
        )?;
        Ok((Self(formed.value), formed.custody))
    }

    pub(in crate::internal::store) fn adopt(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<bank::ObjectCustody, DomError> {
        registry.adopt_object_with_children(
            self.0.identity,
            self.0.own_usage,
            |admit| self.file_dependencies(registry, admit),
            admit,
        )
    }

    fn file_dependencies(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Option<bank::Dependency>, DomError> {
        if self.is_empty() {
            return Ok(None);
        }
        let usage = |capacity: usize| {
            capacity
                .checked_mul(size_of::<bank::ObjectCustody>())
                .and_then(|bytes| bytes.checked_add(size_of::<FileListCustody>()))
                .map(|bytes| custody::Usage::new(bytes, 0))
                .ok_or(DomError::RepresentationExhausted)
        };
        let requested = usage(self.len())?;
        let debit = registry.ledger().retain(requested, &mut *admit)?;
        let mut controls = custody::Held::new(FileListCustody(Vec::new()), debit);
        controls
            .value_mut()
            .0
            .try_reserve_exact(self.len())
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = usage(controls.value().0.capacity())?;
        controls
            .debit_mut()
            .grow(actual.checked_sub(requested)?, &mut *admit)?;
        for file in self.files() {
            controls.value_mut().0.push(registry.adopt_object(
                file.0.identity,
                file.0.own_usage,
                &mut *admit,
            )?);
        }
        bank::Dependency::from_retained(controls, actual, admit).map(Some)
    }
}

pub(super) struct FileListCustody(pub(super) Vec<bank::ObjectCustody>);

pub(crate) fn input_dependency(
    value: &crate::internal::input::DomInputState,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let Some(files) = value.files.as_ref() else {
        return Ok(None);
    };
    let control = files.adopt(registry, admit)?;
    input_file_list_dependency(control, registry, admit).map(Some)
}

pub(super) struct InputFileListCustody {
    pub(super) _list: bank::ObjectCustody,
}

pub(crate) fn input_file_list_dependency(
    control: bank::ObjectCustody,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<bank::Dependency, DomError> {
    bank::Dependency::new(
        InputFileListCustody { _list: control },
        registry.ledger(),
        admit,
    )
}

impl std::fmt::Debug for DomFileList {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DomFileList")
            .field("id", &self.id())
            .field("length", &self.len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for DomFileList {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for DomFileList {}

#[cfg(test)]
impl super::compact_tests::typed_graph::Walker<'_> {
    pub(super) fn file_list(&mut self, list: &DomFileList) {
        use super::compact_tests::typed_graph::Raw;

        let data = &list.0.value;
        let mut raw = Raw::default();
        self.buffer(&mut raw, &data.files, "file.list.members");
        if let Some(value) = &data.filename_value {
            self.text(&mut raw, value);
        }
        self.object(&list.0, raw, "file.list");
        for file in &data.files {
            let data = &file.0.value;
            let mut raw = Raw::default();
            self.buffer(&mut raw, &data.bytes, "file.body");
            self.text(&mut raw, &data.name.block().whole());
            self.text(&mut raw, &data.media_type.block().whole());
            self.object(&file.0, raw, "file.data");
        }
    }
}

//! Immutable, bounded SFNT resources and collection faces.
//!
//! Admission validates container structure, not OpenType table semantics.
//! See the crate README for allocation accounting and capability limits.

#![forbid(unsafe_code)]

use std::{
    fmt,
    hash::{Hash, Hasher},
    mem::size_of,
    sync::Arc,
};

mod sfnt;

/// Caller policy; zero is a valid rejecting cap. No limits are inferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    encoded_bytes: usize,
    decoded_bytes: usize,
    table_bytes: usize,
    tables: usize,
    faces: usize,
    workspace_bytes: usize,
    live_bytes: usize,
}

impl DecodeLimits {
    /// Sets encoded capacity, decoded length, individual table length, aggregate
    /// table count, face count, temporary workspace and peak live byte caps.
    pub const fn new(
        encoded_bytes: usize,
        decoded_bytes: usize,
        table_bytes: usize,
        tables: usize,
        faces: usize,
        workspace_bytes: usize,
        live_bytes: usize,
    ) -> Self {
        Self {
            encoded_bytes,
            decoded_bytes,
            table_bytes,
            tables,
            faces,
            workspace_bytes,
            live_bytes,
        }
    }
    /// Maximum retained encoded `Vec` capacity, including unused capacity.
    pub const fn encoded_bytes(self) -> usize {
        self.encoded_bytes
    }
    /// Maximum logical reconstructed SFNT length.
    pub const fn decoded_bytes(self) -> usize {
        self.decoded_bytes
    }
    /// Maximum logical length of any individual table.
    pub const fn table_bytes(self) -> usize {
        self.table_bytes
    }
    /// Maximum sum of table counts across all faces, including shared tables.
    pub const fn tables(self) -> usize {
        self.tables
    }
    /// Maximum number of collection faces.
    pub const fn faces(self) -> usize {
        self.faces
    }
    /// Maximum requested temporary allocation payload.
    pub const fn workspace_bytes(self) -> usize {
        self.workspace_bytes
    }
    /// Maximum requested simultaneously live payload and known owned metadata.
    pub const fn live_bytes(self) -> usize {
        self.live_bytes
    }
}

/// A caller-selected admission bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LimitKind {
    EncodedBytes,
    DecodedBytes,
    TableBytes,
    Tables,
    Faces,
    WorkspaceBytes,
    LiveBytes,
}

/// Structural container defect. Table payload semantics remain consumer-owned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InvalidKind {
    Signature,
    Truncated,
    CollectionVersion,
    EmptyCollection,
    TableOrder,
    TableTag,
    UnalignedOffset,
    Overlap,
    CollectionSignature,
}

/// A malformed container and the byte offset of the affected field or region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidContainer {
    kind: InvalidKind,
    offset: usize,
}
impl InvalidContainer {
    /// Structural reason for rejection.
    pub const fn kind(self) -> InvalidKind {
        self.kind
    }
    /// Byte offset in the decoded container.
    pub const fn offset(self) -> usize {
        self.offset
    }
}

/// Admission failure; a rejected input never produces a partial resource.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DecodeError {
    LimitExceeded {
        kind: LimitKind,
        required: usize,
        limit: usize,
    },
    InvalidContainer(InvalidContainer),
    AllocationFailed,
    SizeOverflow,
}
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExceeded {
                kind,
                required,
                limit,
            } => write!(f, "{kind:?} requires {required}, limit is {limit}"),
            Self::InvalidContainer(error) => {
                write!(f, "{:?} at container offset {}", error.kind, error.offset)
            }
            Self::AllocationFailed => f.write_str("font allocation failed"),
            Self::SizeOverflow => f.write_str("font size arithmetic overflowed"),
        }
    }
}
impl std::error::Error for DecodeError {}

/// Exact encoded container format, independent of reconstructed face flavor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SourceFormat {
    Sfnt,
    Collection,
}

#[derive(Debug)]
struct SourceInner {
    bytes: Vec<u8>,
    format: SourceFormat,
}

/// Strong immutable provenance retaining the exact original encoded bytes.
#[derive(Clone)]
pub struct FontSource(Arc<SourceInner>);
impl FontSource {
    pub(crate) fn from_encoded(
        bytes: Vec<u8>,
        format: SourceFormat,
        limits: DecodeLimits,
    ) -> Result<Self, DecodeError> {
        check(
            LimitKind::EncodedBytes,
            bytes.capacity(),
            limits.encoded_bytes,
        )?;
        check(
            LimitKind::LiveBytes,
            add(bytes.capacity(), source_metadata())?,
            limits.live_bytes,
        )?;
        Ok(Self(Arc::new(SourceInner { bytes, format })))
    }
    /// Original encoded bytes, including bytes unused by table directories.
    pub fn bytes(&self) -> &[u8] {
        &self.0.bytes
    }
    /// Original container format.
    pub fn format(&self) -> SourceFormat {
        self.0.format
    }
    /// Strong allocation identity; equal content does not imply equal identity.
    pub fn identity(&self) -> SourceId {
        SourceId(self.0.clone())
    }
}
impl fmt::Debug for FontSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontSource")
            .field("format", &self.format())
            .field("bytes", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

/// Strong source identity, retained independently of decoded resource identity.
#[derive(Clone)]
pub struct SourceId(Arc<SourceInner>);
impl PartialEq for SourceId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for SourceId {}
impl Hash for SourceId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
impl fmt::Debug for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SourceId")
            .field(&Arc::as_ptr(&self.0))
            .finish()
    }
}

#[derive(Debug)]
struct ResourceInner {
    source: FontSource,
    decoded: Option<Vec<u8>>,
    face_offsets: Vec<usize>,
}

/// One completely admitted decoded SFNT container. Clones retain its identity.
#[derive(Clone)]
pub struct FontResource(Arc<ResourceInner>);
impl FontResource {
    /// Admits raw SFNT/OTF or TTC/OTC bytes without copying their payload.
    /// The caller's allocation before ownership transfer is outside this call's
    /// allocation-failure boundary; its retained capacity is nevertheless charged.
    pub fn from_sfnt(bytes: Vec<u8>, limits: DecodeLimits) -> Result<Self, DecodeError> {
        let format = if bytes.get(..4) == Some(b"ttcf") {
            SourceFormat::Collection
        } else {
            SourceFormat::Sfnt
        };
        let source = FontSource::from_encoded(bytes, format, limits)?;
        Self::from_backing(source, None, limits)
    }
    pub(crate) fn from_backing(
        source: FontSource,
        decoded: Option<Vec<u8>>,
        limits: DecodeLimits,
    ) -> Result<Self, DecodeError> {
        let bytes = decoded.as_deref().unwrap_or_else(|| source.bytes());
        check(LimitKind::DecodedBytes, bytes.len(), limits.decoded_bytes)?;
        let retained = add(
            source.0.bytes.capacity(),
            decoded.as_ref().map_or(0, Vec::capacity),
        )?;
        let base = add(retained, add(source_metadata(), resource_metadata())?)?;
        check(LimitKind::LiveBytes, base, limits.live_bytes)?;
        let face_offsets = sfnt::admit(bytes, limits, base)?;
        Ok(Self(Arc::new(ResourceInner {
            source,
            decoded,
            face_offsets,
        })))
    }
    /// Complete decoded container backing expected by SFNT readers.
    pub fn bytes(&self) -> &[u8] {
        self.0
            .decoded
            .as_deref()
            .unwrap_or_else(|| self.0.source.bytes())
    }
    /// Exact source provenance. Raw resources share its payload allocation.
    pub fn source(&self) -> &FontSource {
        &self.0.source
    }
    /// Strong decoded-resource identity; this is never a content hash.
    pub fn identity(&self) -> ResourceId {
        ResourceId(self.0.clone())
    }
    /// Actual number of faces, retaining collection order.
    pub fn face_count(&self) -> u32 {
        u32::try_from(self.0.face_offsets.len()).expect("admitted u32 face count")
    }
    /// Selects an original collection index, or returns `None` when out of range.
    pub fn face(&self, index: u32) -> Option<FontFace> {
        self.0.face_offsets.get(usize::try_from(index).ok()?)?;
        Some(FontFace {
            resource: self.clone(),
            index,
        })
    }
}
impl fmt::Debug for FontResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontResource")
            .field("source", self.source())
            .field("faces", &self.face_count())
            .field("bytes", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

/// Strong decoded-resource identity; retaining it retains the resource.
#[derive(Clone)]
pub struct ResourceId(Arc<ResourceInner>);
impl PartialEq for ResourceId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ResourceId {}
impl Hash for ResourceId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
impl fmt::Debug for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ResourceId")
            .field(&Arc::as_ptr(&self.0))
            .finish()
    }
}

/// Validated collection face retaining the complete actual resource.
#[derive(Clone, Debug)]
pub struct FontFace {
    resource: FontResource,
    index: u32,
}
impl FontFace {
    /// Resource whose immutable container admitted this face.
    pub fn resource(&self) -> &FontResource {
        &self.resource
    }
    /// Original collection index (zero for standalone SFNT).
    pub fn index(&self) -> u32 {
        self.index
    }
    fn offset(&self) -> usize {
        self.resource.0.face_offsets[self.index as usize]
    }
    /// Original SFNT scaler signature, independent of table semantic support.
    pub fn sfnt_version(&self) -> [u8; 4] {
        self.resource.bytes()[self.offset()..self.offset() + 4]
            .try_into()
            .expect("admitted signature")
    }
    /// Number of records in this face's admitted directory.
    pub fn table_count(&self) -> u16 {
        sfnt::u16_at(self.resource.bytes(), self.offset() + 4).expect("admitted directory")
    }
    /// Borrows the exact logical table payload. Absence is distinct from an empty table.
    pub fn table(&self, tag: [u8; 4]) -> Option<&[u8]> {
        let bytes = self.resource.bytes();
        let mut left = 0usize;
        let mut right = usize::from(self.table_count());
        while left < right {
            let middle = left + (right - left) / 2;
            let record = self.offset() + 12 + middle * 16;
            match bytes[record..record + 4].cmp(&tag) {
                std::cmp::Ordering::Less => left = middle + 1,
                std::cmp::Ordering::Greater => right = middle,
                std::cmp::Ordering::Equal => {
                    let offset = sfnt::offset_at(bytes, record + 8).expect("admitted table offset");
                    let length =
                        sfnt::offset_at(bytes, record + 12).expect("admitted table length");
                    return Some(&bytes[offset..offset + length]);
                }
            }
        }
        None
    }
}
impl PartialEq for FontFace {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.resource.0, &other.resource.0)
    }
}
impl Eq for FontFace {}
impl Hash for FontFace {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.resource.0).hash(state);
        self.index.hash(state);
    }
}

fn source_metadata() -> usize {
    size_of::<SourceInner>() + 2 * size_of::<usize>()
}
fn resource_metadata() -> usize {
    size_of::<ResourceInner>() + 2 * size_of::<usize>()
}
fn add(left: usize, right: usize) -> Result<usize, DecodeError> {
    left.checked_add(right).ok_or(DecodeError::SizeOverflow)
}
fn multiply(left: usize, right: usize) -> Result<usize, DecodeError> {
    left.checked_mul(right).ok_or(DecodeError::SizeOverflow)
}
fn check(kind: LimitKind, required: usize, limit: usize) -> Result<(), DecodeError> {
    if required > limit {
        Err(DecodeError::LimitExceeded {
            kind,
            required,
            limit,
        })
    } else {
        Ok(())
    }
}
fn invalid(kind: InvalidKind, offset: usize) -> DecodeError {
    DecodeError::InvalidContainer(InvalidContainer { kind, offset })
}

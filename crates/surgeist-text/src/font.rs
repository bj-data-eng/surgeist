use std::{fmt, sync::Arc};

use linebender_resource_handle::Blob;
use skrifa::{Tag, raw::TableProvider};

use crate::{Error, ErrorCode, ErrorDetail, FontCatalogueIdentity, NumericRequirement, Result};

/// Strong actual font resource, collection face and associated catalogue publication.
///
/// Resource equality preserves Blob identity plus collection index. Catalogue
/// provenance is qualified separately; equal resources can have distinct publications.
#[derive(Clone, Debug)]
pub struct FontData {
    data: Blob<u8>,
    index: u32,
    catalogue: FontCatalogueIdentity,
}

impl PartialEq for FontData {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.index == other.index
    }
}

impl FontData {
    /// Original whole-resource bytes, without copying.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.data.as_ref()
    }
    /// Collection face index.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }
    /// Actual retained publication, independent of resource equality.
    #[must_use]
    pub const fn catalogue_identity(&self) -> &FontCatalogueIdentity {
        &self.catalogue
    }
    /// Borrows this same resource, face and publication.
    #[must_use]
    pub fn face(&self) -> FontFace<'_> {
        FontFace::new(&self.data, self.index, &self.catalogue)
    }
    /// Selects an explicit unmodified face and complete canonical coordinates.
    /// This does not match an unresolved Style or silently discard faux synthesis.
    pub fn unsynthesized_instance<'a>(
        &'a self,
        size: f32,
        normalized_coords: &'a [i16],
    ) -> Result<EffectiveFontInstance<'a>> {
        EffectiveFontInstance::qualify(self.face(), size, normalized_coords)
    }
}

/// A borrowed actual resource, collection face and associated publication.
#[derive(Clone, Copy)]
pub struct FontFace<'a> {
    data: &'a Blob<u8>,
    index: u32,
    catalogue: &'a FontCatalogueIdentity,
}

impl fmt::Debug for FontFace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontFace")
            .field("index", &self.index())
            .field("bytes", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

impl<'a> FontFace<'a> {
    pub(crate) const fn new(
        data: &'a Blob<u8>,
        index: u32,
        catalogue: &'a FontCatalogueIdentity,
    ) -> Self {
        Self {
            data,
            index,
            catalogue,
        }
    }

    /// Actual catalogue publication under which this face was associated.
    #[must_use]
    pub const fn catalogue_identity(&self) -> &'a FontCatalogueIdentity {
        self.catalogue
    }

    /// Retains the actual strong resource and publication without copying bytes.
    #[must_use]
    pub fn to_owned(&self) -> FontData {
        FontData {
            data: self.data.clone(),
            index: self.index,
            catalogue: self.catalogue.clone(),
        }
    }

    /// Whole original resource bytes, borrowed without copying.
    #[must_use]
    pub fn bytes(&self) -> &'a [u8] {
        self.data.as_ref()
    }

    /// Actual face index within that resource.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Compares the actual strong allocation and collection index, not IDs or content.
    #[must_use]
    pub fn is_same_face(&self, other: &Self) -> bool {
        let (left, _) = self.data.clone().into_raw_parts();
        let (right, _) = other.data.clone().into_raw_parts();
        self.index() == other.index() && Arc::ptr_eq(&left, &right)
    }
}

/// A borrowed complete canonical font instance with no unresolved faux synthesis.
#[derive(Clone, Copy, Debug)]
pub struct EffectiveFontInstance<'a> {
    face: FontFace<'a>,
    size: f32,
    axes: usize,
    stored: &'a [i16],
}

impl<'a> EffectiveFontInstance<'a> {
    pub(crate) fn qualify(face: FontFace<'a>, size: f32, stored: &'a [i16]) -> Result<Self> {
        if !size.is_finite() || size <= 0.0 {
            return Err(
                Error::new(ErrorCode::InvalidFontInstance, "invalid shaping size").with_detail(
                    ErrorDetail::InvalidNumericField {
                        field: "shaping font size",
                        value: size,
                        requirement: NumericRequirement::FiniteGreaterThanZero,
                    },
                ),
            );
        }
        let axes = axis_count(face)?;
        if !stored.is_empty() && stored.len() != axes {
            return Err(Error::new(
                ErrorCode::InvalidFontInstance,
                "normalized coordinates do not cover the actual axes",
            )
            .with_detail(ErrorDetail::FontCoordinateCount {
                expected: axes,
                actual: stored.len(),
            }));
        }
        for (axis, &bits) in stored.iter().enumerate() {
            if !(-16384..=16384).contains(&bits) {
                return Err(Error::new(
                    ErrorCode::InvalidFontInstance,
                    "noncanonical normalized font coordinate",
                )
                .with_detail(ErrorDetail::FontCoordinate { axis, bits }));
            }
        }
        Ok(Self {
            face,
            size,
            axes,
            stored,
        })
    }

    /// The actual borrowed face used by this instance.
    #[must_use]
    pub const fn face(&self) -> FontFace<'a> {
        self.face
    }

    /// Actual canonical instance size.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.size
    }

    /// Complete actual variation-axis count; zero only for a proven static face.
    #[must_use]
    pub const fn axis_count(&self) -> usize {
        self.axes
    }

    /// Complete normalized F2Dot14 bits in actual axis order, without allocation.
    pub fn normalized_coords(&self) -> impl ExactSizeIterator<Item = i16> + Clone + '_ {
        (0..self.axes).map(|axis| {
            if self.stored.is_empty() {
                0
            } else {
                self.stored[axis]
            }
        })
    }
}

fn axis_count(face: FontFace<'_>) -> Result<usize> {
    let font = skrifa::FontRef::from_index(face.bytes(), face.index()).map_err(|_| malformed())?;
    let tag = Tag::new(b"fvar");
    let mut records = font
        .table_directory()
        .table_records()
        .iter()
        .filter(|record| record.tag() == tag);
    if records.next().is_none() {
        return Ok(0);
    }
    if records.next().is_some() {
        return Err(malformed());
    }
    // Table presence is established separately: inaccessible/truncated data must
    // never be reclassified as a static font by a permissive metadata accessor.
    let data = font.table_data(tag).ok_or_else(malformed)?;
    let fvar = font.fvar().map_err(|_| malformed())?;
    let count = usize::from(fvar.axis_count());
    let offset =
        usize::try_from(fvar.axis_instance_arrays_offset().to_u32()).map_err(|_| malformed())?;
    let instance_size = usize::from(fvar.instance_size());
    let required_instance = count.checked_mul(4).and_then(|n| n.checked_add(4));
    let axes_end = count.checked_mul(20).and_then(|n| offset.checked_add(n));
    let table_end = axes_end.and_then(|n| {
        usize::from(fvar.instance_count())
            .checked_mul(instance_size)
            .and_then(|instances| n.checked_add(instances))
    });
    if fvar.version().major != 1
        || fvar.version().minor != 0
        || data.as_bytes().get(6..8) != Some(&[0, 2])
        || count == 0
        || fvar.axis_size() != 20
        || offset < 16
        || !required_instance.is_some_and(|n| instance_size == n || instance_size == n + 2)
        || !table_end.is_some_and(|end| end <= data.len())
    {
        return Err(malformed());
    }
    let axes = fvar.axes().map_err(|_| malformed())?;
    if axes.len() != count {
        return Err(malformed());
    }
    for (index, axis) in axes.iter().enumerate() {
        if axis.min_value() > axis.default_value()
            || axis.default_value() > axis.max_value()
            || axes[..index]
                .iter()
                .any(|previous| previous.axis_tag() == axis.axis_tag())
        {
            return Err(malformed());
        }
    }
    Ok(count)
}

fn malformed() -> Error {
    Error::new(
        ErrorCode::InvalidFontInstance,
        "malformed actual font variation metadata",
    )
    .with_detail(ErrorDetail::MalformedFontVariations)
}

mod metrics;
pub use metrics::{FontMetrics, StrikeoutMetrics};

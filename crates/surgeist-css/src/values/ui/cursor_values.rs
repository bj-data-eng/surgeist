//! Checked authored UI 4 cursor images, before resource selection or hotspot clamping.

use crate::numeric::{CssNonNegative, CssNumber, CssNumeric, CssResolution};
use crate::values::common::CssUrl;
use crate::values::content::CssContentString;
use crate::values::ui::CssCursorKeyword;

/// An authored cursor keyword or ordered image alternatives with a required fallback.
///
/// # Specified serialization
///
/// Emits canonical authored cursor syntax without loading resources or clamping hotspots. Candidate
/// order and omission survive; set descriptors emit resolution before type. The standard prefixed
/// image-set alias emits `image-set()`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursor {
    /// A cursor keyword with no image alternatives.
    Keyword(CssCursorKeyword),
    /// One or more image alternatives followed by the mandatory fallback keyword.
    Images(CssCursorImages),
}

impl CssCursor {
    /// Requires at least one image alternative; keyword-only values use `Keyword`.
    #[must_use]
    pub fn try_images(images: Vec<CssCursorImage>, fallback: CssCursorKeyword) -> Option<Self> {
        CssCursorImages::try_new(images, fallback).map(Self::Images)
    }
}

/// A nonempty ordered list of image alternatives and its mandatory keyword fallback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCursorImages {
    images: Vec<CssCursorImage>,
    fallback: CssCursorKeyword,
}

impl CssCursorImages {
    /// Builds an image list with its fallback keyword.
    ///
    /// Returns `None` when `images` is empty.
    #[must_use]
    pub fn try_new(images: Vec<CssCursorImage>, fallback: CssCursorKeyword) -> Option<Self> {
        if images.is_empty() {
            None
        } else {
            Some(Self { images, fallback })
        }
    }

    /// Returns the image alternatives in authored order; the list is never empty.
    #[must_use]
    pub fn images(&self) -> &[CssCursorImage] {
        &self.images
    }

    /// Returns the keyword that applies when no image alternative can be used.
    #[must_use]
    pub const fn fallback(&self) -> CssCursorKeyword {
        self.fallback
    }
}

/// One image alternative and an optional complete pair of authored Number coordinates.
/// Omission uses resource metadata downstream and is distinct from explicit `0 0`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCursorImage {
    source: CssCursorImageSource,
    hotspot: Option<[CssNumeric<CssNumber>; 2]>,
}

impl CssCursorImage {
    /// Builds an image alternative from its source and optional hotspot.
    ///
    /// A `None` hotspot means the coordinates were omitted and come from the resource downstream; it is distinct from an explicit `0 0` pair.
    #[must_use]
    pub const fn new(
        source: CssCursorImageSource,
        hotspot: Option<[CssNumeric<CssNumber>; 2]>,
    ) -> Self {
        Self { source, hotspot }
    }

    /// Returns the image source, either a URL or an `image-set()` of URLs.
    #[must_use]
    pub const fn source(&self) -> &CssCursorImageSource {
        &self.source
    }

    /// Returns the authored hotspot coordinates (x, then y) as unclamped numbers, or `None` when they were omitted.
    #[must_use]
    pub const fn hotspot(&self) -> Option<&[CssNumeric<CssNumber>; 2]> {
        self.hotspot.as_ref()
    }
}

/// The selected UI 4 source domain excludes generated images and nested image sets.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursorImageSource {
    /// A single `<url>` reference.
    Url(CssUrl),
    /// An `image-set()` of URL candidates.
    UrlSet(CssCursorUrlSet),
}

/// An authored, nonempty `image-set()` of URL references only.
/// Candidate filtering, MIME support and duplicate-resolution selection are downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCursorUrlSet {
    options: Vec<CssCursorUrlSetOption>,
}

impl CssCursorUrlSet {
    /// Builds an `image-set()` from its candidates.
    ///
    /// Returns `None` when `options` is empty.
    #[must_use]
    pub fn try_new(options: Vec<CssCursorUrlSetOption>) -> Option<Self> {
        if options.is_empty() {
            None
        } else {
            Some(Self { options })
        }
    }

    /// Returns the candidates in authored order; the list is never empty.
    #[must_use]
    pub fn options(&self) -> &[CssCursorUrlSetOption] {
        &self.options
    }
}

/// A URL candidate with at most one resolution and one type, in authored order.
/// An absent resolution is retained: its natural-resolution semantics differ from
/// explicitly overriding resource metadata with `1x`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCursorUrlSetOption {
    reference: CssCursorUrlSetReference,
    descriptors: Vec<CssCursorUrlSetDescriptor>,
}

impl CssCursorUrlSetOption {
    /// Rejects repeated descriptor kinds, without evaluating MIME strings or resolutions.
    #[must_use]
    pub fn try_new(
        reference: CssCursorUrlSetReference,
        descriptors: Vec<CssCursorUrlSetDescriptor>,
    ) -> Option<Self> {
        if descriptors.len() > 2 {
            return None;
        }
        let mut resolution = false;
        let mut image_type = false;
        for descriptor in &descriptors {
            let seen = match descriptor {
                CssCursorUrlSetDescriptor::Resolution(_) => &mut resolution,
                CssCursorUrlSetDescriptor::Type(_) => &mut image_type,
            };
            if *seen {
                return None;
            }
            *seen = true;
        }
        Some(Self {
            reference,
            descriptors,
        })
    }

    /// Returns the candidate's URL or string reference.
    #[must_use]
    pub const fn reference(&self) -> &CssCursorUrlSetReference {
        &self.reference
    }

    /// Returns the original descriptor order; specified output uses grammar order.
    #[must_use]
    pub fn descriptors(&self) -> &[CssCursorUrlSetDescriptor] {
        &self.descriptors
    }

    /// Returns the candidate's non-negative resolution descriptor, or `None` when none was authored.
    #[must_use]
    pub fn resolution(&self) -> Option<&CssNumeric<CssResolution, CssNonNegative>> {
        self.descriptors
            .iter()
            .find_map(|descriptor| match descriptor {
                CssCursorUrlSetDescriptor::Resolution(value) => Some(value),
                CssCursorUrlSetDescriptor::Type(_) => None,
            })
    }

    /// Returns the candidate's `type()` string, or `None` when none was authored.
    #[must_use]
    pub fn image_type(&self) -> Option<&CssContentString> {
        self.descriptors
            .iter()
            .find_map(|descriptor| match descriptor {
                CssCursorUrlSetDescriptor::Type(value) => Some(value),
                CssCursorUrlSetDescriptor::Resolution(_) => None,
            })
    }
}

/// The Images 4 string arm represents a URL while retaining its authored string form.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursorUrlSetReference {
    /// A `url()` reference.
    Url(CssUrl),
    /// A string that stands for a URL, retaining its authored string form.
    String(CssContentString),
}

/// An intrinsic descriptor; arbitrary type strings remain valid authored syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursorUrlSetDescriptor {
    /// A non-negative `<resolution>` descriptor.
    Resolution(CssNumeric<CssResolution, CssNonNegative>),
    /// A `type()` descriptor holding the authored MIME-type string, which is not validated.
    Type(CssContentString),
}

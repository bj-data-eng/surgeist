//! Checked authored UI 4 cursor images, before resource selection or hotspot clamping.

use crate::{CssContentString, CssCursorKeyword, CssResolutionValue, CssSpecifiedNumber, CssUrl};

/// An authored cursor keyword or ordered image alternatives with a required fallback.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursor {
    Keyword(CssCursorKeyword),
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
    #[must_use]
    pub fn try_new(images: Vec<CssCursorImage>, fallback: CssCursorKeyword) -> Option<Self> {
        if images.is_empty() {
            None
        } else {
            Some(Self { images, fallback })
        }
    }

    #[must_use]
    pub fn images(&self) -> &[CssCursorImage] {
        &self.images
    }

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
    hotspot: Option<[CssSpecifiedNumber; 2]>,
}

impl CssCursorImage {
    #[must_use]
    pub const fn new(
        source: CssCursorImageSource,
        hotspot: Option<[CssSpecifiedNumber; 2]>,
    ) -> Self {
        Self { source, hotspot }
    }

    #[must_use]
    pub const fn source(&self) -> &CssCursorImageSource {
        &self.source
    }

    #[must_use]
    pub const fn hotspot(&self) -> Option<&[CssSpecifiedNumber; 2]> {
        self.hotspot.as_ref()
    }
}

/// The selected UI 4 source domain excludes generated images and nested image sets.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursorImageSource {
    Url(CssUrl),
    UrlSet(CssCursorUrlSet),
}

/// An authored, nonempty `image-set()` of URL references only.
/// Candidate filtering, MIME support and duplicate-resolution selection are downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCursorUrlSet {
    options: Vec<CssCursorUrlSetOption>,
}

impl CssCursorUrlSet {
    #[must_use]
    pub fn try_new(options: Vec<CssCursorUrlSetOption>) -> Option<Self> {
        if options.is_empty() {
            None
        } else {
            Some(Self { options })
        }
    }

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

    #[must_use]
    pub const fn reference(&self) -> &CssCursorUrlSetReference {
        &self.reference
    }

    /// Returns the original descriptor order; specified output uses grammar order.
    #[must_use]
    pub fn descriptors(&self) -> &[CssCursorUrlSetDescriptor] {
        &self.descriptors
    }

    #[must_use]
    pub fn resolution(&self) -> Option<&CssResolutionValue> {
        self.descriptors
            .iter()
            .find_map(|descriptor| match descriptor {
                CssCursorUrlSetDescriptor::Resolution(value) => Some(value),
                CssCursorUrlSetDescriptor::Type(_) => None,
            })
    }

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
    Url(CssUrl),
    String(CssContentString),
}

/// An intrinsic descriptor; arbitrary type strings remain valid authored syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCursorUrlSetDescriptor {
    Resolution(CssResolutionValue),
    Type(CssContentString),
}

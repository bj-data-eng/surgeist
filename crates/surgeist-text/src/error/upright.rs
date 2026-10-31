//! Read-only provenance and precise capability failures for upright operations.

use crate::Range;
use icu_properties::{PropertyNamesShort, props::Script};

/// Canonical Unicode ISO15924 script observed during text classification.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextScript {
    iso: [u8; 4],
}

impl TextScript {
    pub(crate) fn from_script(script: Script) -> Option<Self> {
        let name = PropertyNamesShort::<Script>::new().get(script)?;
        Some(Self {
            iso: name.as_bytes().try_into().ok()?,
        })
    }

    /// Returns the canonical four-byte Unicode script label.
    #[must_use]
    pub const fn iso15924(self) -> [u8; 4] {
        self.iso
    }
}

/// Actual captured style owner governing an upright operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextStyleOwner {
    /// Original source correspondence of the governing style.
    Source { range: Range },
    /// Original control occurrence.
    Control { occurrence: usize },
    /// Captured scope ordinal.
    Scope { scope: usize },
    /// Actual paragraph base/default-style strut.
    Paragraph,
    /// Captured nearest-block style used for the tab reference.
    TabReference,
}

/// Actual content or source-free hypothetical operation that failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UprightContent {
    /// Ordered original and indication content in the actual presentation.
    Presentation(UprightPresentation),
    /// Hypothetical tab metric operation under its actual captured style.
    TabMeasurement {
        role: UprightTabMeasurement,
        owner: TextStyleOwner,
    },
}

/// Nonempty ordered flat provenance of actual upright presentation content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UprightPresentation {
    pieces: Vec<UprightContentPiece>,
}

impl UprightPresentation {
    pub(crate) fn new(pieces: Vec<UprightContentPiece>) -> Self {
        assert!(
            !pieces.is_empty(),
            "upright presentation has actual content"
        );
        Self { pieces }
    }

    /// Returns actual pieces in content order, without an invented source hull.
    #[must_use]
    pub fn pieces(&self) -> &[UprightContentPiece] {
        &self.pieces
    }
}

/// One original-source or alternative-local piece of actual presentation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UprightContentPiece {
    /// Actual original UTF-16 source correspondence.
    Source { range: Range },
    /// Actual indication extent and its original-source insertion anchor.
    Indication {
        alternative: usize,
        range: Range,
        insertion_anchor: usize,
    },
}

/// Purpose fixing a hypothetical scalar used by tab preparation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UprightTabMeasurement {
    /// U+0020 under the captured nearest-block tab-reference style/direction.
    SpaceInterval,
    /// U+0030 under the needed analyzed tab-style row and LTR measurement premise.
    ZeroThreshold,
}

/// Actual governing style and its complete original optional locale tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UprightScriptContext {
    owner: TextStyleOwner,
    locale: Option<String>,
}

impl UprightScriptContext {
    pub(crate) fn new(owner: TextStyleOwner, locale: Option<String>) -> Self {
        Self { owner, locale }
    }

    /// Returns the actual governing style owner.
    #[must_use]
    pub const fn owner(&self) -> TextStyleOwner {
        self.owner
    }

    /// Returns the original complete locale tag, without inferred subtags.
    #[must_use]
    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }
}

/// Why effective script candidates cannot use the unrotated upright profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UprightScriptFailure {
    /// All candidates are intrinsic, or the resolver selected an intrinsic member.
    Intrinsic { resolved: Option<TextScript> },
    /// Mixed candidates lack one coherent explicitly qualified script.
    UnresolvedContext { reason: UprightScriptContextFailure },
}

/// Exact failure of the adopted explicit single-script source resolver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UprightScriptContextFailure {
    /// An actual governing owner has no explicit script subtag.
    MissingExplicitScript,
    /// An explicit subtag does not name a qualified single Unicode script.
    UnqualifiedSingleScript,
    /// A qualified explicit script is absent from the effective candidates.
    NonMember,
    /// Individually qualifying governing owners specify different scripts.
    ConflictingScripts,
}

/// Enabled decoration roles requiring unavailable automatic upright metrics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UprightDecorationRole {
    /// Only the offset needs an automatic upright metric.
    Offset,
    /// Only the thickness needs an automatic upright metric.
    Thickness,
    /// Both offset and thickness need automatic upright metrics.
    OffsetAndThickness,
}

/// Actual selected provider context unsupported by supplied vertical positioning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalShapingContextReason {
    /// Actual input direction is not top-to-bottom.
    Direction,
    /// Actual plan and input directions differ.
    PlanDirection,
    /// Actual plan and input scripts differ.
    PlanScript,
    /// The selected plan uses native morx substitution.
    Morx,
    /// The selected plan uses native kerx positioning.
    Kerx,
    /// The selected plan uses native kern positioning.
    Kern,
    /// The selected plan uses native tracking.
    Trak,
}

/// Why an actual unscaled origin reference cannot enter the provider input grid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalOriginReferenceReason {
    /// The actual reference is NaN or infinite.
    NonFinite,
    /// Its doubled value exceeds the signed-i32 input range.
    OutOfRange,
    /// A finite in-range value requires a finer grid than half design units.
    NotHalfUnit,
}

/// Actual positioning owner that exceeded the half-design-unit coordinate range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalPositionStage {
    /// Encoding actual supplied nominal metrics.
    Nominal,
    /// Applying OpenType values, anchors or mark-width adjustment.
    Gpos,
    /// Propagating actual attachment offsets and advances.
    Attachment,
    /// Computing fallback space or mark geometry.
    Fallback,
}

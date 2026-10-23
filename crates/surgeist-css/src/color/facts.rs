//! Concrete immutable facts for reached standalone color dependencies.
//!
//! Creating this borrowed envelope does not scan or copy arrays. Operation
//! actions validate individual headers/units and perform influencing lookups.

use crate::{
    ColorSpace, CssColorProfileComponentName, CssColorProfileName, CssColorProfileRenderingIntent,
    CssSystemColor,
};
use surgeist_color::Color;

/// Immutable fact-set identity and revision supplied by its actual owner.
///
/// The CSS operation compares these values. Its consumer remains responsible
/// for joining them to a real capture/resource revision; equal colors are not
/// a substitute for that qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorFactIdentity {
    identity: u64,
    revision: u64,
}
impl CssColorFactIdentity {
    /// Retains the caller's actual immutable identity and revision.
    #[must_use]
    pub const fn new(identity: u64, revision: u64) -> Self {
        Self { identity, revision }
    }
    /// The supplied identity, without interpreting it as a DOM handle.
    #[must_use]
    pub const fn identity(self) -> u64 {
        self.identity
    }
    /// The supplied revision; CSS never increments it.
    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }
}

/// Current-color interpretation uses initial properties or an actual resolved value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssColorPropertyFacts {
    /// With no context element, the initial current color is CanvasText.
    InitialProperties,
    /// The consumer supplies its actual resolved property value.
    Current(Color),
    /// The required property facts have not been supplied.
    Unavailable,
}

/// The effective source-selected scheme, not inferred from `normal`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorSchemeFact {
    Light,
    Dark,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchemeError {
    ForeignFacts,
    Unavailable,
}

/// A concrete system color entry and its resource qualification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssColorPaletteEntry {
    pub(crate) keyword: CssSystemColor,
    pub(crate) resource: CssColorFactIdentity,
    pub(crate) value: Color,
}
impl CssColorPaletteEntry {
    /// Constructs one fixed checked entry; duplicate-key admission belongs to the operation.
    #[must_use]
    pub const fn new(
        keyword: CssSystemColor,
        resource: CssColorFactIdentity,
        value: Color,
    ) -> Self {
        Self {
            keyword,
            resource,
            value,
        }
    }
    /// The retained canonical or deprecated source keyword.
    #[must_use]
    pub const fn keyword(self) -> CssSystemColor {
        self.keyword
    }
    /// The actual palette resource identity/revision.
    #[must_use]
    pub const fn resource(self) -> CssColorFactIdentity {
        self.resource
    }
    /// Its already resolved finite color.
    #[must_use]
    pub const fn value(self) -> Color {
        self.value
    }
}

/// Known profile availability differs from facts that have not been supplied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorProfileAvailability {
    Unknown,
    Absent,
    NotLoaded,
    Invalid,
    Loaded,
}

/// An immutable profile binding, retaining ordered exact component names.
///
/// Repeated component names remain allowed. CSS's lookup uses the first exact
/// decoded name as selected by the profile grammar, not a case-folded registry.
#[derive(Clone, Copy, Debug)]
pub struct CssColorProfileBinding<'a> {
    pub(crate) name: &'a CssColorProfileName,
    pub(crate) resource: CssColorFactIdentity,
    pub(crate) availability: CssColorProfileAvailability,
    pub(crate) intent: CssColorProfileRenderingIntent,
    pub(crate) parameter_count: usize,
    pub(crate) components: &'a [CssColorProfileComponentName],
}
impl<'a> CssColorProfileBinding<'a> {
    /// Borrows metadata without validating/scanning its whole component array.
    #[must_use]
    pub const fn new(
        name: &'a CssColorProfileName,
        resource: CssColorFactIdentity,
        availability: CssColorProfileAvailability,
        intent: CssColorProfileRenderingIntent,
        parameter_count: usize,
        components: &'a [CssColorProfileComponentName],
    ) -> Self {
        Self {
            name,
            resource,
            availability,
            intent,
            parameter_count,
            components,
        }
    }
    /// The actual profile resource and version.
    #[must_use]
    pub const fn resource(self) -> CssColorFactIdentity {
        self.resource
    }
    /// The explicit availability observation.
    #[must_use]
    pub const fn availability(self) -> CssColorProfileAvailability {
        self.availability
    }
    /// Actual input parameter count from loaded profile metadata; independent
    /// of the optional descriptor's ordered channel names.
    #[must_use]
    pub const fn parameter_count(self) -> usize {
        self.parameter_count
    }
}

/// A profile conversion capability or concrete applicable conversion result.
#[derive(Clone, Copy, Debug)]
pub enum CssColorProfileConversion<'a> {
    /// The selected conversion operation is not implemented for this actual profile.
    Unsupported { profile: CssColorFactIdentity },
    /// The required conversion facts are absent.
    Unavailable { profile: CssColorFactIdentity },
    /// One concrete conversion, joined to exact profile/input/intent/target facts.
    Resolved(CssColorProfileConversionRecord<'a>),
}

/// The complete fixed header and borrowed input channels of one conversion.
///
/// Inputs are checked one at a time by admitted actions, including finite
/// validation and exact input equality. Constructing this envelope does not
/// claim that an arbitrary record applies to the requested color.
#[derive(Clone, Copy, Debug)]
pub struct CssColorProfileConversionRecord<'a> {
    pub(crate) profile: CssColorFactIdentity,
    pub(crate) channels: &'a [f64],
    pub(crate) intent: CssColorProfileRenderingIntent,
    pub(crate) target: ColorSpace,
    pub(crate) value: Color,
}
impl<'a> CssColorProfileConversionRecord<'a> {
    /// Borrows one actual conversion observation, without scanning channels.
    #[must_use]
    pub const fn new(
        profile: CssColorFactIdentity,
        channels: &'a [f64],
        intent: CssColorProfileRenderingIntent,
        target: ColorSpace,
        value: Color,
    ) -> Self {
        Self {
            profile,
            channels,
            intent,
            target,
            value,
        }
    }
}

/// Device processing knowledge; Unknown does not select a known fallback.
#[derive(Clone, Copy, Debug)]
pub enum CssDeviceColorFact<'a> {
    Unknown,
    Absent,
    Unsupported,
    Invalid,
    WrongSpace,
    /// The operation checks input channels/profile/intent and Lab result before use.
    Resolved(CssColorProfileConversionRecord<'a>),
}

/// Borrowed context data; reached validation/lookup is driven by the same action machine.
#[derive(Clone, Copy, Debug)]
pub struct CssColorFacts<'a> {
    pub(crate) identity: CssColorFactIdentity,
    pub(crate) properties: CssColorPropertyFacts,
    pub(crate) scheme: CssColorSchemeFact,
    pub(crate) palette: &'a [CssColorPaletteEntry],
    pub(crate) profiles: &'a [CssColorProfileBinding<'a>],
    pub(crate) conversions: &'a [CssColorProfileConversion<'a>],
    pub(crate) device: CssDeviceColorFact<'a>,
}
impl<'a> CssColorFacts<'a> {
    /// Borrows explicit facts without allocating, scanning, guessing palette entries or resolving
    /// them.
    #[must_use]
    pub const fn new(
        identity: CssColorFactIdentity,
        properties: CssColorPropertyFacts,
        scheme: CssColorSchemeFact,
        palette: &'a [CssColorPaletteEntry],
        profiles: &'a [CssColorProfileBinding<'a>],
        conversions: &'a [CssColorProfileConversion<'a>],
        device: CssDeviceColorFact<'a>,
    ) -> Self {
        Self {
            identity,
            properties,
            scheme,
            palette,
            profiles,
            conversions,
            device,
        }
    }
    /// The exact caller-supplied immutable fact-set qualification.
    #[must_use]
    pub const fn identity(self) -> CssColorFactIdentity {
        self.identity
    }
    /// One admitted fixed fact observation selects the influencing light-dark
    /// child; it does not inspect either child's property/resource facts.
    pub(crate) fn selected_scheme(
        &self,
        expected: CssColorFactIdentity,
    ) -> Result<usize, SchemeError> {
        if self.identity != expected {
            return Err(SchemeError::ForeignFacts);
        }
        match self.scheme {
            CssColorSchemeFact::Light => Ok(0),
            CssColorSchemeFact::Dark => Ok(1),
            CssColorSchemeFact::Unavailable => Err(SchemeError::Unavailable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_scheme_does_not_guess_normal_or_inspect_unreached_properties() {
        let id = CssColorFactIdentity::new(4, 5);
        for (scheme, expected) in [
            (CssColorSchemeFact::Light, Ok(0)),
            (CssColorSchemeFact::Dark, Ok(1)),
            (
                CssColorSchemeFact::Unavailable,
                Err(SchemeError::Unavailable),
            ),
        ] {
            let facts = CssColorFacts::new(
                id,
                CssColorPropertyFacts::Unavailable,
                scheme,
                &[],
                &[],
                &[],
                CssDeviceColorFact::Unknown,
            );
            assert_eq!(facts.selected_scheme(id), expected);
            assert_eq!(
                facts.selected_scheme(CssColorFactIdentity::new(4, 6)),
                Err(SchemeError::ForeignFacts)
            );
        }
    }
}

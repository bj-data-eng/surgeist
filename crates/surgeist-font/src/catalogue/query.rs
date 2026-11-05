use std::{collections::HashMap, ops::ControlFlow, sync::Arc};

use fontique::{FamilyId, Query, QueryFont, QueryStatus};
use parlance::{FontStyle, FontVariation, FontWeight, FontWidth, Tag};

use super::{CatalogueError, CataloguePublication};
use crate::{FontFace, FontResource};

/// One family resolved in a specific catalogue publication.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FontFamily {
    id: FamilyId,
    publication: CataloguePublication,
}

impl FontFamily {
    pub(super) fn new(id: FamilyId, publication: CataloguePublication) -> Self {
        Self { id, publication }
    }

    pub(super) fn validate(
        &self,
        publication: &CataloguePublication,
    ) -> Result<FamilyId, CatalogueError> {
        if !Arc::ptr_eq(&self.publication.owner, &publication.owner) {
            return Err(CatalogueError::ForeignFamily);
        }
        if self.publication.generation != publication.generation {
            return Err(CatalogueError::StaleFamily);
        }
        Ok(self.id)
    }

    /// Publication that resolved this family.
    #[must_use]
    pub const fn publication(&self) -> &CataloguePublication {
        &self.publication
    }
}

/// Width, style and weight requested for provider matching.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontAttributes {
    width: FontWidth,
    style: FontStyle,
    weight: FontWeight,
}

impl FontAttributes {
    /// Creates a matching request with the given attributes.
    #[must_use]
    pub const fn new(width: FontWidth, style: FontStyle, weight: FontWeight) -> Self {
        Self {
            width,
            style,
            weight,
        }
    }

    /// Style requested by the caller.
    #[must_use]
    pub const fn style(self) -> FontStyle {
        self.style
    }

    /// Weight requested by the caller.
    #[must_use]
    pub const fn weight(self) -> FontWeight {
        self.weight
    }
}

/// A candidate from a registered font query with its actual face and suggestions.
#[derive(Clone, Debug)]
pub struct FontCandidate {
    face: FontFace,
    family: FontFamily,
    charmap: fontique::CharmapIndex,
    synthesis: FontSynthesisSuggestions,
}

impl FontCandidate {
    /// Actual face selected by the provider from the registered resource.
    #[must_use]
    pub const fn face(&self) -> &FontFace {
        &self.face
    }

    /// Family that yielded this candidate.
    #[must_use]
    pub const fn family(&self) -> &FontFamily {
        &self.family
    }

    /// Publication in which the provider selected this candidate.
    #[must_use]
    pub const fn publication(&self) -> &CataloguePublication {
        self.family.publication()
    }

    /// Provider suggestions for matching the requested attributes.
    #[must_use]
    pub const fn synthesis(&self) -> &FontSynthesisSuggestions {
        &self.synthesis
    }

    /// Borrows the provider's nominal character map for this exact face.
    #[must_use]
    pub fn charmap(&self) -> Option<NominalCharmap<'_>> {
        self.charmap
            .charmap(self.face.resource().bytes())
            .map(NominalCharmap)
    }
}

/// Provider suggestions for variation settings and faux styling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontSynthesisSuggestions(fontique::Synthesis);

impl FontSynthesisSuggestions {
    /// Suggested variation settings in provider order.
    pub fn variation_settings(&self) -> impl ExactSizeIterator<Item = FontVariation> + '_ {
        self.0
            .variation_settings()
            .iter()
            .map(|&(tag, value)| FontVariation::new(Tag::from_bytes(tag.to_be_bytes()), value))
    }

    /// Whether the provider suggests faux bold.
    #[must_use]
    pub fn embolden(&self) -> bool {
        self.0.embolden()
    }

    /// Suggested faux italic or oblique angle in degrees.
    #[must_use]
    pub fn skew(&self) -> Option<f32> {
        self.0.skew()
    }
}

/// Nominal glyph mapping using the selected provider subtable and encoding.
pub struct NominalCharmap<'a>(fontique::Charmap<'a>);

impl NominalCharmap<'_> {
    /// Looks up the nominal glyph ID, preserving glyph zero when the map returns it.
    #[must_use]
    pub fn glyph_id(&self, codepoint: char) -> Option<u32> {
        self.0.map(codepoint)
    }
}

/// Reusable matching state borrowing one registered catalogue publication.
pub struct FontQuery<'a> {
    inner: Query<'a>,
    resources: &'a HashMap<u64, FontResource>,
    publication: &'a CataloguePublication,
}

impl<'a> FontQuery<'a> {
    pub(super) fn new(
        inner: Query<'a>,
        resources: &'a HashMap<u64, FontResource>,
        publication: &'a CataloguePublication,
    ) -> Self {
        Self {
            inner,
            resources,
            publication,
        }
    }

    /// Visits the provider's candidates for these ordered families.
    ///
    /// All family handles must belong to the borrowed publication. Families may
    /// repeat. Returning `Break` stops matching and returns the supplied value.
    /// Normal exhaustion, including an empty family list, returns `Continue(())`.
    pub fn visit_candidates<B>(
        &mut self,
        families: &[FontFamily],
        attributes: FontAttributes,
        mut visit: impl FnMut(&FontCandidate) -> ControlFlow<B>,
    ) -> Result<ControlFlow<B>, CatalogueError> {
        for family in families {
            family.validate(self.publication)?;
        }
        self.inner
            .set_families(families.iter().map(|family| family.id));
        self.inner.set_attributes(fontique::Attributes::new(
            attributes.width,
            attributes.style,
            attributes.weight,
        ));

        let resources = self.resources;
        let publication = self.publication;
        let mut outcome = Ok(ControlFlow::Continue(()));
        self.inner.matches_with(|font| {
            let candidate = match candidate(resources, publication, font) {
                Ok(candidate) => candidate,
                Err(error) => {
                    outcome = Err(error);
                    return QueryStatus::Stop;
                }
            };
            match visit(&candidate) {
                ControlFlow::Continue(()) => QueryStatus::Continue,
                ControlFlow::Break(value) => {
                    outcome = Ok(ControlFlow::Break(value));
                    QueryStatus::Stop
                }
            }
        });
        outcome
    }
}

fn candidate(
    resources: &HashMap<u64, FontResource>,
    publication: &CataloguePublication,
    font: &QueryFont,
) -> Result<FontCandidate, CatalogueError> {
    let resource = resources
        .get(&font.blob.id())
        .filter(|resource| std::ptr::eq(resource.bytes(), font.blob.as_ref()))
        .ok_or(CatalogueError::ProviderResourceConflict)?;
    let face = resource
        .face(font.index)
        .ok_or(CatalogueError::ProviderResourceConflict)?;
    Ok(FontCandidate {
        face,
        family: FontFamily::new(font.family.0, publication.clone()),
        charmap: font.charmap_index,
        synthesis: FontSynthesisSuggestions(font.synthesis),
    })
}

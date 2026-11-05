use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use fontique::{Collection, CollectionOptions, SourceCache, SourceKind};
use linebender_resource_handle::Blob;

use crate::{FontFace, FontResource};

mod query;

pub use query::{
    FontAttributes, FontCandidate, FontFamily, FontQuery, FontSynthesisSuggestions, NominalCharmap,
};

/// Identity of one successful catalogue publication.
///
/// Clones retain the issuer. Two catalogues at the same numeric generation
/// still have distinct publications.
#[derive(Clone)]
pub struct CataloguePublication {
    owner: Arc<()>,
    generation: u64,
}

impl CataloguePublication {
    fn initial() -> Self {
        Self {
            owner: Arc::new(()),
            generation: 0,
        }
    }

    /// Numeric generation within this publication's issuer.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    fn successor(&self) -> Result<Self, CatalogueError> {
        let generation =
            self.generation
                .checked_add(1)
                .ok_or(CatalogueError::GenerationExhausted {
                    generation: self.generation,
                })?;
        Ok(Self {
            owner: Arc::clone(&self.owner),
            generation,
        })
    }
}

impl PartialEq for CataloguePublication {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner) && self.generation == other.generation
    }
}

impl Eq for CataloguePublication {}

impl Hash for CataloguePublication {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.owner).hash(state);
        self.generation.hash(state);
    }
}

impl fmt::Debug for CataloguePublication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CataloguePublication")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

/// A catalogue operation failed and left the current publication intact.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CatalogueError {
    /// The issuer has no remaining generation to publish.
    GenerationExhausted { generation: u64 },
    /// Provider face records did not cover each original index exactly once.
    FacesNotAdmitted { expected: u32, admitted: usize },
    /// A family handle belongs to a different catalogue issuer.
    ForeignFamily,
    /// A family handle was issued before the current publication.
    StaleFamily,
    /// A provider result could not be associated with its exact registered face.
    ProviderResourceConflict,
}

impl fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenerationExhausted { generation } => {
                write!(f, "font catalogue generation {generation} is exhausted")
            }
            Self::FacesNotAdmitted { expected, admitted } => write!(
                f,
                "font provider admitted {admitted} face records without covering original indices 0..{expected} exactly once"
            ),
            Self::ForeignFamily => f.write_str("font family belongs to a different catalogue"),
            Self::StaleFamily => f.write_str("font family belongs to a preceding publication"),
            Self::ProviderResourceConflict => {
                f.write_str("font provider resource association conflicts")
            }
        }
    }
}

impl std::error::Error for CatalogueError {}

/// Publishes an ordered catalogue of explicitly registered font resources.
pub struct FontCatalogue {
    publication: CataloguePublication,
    provider: RegisteredProvider,
    registrations: Vec<FontResource>,
}

impl FontCatalogue {
    /// Creates an empty catalogue restricted to explicitly registered resources.
    #[must_use]
    pub fn registered_only() -> Self {
        Self {
            publication: CataloguePublication::initial(),
            provider: RegisteredProvider::empty(),
            registrations: Vec::new(),
        }
    }

    /// Current registration publication.
    #[must_use]
    pub const fn publication(&self) -> &CataloguePublication {
        &self.publication
    }

    /// Appends an admitted resource after the provider accepts its complete face set.
    ///
    /// A successful registration retains the resource and its original face
    /// order. Failure preserves the current publication and registrations.
    pub fn register(
        &mut self,
        resource: FontResource,
    ) -> Result<CataloguePublication, CatalogueError> {
        let publication = self.publication.successor()?;
        let mut provider = self.provider.clone();
        provider.register(&resource)?;
        self.registrations.push(resource);
        self.provider = provider;
        self.publication = publication.clone();
        Ok(publication)
    }

    /// Enumerates strong faces in registration order, then original file order.
    pub fn registered_faces(&self) -> impl Iterator<Item = FontFace> + '_ {
        self.registrations.iter().flat_map(|resource| {
            (0..resource.face_count()).map(|index| {
                resource
                    .face(index)
                    .expect("an admitted resource retains every original face index")
            })
        })
    }

    /// Enumerates registered family names in provider order.
    pub fn family_names(&mut self) -> impl Iterator<Item = &str> + '_ {
        self.provider.collection.family_names()
    }

    /// Resolves a registered family name using provider name matching.
    pub fn family_by_name(&mut self, name: &str) -> Option<FontFamily> {
        let id = self.provider.collection.family_id(name)?;
        Some(FontFamily::new(id, self.publication.clone()))
    }

    /// Borrows this publication for one or more registered candidate queries.
    pub fn query(&mut self) -> FontQuery<'_> {
        FontQuery::new(
            self.provider
                .collection
                .query(&mut self.provider.source_cache),
            &self.provider.resources,
            &self.publication,
        )
    }

    /// Replays every registered resource into a fresh provider collection.
    ///
    /// The successful publication keeps the original resource identities and
    /// registration order. A replay failure preserves the current publication.
    pub fn refresh(&mut self) -> Result<CataloguePublication, CatalogueError> {
        let publication = self.publication.successor()?;
        let mut provider = RegisteredProvider::empty();
        for resource in &self.registrations {
            provider.register(resource)?;
        }
        self.provider = provider;
        self.publication = publication.clone();
        Ok(publication)
    }
}

#[derive(Clone)]
struct RegisteredProvider {
    collection: Collection,
    source_cache: SourceCache,
    resources: HashMap<u64, FontResource>,
}

impl RegisteredProvider {
    fn empty() -> Self {
        Self {
            collection: Collection::new(CollectionOptions {
                shared: false,
                system_fonts: false,
            }),
            source_cache: SourceCache::default(),
            resources: HashMap::new(),
        }
    }

    fn register(&mut self, resource: &FontResource) -> Result<(), CatalogueError> {
        // Provider scans borrow the already admitted bytes through this Blob.
        let data = Blob::new(Arc::new(ResourceBytes(resource.clone())));
        let id = data.id();
        let Entry::Vacant(entry) = self.resources.entry(id) else {
            return Err(CatalogueError::ProviderResourceConflict);
        };
        let families = self.collection.register_fonts(data, None);
        let mut fonts = families.iter().flat_map(|(_, fonts)| fonts);
        let expected = resource.face_count();
        let mut admitted: Vec<_> = fonts.clone().map(fontique::FontInfo::index).collect();
        admitted.sort_unstable();
        if !admitted.iter().copied().eq(0..expected) {
            return Err(CatalogueError::FacesNotAdmitted {
                expected,
                admitted: admitted.len(),
            });
        }

        // Only the exact original memory Blob is eligible for later queries.
        if !fonts.all(|font| {
            matches!(font.source().kind(), SourceKind::Memory(blob)
                if blob.id() == id && std::ptr::eq(blob.as_ref(), resource.bytes()))
        }) {
            return Err(CatalogueError::ProviderResourceConflict);
        }
        entry.insert(resource.clone());
        Ok(())
    }
}

struct ResourceBytes(FontResource);

impl AsRef<[u8]> for ResourceBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DecodeLimits;

    #[test]
    fn exhausted_generation_refuses_registration_and_refresh() {
        let mut catalogue = FontCatalogue::registered_only();
        catalogue.publication.generation = u64::MAX;
        let before = catalogue.publication().clone();
        let limits = DecodeLimits::new(1024, 1024, 1024, 16, 4, 1024, 4096);
        let resource = FontResource::decode(b"\0\x01\0\0\0\0\0\0\0\0\0\0".to_vec(), limits)
            .expect("an empty table directory passes structural admission");

        let error = CatalogueError::GenerationExhausted {
            generation: u64::MAX,
        };
        assert_eq!(catalogue.register(resource), Err(error.clone()));
        assert_eq!(catalogue.refresh(), Err(error));
        assert_eq!(catalogue.publication(), &before);
        assert!(catalogue.registered_faces().next().is_none());
    }

    #[test]
    fn a_provider_candidate_cannot_rebind_to_equal_bytes_from_another_resource() {
        let bytes = include_bytes!("../../surgeist-render/tests/fixtures/fonts/ahem/Ahem.ttf");
        let capacity = 1 << 20;
        let limits = DecodeLimits::new(capacity, capacity, capacity, 100, 1, capacity, capacity);
        let original = FontResource::decode(bytes.to_vec(), limits).unwrap();
        let unrelated = FontResource::decode(bytes.to_vec(), limits).unwrap();
        let mut catalogue = FontCatalogue::registered_only();
        catalogue.register(original).unwrap();
        let family = catalogue.family_by_name("Ahem").unwrap();
        let slot = catalogue.provider.resources.values_mut().next().unwrap();
        *slot = unrelated;

        let outcome =
            catalogue
                .query()
                .visit_candidates::<()>(&[family], FontAttributes::default(), |_| {
                    panic!("unrelated backing must be rejected before the callback")
                });
        assert_eq!(outcome, Err(CatalogueError::ProviderResourceConflict));
    }
}

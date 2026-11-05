use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use fontique::{Collection, CollectionOptions, SourceId as ProviderSourceId};
use linebender_resource_handle::Blob;

use crate::{FontFace, FontResource};

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

/// A catalogue transition failed and left the preceding publication intact.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CatalogueError {
    /// The issuer has no remaining generation to publish.
    GenerationExhausted { generation: u64 },
    /// Provider face records did not cover each original index exactly once.
    FacesNotAdmitted { expected: u32, admitted: usize },
    /// Provider source identifiers could not unambiguously identify a registration.
    ProviderSourceConflict,
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
            Self::ProviderSourceConflict => {
                f.write_str("font provider source identifiers conflict")
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
    sources: HashMap<ProviderSourceId, FontResource>,
}

impl RegisteredProvider {
    fn empty() -> Self {
        Self {
            collection: Collection::new(CollectionOptions {
                shared: false,
                system_fonts: false,
            }),
            sources: HashMap::new(),
        }
    }

    fn register(&mut self, resource: &FontResource) -> Result<(), CatalogueError> {
        // Provider scans borrow the already admitted bytes through this Blob.
        let data = Blob::new(Arc::new(ResourceBytes(resource.clone())));
        let families = self.collection.register_fonts(data, None);
        let fonts = families.iter().flat_map(|(_, fonts)| fonts);
        let expected = resource.face_count();
        let mut admitted: Vec<_> = fonts.clone().map(fontique::FontInfo::index).collect();
        admitted.sort_unstable();
        if !admitted.iter().copied().eq(0..expected) {
            return Err(CatalogueError::FacesNotAdmitted {
                expected,
                admitted: admitted.len(),
            });
        }

        // Fontique issues one memory source for all faces of a registration.
        let mut sources = fonts.map(|font| font.source().id());
        let source = sources.next().expect("the admitted face set is nonempty");
        if sources.any(|other| other != source) {
            return Err(CatalogueError::ProviderSourceConflict);
        }
        let Entry::Vacant(entry) = self.sources.entry(source) else {
            return Err(CatalogueError::ProviderSourceConflict);
        };
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
}

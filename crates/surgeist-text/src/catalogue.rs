use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use fontique::{Collection, CollectionOptions, SourceCache};
use linebender_resource_handle::Blob;
use skrifa::raw::FileRef;

use crate::{Error, ErrorCode, ErrorDetail, FontFace, FontGeneration, Result};

/// Sources available to a text system's font catalogue.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FontCatalogueMode {
    /// Discover system fonts as well as fonts registered with the system.
    #[default]
    System,
    /// Match only registered fonts, without system discovery or fallback.
    RegisteredOnly,
}

/// Identity of one registration and discovery publication by a text system.
///
/// Equal generations from different systems are distinct. Clones retain the
/// original issuer and publication; this identity is not a font-content hash.
#[derive(Clone)]
pub struct FontCatalogueIdentity {
    owner: Arc<()>,
    generation: FontGeneration,
}

impl FontCatalogueIdentity {
    fn initial() -> Self {
        Self {
            owner: Arc::new(()),
            generation: FontGeneration::initial(),
        }
    }

    /// Returns this publication's numeric generation within its issuer.
    #[must_use]
    pub const fn generation(&self) -> FontGeneration {
        self.generation
    }

    fn successor(&self) -> Result<Self> {
        let generation = self.generation.checked_next().ok_or_else(|| {
            Error::new(
                ErrorCode::FontGenerationExhausted,
                "font catalogue generation is exhausted",
            )
            .with_detail(ErrorDetail::FontGenerationExhausted {
                generation: self.generation.get(),
            })
        })?;
        Ok(Self {
            owner: Arc::clone(&self.owner),
            generation,
        })
    }
}

impl PartialEq for FontCatalogueIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner) && self.generation == other.generation
    }
}

impl Eq for FontCatalogueIdentity {}

impl Hash for FontCatalogueIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.owner).hash(state);
        self.generation.hash(state);
    }
}

impl fmt::Debug for FontCatalogueIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontCatalogueIdentity")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
struct RegisteredFont {
    data: Blob<u8>,
    indices: Vec<u32>,
}

#[derive(Clone)]
pub(crate) struct Catalogue {
    mode: FontCatalogueMode,
    identity: FontCatalogueIdentity,
    collection: Collection,
    source_cache: SourceCache,
    registered: Vec<RegisteredFont>,
}

impl Catalogue {
    pub(crate) fn new(mode: FontCatalogueMode) -> Self {
        Self {
            mode,
            identity: FontCatalogueIdentity::initial(),
            collection: new_collection(mode),
            source_cache: SourceCache::default(),
            registered: Vec::new(),
        }
    }

    pub(crate) const fn identity(&self) -> &FontCatalogueIdentity {
        &self.identity
    }

    pub(crate) const fn collection_mut(&mut self) -> &mut Collection {
        &mut self.collection
    }

    pub(crate) fn query(&mut self) -> fontique::Query<'_> {
        self.source_cache.prune(128, false);
        self.collection.query(&mut self.source_cache)
    }

    pub(crate) fn registered_faces(&self) -> impl Iterator<Item = FontFace<'_>> {
        self.registered.iter().flat_map(|registration| {
            registration
                .indices
                .iter()
                .map(|&index| FontFace::new(&registration.data, index, &self.identity))
        })
    }

    pub(crate) fn with_registration(&self, bytes: Vec<u8>) -> Result<Self> {
        // Check issuance before parsing, allocating provider identities, or
        // touching even candidate font state.
        let identity = self.identity.successor()?;
        let indices = font_indices(&bytes)?;
        let registration = RegisteredFont {
            data: Blob::from(bytes),
            indices,
        };
        // All collections originate unshared: registration mutates this clone's
        // CommonData, never the published collection's registrations.
        let mut candidate = self.clone();
        admit_registration(&mut candidate.collection, &registration)?;
        candidate.registered.push(registration);
        candidate.identity = identity;
        Ok(candidate)
    }

    pub(crate) fn refreshed(&self) -> Result<Self> {
        let identity = self.identity.successor()?;
        let mut collection = new_collection(self.mode);
        for registration in &self.registered {
            admit_registration(&mut collection, registration)?;
        }
        Ok(Self {
            mode: self.mode,
            identity,
            collection,
            source_cache: SourceCache::default(),
            registered: self.registered.clone(),
        })
    }

    #[cfg(test)]
    pub(crate) fn exhaust_generation_for_test(&mut self) {
        self.identity.generation = FontGeneration::exhausted_for_test();
    }
}

fn new_collection(mode: FontCatalogueMode) -> Collection {
    Collection::new(CollectionOptions {
        shared: false,
        system_fonts: mode == FontCatalogueMode::System,
    })
}

fn font_indices(bytes: &[u8]) -> Result<Vec<u32>> {
    let file = FileRef::new(bytes).map_err(|_| invalid_container())?;
    let mut indices = Vec::new();
    for (index, font) in file.fonts().enumerate() {
        let index = u32::try_from(index).map_err(|_| invalid_container())?;
        font.map_err(|_| {
            Error::new(ErrorCode::FontLoadFailed, "invalid font face")
                .with_detail(ErrorDetail::InvalidFontFace { index })
        })?;
        indices.push(index);
    }
    if indices.is_empty() {
        return Err(invalid_container());
    }
    Ok(indices)
}

fn invalid_container() -> Error {
    Error::new(ErrorCode::FontLoadFailed, "invalid or empty font container")
        .with_detail(ErrorDetail::InvalidFontContainer)
}

fn admit_registration(collection: &mut Collection, registration: &RegisteredFont) -> Result<()> {
    let families = collection.register_fonts(registration.data.clone(), None);
    let mut admitted: Vec<_> = families
        .iter()
        .flat_map(|(_, fonts)| fonts.iter().map(fontique::FontInfo::index))
        .collect();
    admitted.sort_unstable();
    if admitted != registration.indices {
        return Err(Error::new(
            ErrorCode::FontLoadFailed,
            "not every font face was admitted",
        )
        .with_detail(ErrorDetail::FontFacesNotAdmitted {
            expected: registration.indices.len(),
            admitted: admitted.len(),
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_replays_fonts_into_a_fresh_provider_family_namespace() {
        let catalogue = Catalogue::new(FontCatalogueMode::RegisteredOnly);
        let bytes = include_bytes!("../tests/fixtures/fonts/ahem/Ahem.ttf");
        let mut registered = catalogue
            .with_registration(bytes.to_vec())
            .expect("Ahem is admitted");
        let old_id = registered
            .collection
            .family_id("Ahem")
            .expect("the actual registration issues a family identity");
        let old_family = registered
            .collection
            .family(old_id)
            .expect("the original provider resolves its issued family");

        let mut refreshed = registered.refreshed().expect("Ahem is replayed");
        let new_id = refreshed
            .collection
            .family_id("Ahem")
            .expect("the fresh provider admits the replayed family");
        assert_ne!(new_id, old_id);
        assert!(refreshed.collection.family(old_id).is_none());
        let new_family = refreshed
            .collection
            .family(new_id)
            .expect("the fresh provider resolves its own family identity");
        assert_eq!(new_family.name(), "Ahem");
        assert!(registered.collection.family(new_id).is_none());
        assert_eq!(
            registered
                .collection
                .family(old_id)
                .expect("refresh leaves the original provider intact")
                .id(),
            old_id
        );
        for family in [old_family, new_family] {
            let data = family
                .default_font()
                .expect("the actual family has an admitted face")
                .load(None)
                .expect("the family retains its registered font resource");
            assert_eq!(data.as_ref(), bytes);
        }
    }

    #[test]
    fn registered_only_excludes_system_families_before_and_after_refresh() {
        let mut catalogue = Catalogue::new(FontCatalogueMode::RegisteredOnly);
        assert_eq!(catalogue.collection.family_names().count(), 0);
        let bytes = include_bytes!("../tests/fixtures/fonts/ahem/Ahem.ttf");
        let mut registered = catalogue
            .with_registration(bytes.to_vec())
            .expect("Ahem is admitted");
        assert_eq!(
            registered.collection.family_names().collect::<Vec<_>>(),
            ["Ahem"]
        );
        let mut refreshed = registered.refreshed().expect("Ahem is replayed");
        assert_eq!(
            refreshed.collection.family_names().collect::<Vec<_>>(),
            ["Ahem"]
        );
        assert_eq!(catalogue.collection.family_names().count(), 0);
    }
}

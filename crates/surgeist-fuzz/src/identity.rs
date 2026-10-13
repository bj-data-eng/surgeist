#![forbid(unsafe_code)]

use crate::{FuzzErrorKind, Result, error::invalid};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

pub(crate) fn kebab(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
        && value.as_bytes()[0].is_ascii_lowercase()
}

pub(crate) fn owner(value: &str) -> bool {
    value.strip_prefix("surgeist-").is_some_and(kebab)
}

pub(crate) fn qualified(value: &str) -> bool {
    value
        .split_once("::")
        .is_some_and(|(prefix, name)| owner(prefix) && kebab(name))
}

macro_rules! qualified_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                if !qualified(&value) {
                    return Err(invalid(
                        FuzzErrorKind::InvalidIdentity,
                        "identity must be surgeist-owner::kebab-name",
                    ));
                }
                Ok(Self(value))
            }
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
            #[must_use]
            pub fn owner(&self) -> &str {
                self.0
                    .split_once("::")
                    .expect("validated qualified identity")
                    .0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(
                deserializer: D,
            ) -> std::result::Result<Self, D::Error> {
                Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

qualified_id!(
    TargetId,
    "Owner-qualified identity of an executable fuzz target."
);
qualified_id!(
    PropertyId,
    "Owner-qualified identity of a semantic property."
);

/// Identity of a persisted owner codec, including its positive protocol revision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawFormat")]
pub struct CaseFormat {
    identity: String,
    revision: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFormat {
    identity: String,
    revision: u32,
}

impl TryFrom<RawFormat> for CaseFormat {
    type Error = crate::FuzzError;
    fn try_from(raw: RawFormat) -> Result<Self> {
        Self::new(raw.identity, raw.revision)
    }
}

impl CaseFormat {
    pub fn new(identity: impl Into<String>, revision: u32) -> Result<Self> {
        let identity = identity.into();
        if !qualified(&identity) || revision == 0 {
            return Err(invalid(
                FuzzErrorKind::InvalidIdentity,
                "case format requires a qualified identity and positive revision",
            ));
        }
        Ok(Self { identity, revision })
    }
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }

    /// Rejects an unsupported persisted codec before interpreting its bytes.
    pub fn require(&self, supported: &Self) -> Result<()> {
        if self != supported {
            return Err(invalid(
                FuzzErrorKind::UnsupportedFormat,
                format!(
                    "unsupported case format {} revision {}",
                    self.identity, self.revision
                ),
            ));
        }
        Ok(())
    }
}

/// Versioned identity of the deterministic generation encoding, separate from replay.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawGenerator")]
pub struct GeneratorIdentity {
    identity: String,
    revision: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGenerator {
    identity: String,
    revision: u32,
}

impl TryFrom<RawGenerator> for GeneratorIdentity {
    type Error = crate::FuzzError;
    fn try_from(raw: RawGenerator) -> Result<Self> {
        Self::new(raw.identity, raw.revision)
    }
}

impl GeneratorIdentity {
    pub fn new(identity: impl Into<String>, revision: u32) -> Result<Self> {
        let identity = identity.into();
        if !qualified(&identity) || revision == 0 {
            return Err(invalid(
                FuzzErrorKind::InvalidIdentity,
                "generator requires a qualified identity and positive revision",
            ));
        }
        Ok(Self { identity, revision })
    }
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }
    pub fn require(&self, supported: &Self) -> Result<()> {
        if self != supported {
            return Err(invalid(
                FuzzErrorKind::UnsupportedFormat,
                "unsupported generator identity or revision",
            ));
        }
        Ok(())
    }
}

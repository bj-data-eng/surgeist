#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StateVersion(u64);

impl StateVersion {
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotBinding {
    pub name: String,
    pub source_type: &'static str,
}

impl SnapshotBinding {
    #[must_use]
    pub fn new(name: impl Into<String>, source_type: &'static str) -> Self {
        Self {
            name: name.into(),
            source_type,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppSnapshot {
    version: StateVersion,
    bindings: Vec<SnapshotBinding>,
}

impl AppSnapshot {
    #[must_use]
    pub fn new(version: StateVersion) -> Self {
        Self {
            version,
            bindings: Vec::new(),
        }
    }

    #[must_use]
    pub const fn version(&self) -> StateVersion {
        self.version
    }
}

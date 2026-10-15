//! Shared #child-navigables null relationship, without family load policy.

/// Proof of the actual initially-null relationship. Families leave this domain
/// before installing content; present or unknown retained content cannot enter it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NullChildNavigable {
    closed: (),
}

impl NullChildNavigable {
    pub(crate) const fn fresh() -> Self {
        Self { closed: () }
    }

    pub(crate) const fn is_null(&self) -> bool {
        true
    }

    // The content-document/window algorithms return null before origin or
    // WindowProxy access. Actual delay and destroy-child both require content.
    pub(crate) const fn content_document_is_null(&self) -> bool {
        self.is_null()
    }

    pub(crate) const fn content_window_is_null(&self) -> bool {
        self.is_null()
    }

    pub(crate) const fn delays_load(&self) -> bool {
        !self.is_null()
    }

    pub(crate) const fn removal_requires_child_destruction(&self) -> bool {
        !self.is_null()
    }
}

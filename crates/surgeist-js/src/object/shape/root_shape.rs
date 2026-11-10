use crate::{JsResult, Runtime};

use super::SharedShape;

/// This is a wrapper around [`SharedShape`] that ensures it's root shape.
///
/// Represent the root shape that [`SharedShape`] transitions start from.
#[derive(Debug, Clone)]
pub struct RootShape {
    shape: SharedShape,
    runtime: Runtime,
}

impl RootShape {
    /// Creates a root shape on the given runtime.
    pub fn new(runtime: Runtime) -> JsResult<Self> {
        Ok(Self {
            shape: SharedShape::root(&runtime.heap)?,
            runtime,
        })
    }

    pub(crate) const fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// Gets the inner [`SharedShape`].
    #[must_use]
    pub const fn shape(&self) -> &SharedShape {
        &self.shape
    }
}

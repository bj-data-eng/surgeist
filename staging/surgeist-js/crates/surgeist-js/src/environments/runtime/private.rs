use surgeist_js_gc::heap::ObjectId;

use crate::JsString;

/// Private names introduced by one class.
///
/// The class identity distinguishes names without retaining the class object.
#[derive(Debug)]
pub(crate) struct PrivateEnvironment {
    /// The unique identifier of the private names.
    id: ObjectId,

    /// The `[[Description]]` internal slot of the private names.
    descriptions: Vec<JsString>,
}

impl PrivateEnvironment {
    /// Creates a new `PrivateEnvironment`.
    pub(crate) fn new(id: ObjectId, descriptions: Vec<JsString>) -> Self {
        Self { id, descriptions }
    }

    /// Gets the id of this private environment.
    pub(crate) const fn id(&self) -> ObjectId {
        self.id
    }

    /// Gets the descriptions of this private environment.
    pub(crate) fn descriptions(&self) -> &[JsString] {
        &self.descriptions
    }
}

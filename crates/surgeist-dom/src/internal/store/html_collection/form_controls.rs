//! Retained Form and Fieldset element collections.

use super::{
    DomHtmlCollection, DomHtmlCollectionLimits, DomHtmlCollectionNames, DomHtmlCollectionRead,
    Filter,
};
use crate::internal::{
    identity::NodeId,
    store::{Dom, DomError},
};
use std::sync::atomic::AtomicBool;

/// A live HTMLFormControlsCollection tied to one exact native Form.
///
/// Each read resolves the Form's current ordinary tree and its listed controls.
#[derive(Debug)]
pub struct DomHtmlFormControlsCollection(DomHtmlCollection);

/// A qualified read of the Form's current listed controls.
pub struct DomHtmlFormControlsCollectionRead<'dom, A>(DomHtmlCollectionRead<'dom, A>);

impl<A> std::fmt::Debug for DomHtmlFormControlsCollectionRead<'_, A> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("DomHtmlFormControlsCollectionRead")
            .field(&self.0)
            .finish()
    }
}

impl Dom {
    /// Form the live HTMLFormControlsCollection for an exact native Form.
    ///
    /// # Errors
    /// Returns the first Source, identity, cancellation, or receiver failure.
    pub fn form_elements(
        &self,
        form: NodeId,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlFormControlsCollection, DomError> {
        self.with_collection_root(form, cancel, |_, root, step| {
            step()?;
            root.as_form().ok_or(DomError::WrongNodeKind)?;
            Ok(())
        })?;
        Ok(DomHtmlFormControlsCollection(DomHtmlCollection {
            root: form,
            filter: Filter::FormControls,
        }))
    }

    /// Form the live HTMLCollection of a Fieldset's listed ordinary descendants.
    ///
    /// # Errors
    /// Returns the first Source, identity, cancellation, or receiver failure.
    pub fn fieldset_elements(
        &self,
        fieldset: NodeId,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlCollection, DomError> {
        self.with_collection_root(fieldset, cancel, |_, root, step| {
            step()?;
            root.as_fieldset().ok_or(DomError::WrongNodeKind)?;
            Ok(())
        })?;
        Ok(DomHtmlCollection {
            root: fieldset,
            filter: Filter::FieldsetControls,
        })
    }

    /// Open a new Form collection read after a canonical tree change.
    ///
    /// # Errors
    /// Returns the first Source, identity, caller, cancellation, or work failure.
    pub fn read_html_form_controls_collection<'dom, A, E>(
        &'dom self,
        collection: &'dom DomHtmlFormControlsCollection,
        limits: DomHtmlCollectionLimits,
        admission: A,
        cancel: &'dom AtomicBool,
    ) -> Result<DomHtmlFormControlsCollectionRead<'dom, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        self.read_html_collection(&collection.0, limits, admission, cancel)
            .map(DomHtmlFormControlsCollectionRead)
    }
}

impl<A, E> DomHtmlFormControlsCollectionRead<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    /// Count the Form's current listed controls, supplying its native length.
    pub fn length(&self) -> Result<usize, E> {
        self.0.length()
    }

    /// Read the zero-based member in the current ordinary tree order.
    pub fn item(&self, index: usize) -> Result<Option<NodeId>, E> {
        self.0.item(index)
    }

    /// Return each nonempty ID or name once, in first occurrence order.
    pub fn supported_names(&self) -> Result<DomHtmlCollectionNames, E> {
        self.0.supported_names()
    }
}

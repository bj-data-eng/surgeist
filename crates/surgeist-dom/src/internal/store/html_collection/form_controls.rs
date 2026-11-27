//! Retained Form and Fieldset element collections.

use super::{
    DomHtmlCollection, DomHtmlCollectionLimits, DomHtmlCollectionNames, DomHtmlCollectionRead,
    ElementMatches, ElementNameScope, Filter,
};
use crate::internal::{
    batch::Command,
    identity::NodeId,
    source::DomSourceEdit,
    store::{Dom, DomError, DomResource, admit},
};
use std::{ops::ControlFlow, sync::atomic::AtomicBool};
use surgeist_html::{attributes::HtmlInputType, values::DocumentString};

/// A live HTMLFormControlsCollection tied to one exact native Form.
///
/// Each read resolves the Form's current ordinary tree and its listed controls.
#[derive(Debug)]
pub struct DomHtmlFormControlsCollection(DomHtmlCollection);

/// The current result of a named Form or Form collection lookup.
#[derive(Debug)]
pub enum DomFormNamedValue {
    /// The query currently names no element.
    Absent,
    /// Exactly one element is returned.
    Element(NodeId),
    /// Multiple elements match. The returned RadioNodeList remains live.
    List(DomRadioNodeList),
}

/// A live RadioNodeList retaining the exact Form and named filter.
#[derive(Debug)]
pub struct DomRadioNodeList(DomHtmlCollection);

impl DomSourceEdit {
    /// Set the checkedness of the first represented Radio Button whose
    /// current value exactly matches. An unmatched value changes no Input.
    ///
    /// # Errors
    /// Returns an error when the retained list has no qualified filter.
    pub fn set_radio_node_list_value(
        list: &DomRadioNodeList,
        value: impl Into<DocumentString>,
    ) -> Result<Self, DomError> {
        let Filter::NamedFormControls(name) = &list.0.filter else {
            return Err(DomError::WrongNodeKind);
        };
        Ok(Self {
            command: Command::SetRadioNodeListValue {
                form: list.0.root.into(),
                name: name.clone(),
                value: value.into(),
            },
        })
    }
}

/// A qualified read of the RadioNodeList's current members and value.
pub struct DomRadioNodeListRead<'dom, A>(DomHtmlCollectionRead<'dom, A>);

impl<A> std::fmt::Debug for DomRadioNodeListRead<'_, A> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("DomRadioNodeListRead")
            .field(&self.0)
            .finish()
    }
}

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

    /// Reopen the RadioNodeList against its Form's current ordinary tree.
    ///
    /// # Errors
    /// Returns the first Source, identity, caller, cancellation, or work failure.
    pub fn read_radio_node_list<'dom, A, E>(
        &'dom self,
        list: &'dom DomRadioNodeList,
        limits: DomHtmlCollectionLimits,
        admission: A,
        cancel: &'dom AtomicBool,
    ) -> Result<DomRadioNodeListRead<'dom, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        self.read_html_collection(&list.0, limits, admission, cancel)
            .map(DomRadioNodeListRead)
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

    /// Resolve an exact current ID or name. A multiple result retains the
    /// query backing and uses the same Form for subsequent reads.
    pub fn named_item(
        &self,
        name: &DocumentString,
        max_retained_query_units: usize,
    ) -> Result<DomFormNamedValue, E> {
        self.0.view.meter.step()?;
        if name.is_empty() {
            return Ok(DomFormNamedValue::Absent);
        }
        match self.0.first_two_matches(|node| {
            self.0
                .element_matches_name(node, name, ElementNameScope::Collection)
        })? {
            ElementMatches::Absent => Ok(DomFormNamedValue::Absent),
            ElementMatches::One(node) => Ok(DomFormNamedValue::Element(node)),
            ElementMatches::Multiple => {
                admit(
                    name.retained_units(),
                    max_retained_query_units,
                    DomResource::HtmlCollectionQueryUnits,
                )?;
                Ok(DomFormNamedValue::List(DomRadioNodeList(
                    DomHtmlCollection {
                        root: self.0.collection.root,
                        filter: Filter::NamedFormControls(name.clone()),
                    },
                )))
            }
        }
    }

    /// Return each nonempty ID or name once, in first occurrence order.
    pub fn supported_names(&self) -> Result<DomHtmlCollectionNames, E> {
        self.0.supported_names()
    }
}

impl<A, E> DomRadioNodeListRead<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    /// Count all current represented elements.
    pub fn length(&self) -> Result<usize, E> {
        self.0.length()
    }

    /// Read the zero-based member in current ordinary tree order.
    pub fn item(&self, index: usize) -> Result<Option<NodeId>, E> {
        self.0.item(index)
    }

    /// Borrow the first checked Radio Button's current exact value.
    /// An empty selection yields the empty string.
    pub fn value(&self) -> Result<&[u16], E> {
        let step = || self.0.view.meter.step();
        let value = self.0.walk_where(
            |node| node.as_input().is_some(),
            |node| {
                let input = node.as_input().ok_or(DomError::WrongNodeKind)?;
                if input.input_type_with(step)? != HtmlInputType::Radio {
                    return Ok(ControlFlow::Continue(()));
                }
                step()?;
                if input.state().flags().checkedness() {
                    return Ok(ControlFlow::Break(input.value_with(step)?));
                }
                Ok(ControlFlow::Continue(()))
            },
        )?;
        Ok(value.unwrap_or_default())
    }
}

//! Element-owned immutable feature bodies. Controlled custody stays with the
//! constructing transaction, never inside these semantic records.
use super::{DomElement, DomElementDefinition};
use crate::name::AttributeStorage;
use crate::store::{
    bank,
    custody::Usage,
    payload,
    source::{formation::Formation, storage},
};
use crate::{DocumentString, DomError, DomQualifiedName, DomRegistryBinding};
use std::{sync::Arc, sync::atomic::AtomicBool};
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ElementName {
    pub value: DomQualifiedName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AttributeBinding {
    pub storage: AttributeStorage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CustomAssociation {
    pub registry: DomRegistryBinding,
    pub definition: DomElementDefinition,
    pub saved_is: Option<DocumentString>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NonceSlot {
    pub value: DocumentString,
}

pub(crate) enum NonceUpdate {
    Unchanged,
    Set(DocumentString),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FocusReference<R> {
    pub value: R,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostedContent {
    pub value: crate::DomHostedNavigableId,
}

/// Only controls newly formed while constructing one Element binding. Existing
/// child targets are adopted by identity at the receiving language payload.
#[derive(Default)]
pub(crate) struct ElementChildren {
    pub name: Option<bank::ObjectCustody>,
    pub attributes: Option<bank::ObjectCustody>,
    pub native: Option<bank::ObjectCustody>,
    pub custom: Option<bank::ObjectCustody>,
    pub nonce: Option<bank::ObjectCustody>,
    pub focus: Option<bank::ObjectCustody>,
    pub hosted: Option<bank::ObjectCustody>,
}

pub(crate) struct FormedElement {
    pub value: DomElement,
    pub children: ElementChildren,
}

/// Borrowed admission authority for one actual Element construction/edit. It
/// does not outlive that operation or become part of an immutable snapshot.
pub(crate) struct ElementFormation<'a> {
    pub registry: &'a bank::CustodyRegistry,
    pub budget: Option<&'a Formation>,
    pub cancel: &'a AtomicBool,
    pub admit: &'a mut dyn FnMut(Usage) -> Result<(), DomError>,
    children: ElementChildren,
}
impl<'a> ElementFormation<'a> {
    pub(crate) fn new(
        registry: &'a bank::CustodyRegistry,
        budget: Option<&'a Formation>,
        cancel: &'a AtomicBool,
        admit: &'a mut dyn FnMut(Usage) -> Result<(), DomError>,
    ) -> Self {
        Self {
            registry,
            budget,
            cancel,
            admit,
            children: ElementChildren::default(),
        }
    }

    fn form<T>(
        &mut self,
        extra: storage::Payload,
        original: storage::Payload,
        build: impl FnOnce() -> Result<T, DomError>,
    ) -> Result<payload::Sealed<T>, DomError> {
        crate::store::check_cancel(self.cancel)?;
        let usage = Usage::new(size_of::<payload::Payload<T>>(), 0)
            .checked_add(Usage::new(extra.bytes, extra.units))?;
        payload::form_group(
            self.registry,
            self.budget,
            original,
            usage,
            &mut *self.admit,
            build,
        )
    }

    pub(crate) fn native<T>(
        &mut self,
        extra: storage::Payload,
        original: storage::Payload,
        build: impl FnOnce() -> Result<T, DomError>,
    ) -> Result<Arc<payload::Payload<T>>, DomError> {
        assert!(self.children.native.is_none(), "one selected native family");
        let formed = self.form(extra, original, build)?;
        self.children.native = Some(formed.custody);
        Ok(formed.value)
    }

    pub(crate) fn custom(
        &mut self,
        value: CustomAssociation,
        original: storage::Payload,
    ) -> Result<Arc<payload::Payload<CustomAssociation>>, DomError> {
        assert!(
            self.children.custom.is_none(),
            "one selected custom association"
        );
        let mut extra = storage::Payload::default();
        if let Some(text) = &value.saved_is {
            extra.string(text)?;
        }
        let formed = self.form(extra, original, || Ok(value))?;
        self.children.custom = Some(formed.custody);
        Ok(formed.value)
    }

    pub(crate) fn nonce(
        &mut self,
        value: DocumentString,
        original: storage::Payload,
    ) -> Result<Arc<payload::Payload<NonceSlot>>, DomError> {
        assert!(self.children.nonce.is_none(), "one selected nonce slot");
        let mut extra = storage::Payload::default();
        extra.string(&value)?;
        let formed = self.form(extra, original, || Ok(NonceSlot { value }))?;
        self.children.nonce = Some(formed.custody);
        Ok(formed.value)
    }

    pub(crate) fn fresh_nonce(&mut self) -> Result<Arc<payload::Payload<NonceSlot>>, DomError> {
        assert!(self.children.nonce.is_none(), "one selected nonce slot");
        let extra = storage::Payload {
            bytes: size_of::<Vec<u16>>(),
            units: 0,
        };
        let formed = self.form(extra, storage::Payload::default(), || {
            Ok(NonceSlot {
                value: DocumentString::from(""),
            })
        })?;
        self.children.nonce = Some(formed.custody);
        Ok(formed.value)
    }

    pub(crate) fn focus<R>(
        &mut self,
        value: R,
    ) -> Result<Arc<payload::Payload<FocusReference<R>>>, DomError> {
        assert!(
            self.children.focus.is_none(),
            "one selected focus reference"
        );
        let formed = self.form(
            storage::Payload::default(),
            storage::Payload::default(),
            || Ok(FocusReference { value }),
        )?;
        self.children.focus = Some(formed.custody);
        Ok(formed.value)
    }

    pub(crate) fn hosted(
        &mut self,
        value: crate::DomHostedNavigableId,
    ) -> Result<Arc<payload::Payload<HostedContent>>, DomError> {
        assert!(
            self.children.hosted.is_none(),
            "one selected hosted content"
        );
        let formed = self.form(
            storage::Payload::default(),
            storage::Payload::default(),
            || Ok(HostedContent { value }),
        )?;
        self.children.hosted = Some(formed.custody);
        Ok(formed.value)
    }

    pub(crate) fn finish(self, value: DomElement) -> FormedElement {
        FormedElement {
            value,
            children: self.children,
        }
    }
    pub(crate) fn into_children(self) -> ElementChildren {
        self.children
    }
}

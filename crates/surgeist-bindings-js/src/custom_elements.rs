//! Scoped Web IDL identity for a host-projected CustomElementRegistry.

use crate::{Error, NativeObjectToken, ObjectPrototype, RetainedCallbackObject, Scope};

pub(crate) struct RegistryBrand;

/// An exact registry object identity. The host qualifies its native registry
/// authority separately. This token does not keep the object alive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustomElementRegistryIdentity {
    owner: u64,
    index: u32,
}

/// A canonical publication joined to its exact authenticated guest object.
pub struct PublishedCustomElementRegistry<'scope, T> {
    value: crate::Value<'scope>,
    holder: RetainedCallbackObject,
    identity: CustomElementRegistryIdentity,
    host: T,
}

impl<'scope, T> PublishedCustomElementRegistry<'scope, T> {
    pub fn into_parts(
        self,
    ) -> (
        crate::Value<'scope>,
        RetainedCallbackObject,
        CustomElementRegistryIdentity,
        T,
    ) {
        (self.value, self.holder, self.identity, self.host)
    }
}

impl CustomElementRegistryIdentity {
    pub(crate) fn from_brand(token: NativeObjectToken<RegistryBrand>) -> Self {
        let (owner, index) = token.identity_parts();
        Self { owner, index }
    }
}

impl<'scope> Scope<'scope> {
    /// Observes the exact registered receiver without invoking guest code.
    pub fn custom_element_registry_identity(
        &mut self,
        value: &crate::Value<'scope>,
    ) -> Result<Option<CustomElementRegistryIdentity>, Error> {
        self.native_object_token::<RegistryBrand>(value)
            .map(|token| token.map(CustomElementRegistryIdentity::from_brand))
    }

    /// Form and retain an exact registry object in the current realm. The
    /// caller owns the corresponding canonical registry and its lifetime.
    pub fn project_custom_element_registry<Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        admit: impl FnOnce(usize, usize) -> Result<(Storage, Storage), E>,
    ) -> Result<(RetainedCallbackObject, CustomElementRegistryIdentity), E> {
        let (_, holder, identity, ()) = self
            .publish_custom_element_registry(prototype, admit, || Ok(()))?
            .into_parts();
        Ok((holder, identity))
    }

    /// Qualify a private registry wrapper, then publish it with the canonical
    /// host operation. A refused host operation discards the private brand.
    /// The closure receives no guest value and cannot expose the wrapper.
    pub fn publish_custom_element_registry<Storage: 'static, E: From<Error>, T>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        admit: impl FnOnce(usize, usize) -> Result<(Storage, Storage), E>,
        publish_host: impl FnOnce() -> Result<T, E>,
    ) -> Result<PublishedCustomElementRegistry<'scope, T>, E> {
        let prepared = self.prepare_retained_native_object(
            crate::NativeProjectionPrototype::Current(prototype),
            RegistryBrand,
            admit,
        )?;
        let identity = CustomElementRegistryIdentity::from_brand(prepared.token());
        let output = publish_host()?;
        let (value, holder, ()) = prepared.publish(|_| ());
        Ok(PublishedCustomElementRegistry {
            value,
            holder,
            identity,
            host: output,
        })
    }
}

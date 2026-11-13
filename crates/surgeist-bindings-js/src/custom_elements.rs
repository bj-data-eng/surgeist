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

impl CustomElementRegistryIdentity {
    pub(crate) fn from_brand(token: NativeObjectToken<RegistryBrand>) -> Self {
        let (owner, index) = token.identity_parts();
        Self { owner, index }
    }
}

impl<'scope> Scope<'scope> {
    /// Form and retain an exact registry object in the current realm. The
    /// caller owns the corresponding canonical registry and its lifetime.
    pub fn project_custom_element_registry<Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        admit: impl FnOnce(usize, usize) -> Result<(Storage, Storage), E>,
    ) -> Result<(RetainedCallbackObject, CustomElementRegistryIdentity), E> {
        let prepared = self.prepare_retained_native_object(
            crate::NativeProjectionPrototype::Current(prototype),
            RegistryBrand,
            admit,
        )?;
        let identity = CustomElementRegistryIdentity::from_brand(prepared.token());
        let (_, holder, ()) = prepared.publish(|_| ());
        Ok((holder, identity))
    }
}

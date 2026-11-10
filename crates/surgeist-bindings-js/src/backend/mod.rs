pub(crate) mod v8;

use crate::{Error, Realm, Root};

/// The intentional engine replacement boundary. Binding algorithms operate on
/// owned identities and exact UTF-16; concrete handles never cross this module.
/// Implementations own entry, rooting, exception capture, jobs and teardown.
pub(crate) trait Engine {
    fn create_realm(&mut self) -> Result<Realm, Error>;
    fn validate_realm(&self, realm: &Realm) -> Result<(), Error>;
    fn validate_root(&self, root: &Root) -> Result<(), Error>;
    fn evaluate(&mut self, realm: &Realm, source: &[u16]) -> Result<Root, Error>;
    fn string(&mut self, realm: &Realm, units: &[u16]) -> Result<Root, Error>;
    fn global(&mut self, realm: &Realm) -> Result<Root, Error>;
    fn get(&mut self, realm: &Realm, object: &Root, name: &[u16]) -> Result<Root, Error>;
    fn set(
        &mut self,
        realm: &Realm,
        object: &Root,
        name: &[u16],
        value: &Root,
    ) -> Result<(), Error>;
    fn call(
        &mut self,
        realm: &Realm,
        function: &Root,
        receiver: &Root,
        arguments: &[Root],
    ) -> Result<Root, Error>;
    fn to_utf16(&mut self, realm: &Realm, value: &Root) -> Result<Vec<u16>, Error>;
    fn to_boolean(&mut self, realm: &Realm, value: &Root) -> Result<bool, Error>;
    fn to_number(&mut self, realm: &Realm, value: &Root) -> Result<f64, Error>;
    fn strict_equals(&mut self, left: &Root, right: &Root) -> Result<bool, Error>;
    fn checkpoint(&mut self) -> Result<(), Error>;
    fn memory_pressure(&mut self);
    fn reap(&mut self);
}

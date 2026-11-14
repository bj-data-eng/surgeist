//! CSSOM namespace installation in a host-qualified Window realm.

use std::mem::size_of;

use surgeist_css::serialization::{CssSerializationError, CssomIdentifierLimits};
use surgeist_ledger::{Limits, Operation, SharedOperation, Tariffs};

use crate::conversion::{
    ArgumentValue, ConversionAccount, Resource, Work,
    declarations::{
        WINDOW_CSS_ESCAPE_ARITY, WINDOW_CSS_ESCAPE_NAME, WINDOW_CSS_NAME, css_escape_0_ident,
    },
};
use crate::{
    Arguments, Callback, Completion, Error, FunctionMetadata, HostFailureSlot, ObjectPrototype,
    PropertyDescriptor, PropertyKey, RecordFailure, Scope, WellKnownSymbol,
};

/// Per-realm conversion admission and per-call native identifier limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssNamespaceLimits {
    calls: u64,
    copied_units: u64,
    buffer_bytes: u64,
    identifier: CssomIdentifierLimits,
}

impl CssNamespaceLimits {
    /// Caps cumulative argument decisions, cumulative copied UTF-16 units,
    /// and live Rust conversion bytes. Native CSSOM limits apply per call.
    pub const fn new(
        calls: u64,
        copied_units: u64,
        buffer_bytes: u64,
        identifier: CssomIdentifierLimits,
    ) -> Self {
        Self {
            calls,
            copied_units,
            buffer_bytes,
            identifier,
        }
    }
}

impl Default for CssNamespaceLimits {
    fn default() -> Self {
        Self::new(
            1_000_000,
            16_777_216,
            262_144,
            CssomIdentifierLimits::default(),
        )
    }
}

struct Escape<Storage> {
    conversion: ConversionAccount,
    identifier: CssomIdentifierLimits,
    failure: HostFailureSlot<CssSerializationError, Storage>,
    _backing: Storage,
}

impl<Storage: 'static> Escape<Storage> {
    fn call<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let argument = css_escape_0_ident();
        let converted = self.conversion.run(scope, |conversion| {
            conversion.argument(&argument, arguments.get(argument.index()))
        })?;
        let ArgumentValue::Present(ident) = converted.get() else {
            return Err(Error::BackendUnavailable);
        };
        scope.check_active()?;
        match surgeist_cssom::css::escape(ident.as_code_units(), self.identifier) {
            Ok(output) => Ok(Completion::Return(scope.string(&output)?)),
            Err(error) => match self.failure.record(error) {
                Ok(failure) | Err(RecordFailure::Occupied { first: failure, .. }) => {
                    Err(scope.refuse_host(&failure))
                }
                Err(RecordFailure::Inactive { error, .. }) => Err(error),
            },
        }
    }
}

/// Requested Rust callback target, conversion account and shared Ledger bodies,
/// including the two reserved scopes. Returns `None` on size overflow.
/// The host keeps the separate native failure custody funded using
/// [`HostFailureSlot::storage_bytes`].
pub fn callback_storage_bytes<Storage>() -> Option<usize> {
    size_of::<Escape<Storage>>()
        .checked_add(ConversionAccount::retained_body_bytes())?
        .checked_add(SharedOperation::<Work, Resource>::reserved_backing_bytes(
            2,
        )?)
}

/// Installs the standard namespace in the entered, host-authenticated Window.
/// The host supplies already admitted callback and failure backing and calls
/// this before publishing the realm to guest code. An existing own global
/// property refuses installation, including one whose value is undefined.
///
/// # Errors
/// Returns the original collision, resource or backend refusal.
pub fn install_window_css<Storage: 'static>(
    scope: &mut Scope<'_>,
    limits: CssNamespaceLimits,
    callback_backing: Storage,
    failure_backing: Storage,
) -> Result<(), Error> {
    let global = scope.global()?;
    let css_key = PropertyKey::String(WINDOW_CSS_NAME);
    let conversion_limits = Limits::from_fn(
        |work| match work {
            Work::Argument | Work::Value => limits.calls,
            Work::CodeUnit => limits.copied_units,
            _ => 0,
        },
        |_| limits.buffer_bytes,
    );
    let ledger = Operation::with_reserved_scopes("window-css", conversion_limits, 2)
        .map_err(Error::ConversionLedger)?
        .share();
    let conversion = ConversionAccount::new(
        scope,
        ledger.clone(),
        ledger.root(),
        conversion_limits,
        Tariffs::identity(),
    )?;
    let escape = Escape {
        conversion,
        identifier: limits.identifier,
        failure: HostFailureSlot::new(&scope.realm_authority()?, failure_backing)?,
        _backing: callback_backing,
    };
    let operation = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| escape.call(scope, arguments)),
        FunctionMetadata {
            name: WINDOW_CSS_ESCAPE_NAME,
            length: WINDOW_CSS_ESCAPE_ARITY,
        },
    )?;
    let namespace = scope.object(ObjectPrototype::Default)?;
    scope.define_property(
        &namespace,
        PropertyKey::String(WINDOW_CSS_ESCAPE_NAME),
        PropertyDescriptor::Data {
            value: &operation,
            writable: true,
            enumerable: true,
            configurable: true,
        },
    )?;
    let tag_key = scope.well_known_symbol(WellKnownSymbol::ToStringTag)?;
    let tag = scope.string(WINDOW_CSS_NAME)?;
    scope.define_property(
        &namespace,
        PropertyKey::Value(&tag_key),
        PropertyDescriptor::Data {
            value: &tag,
            writable: false,
            enumerable: false,
            configurable: true,
        },
    )?;
    scope.define_new_property(
        &global,
        css_key,
        PropertyDescriptor::Data {
            value: &namespace,
            writable: true,
            enumerable: false,
            configurable: true,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_storage_includes_its_shared_ledger_and_two_reserved_scopes() {
        let ledger = SharedOperation::<Work, Resource>::reserved_backing_bytes(2).unwrap();
        let expected =
            size_of::<Escape<[u8; 3]>>() + ConversionAccount::retained_body_bytes() + ledger;
        assert_eq!(callback_storage_bytes::<[u8; 3]>(), Some(expected));
    }
}

//! CSSOM namespace installation in a host-qualified Window realm.

use std::mem::size_of;

use surgeist_css::serialization::{CssSerializationError, CssomIdentifierLimits};
use surgeist_ledger::{Limits, Operation, SharedOperation, Tariffs};

use crate::conversion::{
    ArgumentValue, ConversionAccount, Host, Hosted, Resource, Work,
    declarations::{
        WINDOW_CSS_ESCAPE_ARITY, WINDOW_CSS_ESCAPE_NAME, WINDOW_CSS_NAME, css_escape_0_ident,
    },
};
use crate::{
    Arguments, Callback, Completion, Error, FunctionMetadata, HostFailureSlot, ObjectPrototype,
    PropertyDescriptor, PropertyKey, RecordFailure, Scope, Value, WellKnownSymbol,
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

    fn conversion_limits(self) -> Limits<Work, Resource> {
        Limits::from_fn(
            |work| match work {
                Work::Argument | Work::Value => self.calls,
                Work::CodeUnit => self.copied_units,
                _ => 0,
            },
            |_| self.buffer_bytes,
        )
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
    callback_target_bytes::<Storage>()
        .checked_add(ConversionAccount::retained_body_bytes())?
        .checked_add(SharedOperation::<Work, Resource>::reserved_backing_bytes(
            2,
        )?)
}

/// Requested callback target body, excluding conversion-account setup and
/// separate native failure custody. A hosted installation funds conversion
/// setup using [`Hosted::storage_bytes`] and failure custody using
/// [`HostFailureSlot::storage_bytes`].
pub const fn callback_target_bytes<Storage>() -> usize {
    size_of::<Escape<Storage>>()
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
    let conversion_limits = limits.conversion_limits();
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
    install_account(
        scope,
        global,
        limits.identifier,
        conversion,
        callback_backing,
        failure_backing,
    )
}

/// Installs the namespace with producer-local caps and admission delegated to
/// the supplied host. Conversion setup backing funds [`Hosted::storage_bytes`];
/// callback and failure backing fund [`callback_target_bytes`] and
/// [`HostFailureSlot::storage_bytes`] independently.
///
/// # Errors
/// Returns the original collision, resource or backend refusal.
pub fn install_window_css_hosted<H: Host, B: 'static, Storage: 'static>(
    scope: &mut Scope<'_>,
    limits: CssNamespaceLimits,
    hosted: Hosted<H>,
    conversion_backing: B,
    callback_backing: Storage,
    failure_backing: Storage,
) -> Result<(), Error> {
    let global = scope.global()?;
    let conversion = hosted.install(scope, limits.conversion_limits(), conversion_backing)?;
    install_account(
        scope,
        global,
        limits.identifier,
        conversion,
        callback_backing,
        failure_backing,
    )
}

fn install_account<'scope, Storage: 'static>(
    scope: &mut Scope<'scope>,
    global: Value<'scope>,
    identifier: CssomIdentifierLimits,
    conversion: ConversionAccount,
    callback_backing: Storage,
    failure_backing: Storage,
) -> Result<(), Error> {
    let escape = Escape {
        conversion,
        identifier,
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
        PropertyKey::String(WINDOW_CSS_NAME),
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

    #[test]
    fn hosted_callback_target_excludes_independently_funded_conversion_setup() {
        assert_eq!(
            callback_target_bytes::<[u8; 3]>(),
            size_of::<Escape<[u8; 3]>>()
        );
        assert_eq!(callback_target_bytes::<()>(), size_of::<Escape<()>>());
    }

    #[test]
    fn conversion_caps_keep_calls_units_and_live_buffers_independent() {
        let limits = CssNamespaceLimits::new(7, 11, 13, CssomIdentifierLimits::default())
            .conversion_limits();
        assert_eq!(limits.work(Work::Argument), 7);
        assert_eq!(limits.work(Work::Value), 7);
        assert_eq!(limits.work(Work::CodeUnit), 11);
        for work in [
            Work::EnumerationLookup,
            Work::DictionaryMember,
            Work::IteratorOperation,
            Work::SequenceItem,
            Work::SequenceMove,
        ] {
            assert_eq!(limits.work(work), 0);
        }
        assert_eq!(limits.live(Resource::BufferBytes), 13);
    }
}

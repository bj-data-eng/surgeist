//! Typed Web IDL conversion with binding-owned admission and result custody.
//! Accounts borrow the entered execution boundary and never own native state.

mod account;
mod composition;
pub mod declarations;
mod failure;
mod listener;
mod sequence;
#[cfg(test)]
pub(crate) mod tests;
mod toggle_event;

use std::rc::Rc;

use surgeist_ledger::Change;

use crate::idl::string::ByteStringStep;
use crate::{
    ByteString, DomString, Error, Exception, ExceptionPhase, IntegerKind, IntegerMode,
    IntegerValue, Scope, UsvString, Value, ValueKind,
};

pub use account::{
    CleanupStatus, ConversionAccount, Converted, LedgerError, Resource, ResourceTable, Work,
    WorkTable,
};
pub use composition::{Argument, ArgumentValue, Descriptor, Nullable};
pub use failure::{
    CleanupFailure, CleanupFault, CleanupPrimary, CleanupStage, Failure, FailureKind, Rule,
};
pub use listener::StandaloneListenerOptions;
pub use sequence::Sequence;
pub use toggle_event::ToggleEventOptions;

use account::{Account, Storage};

type Scalar<T> = for<'borrow, 'scope> fn(
    &mut Conversion<'borrow, 'scope>,
    &Value<'scope>,
) -> Result<Converted<T>, Failure>;

enum Converter<T> {
    Scalar(Scalar<T>),
    Enumeration {
        name: &'static str,
        lookup: fn(&[u16]) -> Option<T>,
    },
}

/// Closed typed conversion descriptor. Its constructors are owned by bindings.
pub struct Type<T> {
    converter: Converter<T>,
    includes_undefined: bool,
}

impl<T> Type<T> {
    const fn scalar(convert: Scalar<T>) -> Self {
        Self {
            converter: Converter::Scalar(convert),
            includes_undefined: false,
        }
    }

    /// Adds IDL nullability once. This constructor is unavailable on a
    /// nullable descriptor, so nullable-of-nullable is not representable.
    pub fn nullable(self) -> Nullable<Self> {
        Nullable(self)
    }
    pub fn sequence(self) -> Sequence<Self> {
        Sequence(self)
    }
}

/// Declared enumeration descriptor, also usable as an ordinary typed value.
pub struct Enumeration<E>(Type<E>);

impl<E> Enumeration<E> {
    pub(crate) const fn new(name: &'static str, lookup: fn(&[u16]) -> Option<E>) -> Self {
        Self(Type {
            converter: Converter::Enumeration { name, lookup },
            includes_undefined: false,
        })
    }
    pub fn value_type(&self) -> &Type<E> {
        &self.0
    }
    /// Nullable ordinary enum conversion; assignment's ignored-nonmember
    /// policy remains a separate operation.
    pub fn nullable(self) -> Nullable<Type<E>> {
        self.0.nullable()
    }
    pub fn sequence(self) -> Sequence<Type<E>> {
        self.0.sequence()
    }
}

/// Enum assignment deliberately ignores nonmembers after one coercion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumAssignment<E> {
    Member(E),
    Ignored,
}

/// Closed integer kind/policy pair, using the existing typed IDL algorithms.
#[derive(Clone, Copy, Debug)]
pub struct IntegerType {
    kind: IntegerKind,
    mode: IntegerMode,
}

impl IntegerType {
    pub const fn new(kind: IntegerKind, mode: IntegerMode) -> Self {
        Self { kind, mode }
    }
    pub fn nullable(self) -> Nullable<Self> {
        Nullable(self)
    }
    pub fn sequence(self) -> Sequence<Self> {
        Sequence(self)
    }
}

/// Scalar descriptors; string allocation and numeric algorithms share the
/// existing core primitive implementations rather than repeating coercion.
pub mod types {
    use super::{
        Type, boolean, byte_string, dom_string, dom_string_null_to_empty, double, float,
        unrestricted_double, unrestricted_float, usv_string,
    };
    use crate::{ByteString, DomString, UsvString};
    pub const BOOLEAN: Type<bool> = Type::scalar(boolean);
    /// The unique IDL undefined value. Nullable undefined still retains
    /// undefined separately from null.
    pub const UNDEFINED: Type<()> = Type {
        converter: super::Converter::Scalar(super::undefined),
        includes_undefined: true,
    };
    pub const DOM_STRING: Type<DomString> = Type::scalar(dom_string);
    /// DOMString conversion with the selected LegacyNullToEmptyString annotation.
    pub const DOM_STRING_NULL_TO_EMPTY: Type<DomString> = Type::scalar(dom_string_null_to_empty);
    pub const USV_STRING: Type<UsvString> = Type::scalar(usv_string);
    pub const BYTE_STRING: Type<ByteString> = Type::scalar(byte_string);
    pub const FLOAT: Type<f32> = Type::scalar(float);
    pub const DOUBLE: Type<f64> = Type::scalar(double);
    pub const UNRESTRICTED_FLOAT: Type<f32> = Type::scalar(unrestricted_float);
    pub const UNRESTRICTED_DOUBLE: Type<f64> = Type::scalar(unrestricted_double);
}

/// One conversion run borrowing the actual entered scope. Ledger calls and
/// identity checks finish before any guest coercion or invocation.
pub struct Conversion<'borrow, 'scope> {
    scope: &'borrow mut Scope<'scope>,
    account: Rc<Account>,
}

impl<'scope> Conversion<'_, 'scope> {
    fn charge(&mut self, changes: &[Change<Work, Resource>]) -> Result<(), Error> {
        self.scope.check_active()?;
        self.account
            .ledger
            .apply_batch(self.account.scope, changes)
            .map_err(|error| self.scope.operations.refuse(Error::ConversionLedger(error)))
    }

    fn work(&mut self, work: Work, amount: u64) -> Result<(), Error> {
        if amount == 0 {
            return self.scope.check_active();
        }
        self.charge(&[Change::Work(work, amount)])
    }

    fn reserve(&mut self, bytes: u64) -> Result<Storage, Error> {
        if bytes != 0 {
            self.charge(&[Change::Acquire(Resource::BufferBytes, bytes)])?;
        }
        Ok(Storage {
            account: self.account.clone(),
            bytes,
        })
    }

    fn host(&self, error: Error) -> Failure {
        Failure(self.scope.operations.refuse(error))
    }

    fn core<T>(&self, result: Result<T, Error>) -> Result<T, Failure> {
        result.map_err(|error| match error {
            Error::Exception(_) => Failure(error),
            _ => self.host(error),
        })
    }

    fn plain<T>(&self, value: T) -> Converted<T> {
        Converted {
            value,
            storage: Storage {
                account: self.account.clone(),
                bytes: 0,
            },
        }
    }

    fn validation(&mut self, rule: Rule) -> Failure {
        // Fixed embedding diagnostic; neither formatting nor guest diagnostics
        // run after failure. The SDK selects the realm's intrinsic TypeError.
        let result = self.scope.type_error(&[
            87, 101, 98, 32, 73, 68, 76, 32, 99, 111, 110, 118, 101, 114, 115, 105, 111, 110, 32,
            102, 97, 105, 108, 101, 100,
        ]);
        match result {
            Ok(value) => Failure(Error::Exception(Exception {
                phase: ExceptionPhase::Coerce,
                value: self.scope.retain(&value),
                rule: Some(rule),
            })),
            Err(error) => self.host(error),
        }
    }

    fn string_units(&mut self, value: &Value<'scope>) -> Result<Converted<Vec<u16>>, Failure> {
        let result = self.scope.operations.coerce_string(&value.root);
        let string = self.core(result)?;
        let (mut output, storage) = self.string_buffer(string.len())?;
        output.resize(string.len(), 0);
        let result = self.scope.operations.copy_string(&string, &mut output);
        self.core(result)?;
        Ok(Converted {
            value: output,
            storage,
        })
    }

    /// Admit one explicit Rust-to-JavaScript payload copy before entering the
    /// SDK. The meter counts code units submitted to the backend, independently
    /// of the engine's internal representation and allocation.
    pub fn copy_to_js(&mut self, units: &[u16]) -> Result<Value<'scope>, Failure> {
        self.core(self.scope.operations.validate_string_length(units.len()))?;
        let length = u64::try_from(units.len()).map_err(|_| self.host(Error::StringTooLong))?;
        self.work(Work::CodeUnit, length).map_err(Failure)?;
        let result = self.scope.string(units);
        self.core(result)
    }

    fn default_dom_string(&mut self, units: &[u16]) -> Result<Converted<DomString>, Failure> {
        let (mut output, storage) = self.string_buffer(units.len())?;
        output.extend_from_slice(units);
        Ok(Converted {
            value: DomString::new(output),
            storage,
        })
    }

    fn string_buffer(&mut self, length: usize) -> Result<(Vec<u16>, Storage), Failure> {
        let units = u64::try_from(length).map_err(|_| self.host(Error::StringTooLong))?;
        let bytes = units
            .checked_mul(2)
            .ok_or_else(|| self.host(Error::StringTooLong))?;
        let storage = self.reserve(bytes).map_err(Failure)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(length)
            .map_err(|_| self.host(Error::Allocation))?;
        self.work(Work::CodeUnit, units).map_err(Failure)?;
        Ok((output, storage))
    }

    pub fn value<D: Descriptor>(
        &mut self,
        ty: &D,
        value: &Value<'scope>,
    ) -> Result<Converted<D::Output>, Failure> {
        composition::sealed::Convert::convert(ty, self, value)
    }

    fn typed<T>(&mut self, ty: &Type<T>, value: &Value<'scope>) -> Result<Converted<T>, Failure> {
        match &ty.converter {
            Converter::Scalar(convert) => convert(self, value),
            Converter::Enumeration { name, lookup } => {
                let units = self.string_units(value)?;
                self.work(Work::EnumerationLookup, 1).map_err(Failure)?;
                let result = match lookup(units.get()) {
                    Some(member) => Ok(self.plain(member)),
                    None => Err(self.validation(Rule::Enumeration(name))),
                };
                drop(units);
                result
            }
        }
    }

    pub fn enum_assignment<E>(
        &mut self,
        ty: &Enumeration<E>,
        value: &Value<'scope>,
    ) -> Result<EnumAssignment<E>, Failure> {
        self.work(Work::Value, 1).map_err(Failure)?;
        let units = self.string_units(value)?;
        self.work(Work::EnumerationLookup, 1).map_err(Failure)?;
        match &ty.0.converter {
            Converter::Enumeration { lookup, .. } => {
                Ok(lookup(units.get()).map_or(EnumAssignment::Ignored, EnumAssignment::Member))
            }
            Converter::Scalar(_) => Err(self.host(Error::BackendUnavailable)),
        }
    }

    fn integer(
        &mut self,
        ty: IntegerType,
        value: &Value<'scope>,
    ) -> Result<Converted<IntegerValue>, Failure> {
        match self.scope.integer(value, ty.kind, ty.mode) {
            Ok(value) => Ok(self.plain(value)),
            Err(Error::IntegerOutOfRange) => Err(self.validation(Rule::IntegerRange)),
            Err(error) => self.core(Err(error)),
        }
    }
}

fn undefined<'s>(cx: &mut Conversion<'_, 's>, value: &Value<'s>) -> Result<Converted<()>, Failure> {
    let result = cx.scope.kind(value);
    cx.core(result)?;
    Ok(cx.plain(()))
}

fn boolean<'s>(cx: &mut Conversion<'_, 's>, value: &Value<'s>) -> Result<Converted<bool>, Failure> {
    let result = cx.scope.boolean(value);
    cx.core(result).map(|value| cx.plain(value))
}

fn dom_string<'s>(
    cx: &mut Conversion<'_, 's>,
    value: &Value<'s>,
) -> Result<Converted<DomString>, Failure> {
    let Converted { value, storage } = cx.string_units(value)?;
    Ok(Converted {
        value: DomString::new(value),
        storage,
    })
}

fn dom_string_null_to_empty<'s>(
    cx: &mut Conversion<'_, 's>,
    value: &Value<'s>,
) -> Result<Converted<DomString>, Failure> {
    let kind = cx.scope.kind(value);
    if cx.core(kind)? == ValueKind::Null {
        return Ok(cx.plain(DomString::new(Vec::new())));
    }
    dom_string(cx, value)
}

fn usv_string<'s>(
    cx: &mut Conversion<'_, 's>,
    value: &Value<'s>,
) -> Result<Converted<UsvString>, Failure> {
    let units = cx.string_units(value)?;
    cx.work(
        Work::CodeUnit,
        u64::try_from(units.value.len()).map_err(|_| cx.host(Error::StringTooLong))?,
    )
    .map_err(Failure)?;
    let Converted { value, storage } = units;
    Ok(Converted {
        value: UsvString::normalize(value),
        storage,
    })
}

fn byte_string<'s>(
    cx: &mut Conversion<'_, 's>,
    value: &Value<'s>,
) -> Result<Converted<ByteString>, Failure> {
    let Converted {
        value,
        storage: original,
    } = cx.string_units(value)?;
    let mut storage = None;
    let result = ByteString::convert_with(value, |step| match step {
        ByteStringStep::Inspect => cx.work(Work::CodeUnit, 1),
        ByteStringStep::Allocate(length) => {
            let bytes = u64::try_from(length)
                .map_err(|_| cx.scope.operations.refuse(Error::StringTooLong))?;
            storage = Some(cx.reserve(bytes)?);
            Ok(())
        }
        ByteStringStep::Copy(length) => cx.work(
            Work::CodeUnit,
            u64::try_from(length).map_err(|_| cx.scope.operations.refuse(Error::StringTooLong))?,
        ),
    });
    let result = match result {
        Ok(value) => match storage {
            Some(storage) => Ok(Converted { value, storage }),
            None => Err(cx.host(Error::BackendUnavailable)),
        },
        Err(Error::ByteStringOutOfRange) => Err(cx.validation(Rule::ByteString)),
        Err(error) => Err(cx.host(error)),
    };
    drop(original);
    result
}

fn float<'s>(cx: &mut Conversion<'_, 's>, value: &Value<'s>) -> Result<Converted<f32>, Failure> {
    match cx.scope.float(value) {
        Ok(value) => Ok(cx.plain(value)),
        Err(Error::FloatOutOfRange) => Err(cx.validation(Rule::FloatingRange)),
        Err(error) => cx.core(Err(error)),
    }
}

fn double<'s>(cx: &mut Conversion<'_, 's>, value: &Value<'s>) -> Result<Converted<f64>, Failure> {
    match cx.scope.double(value) {
        Ok(value) => Ok(cx.plain(value)),
        Err(Error::FloatOutOfRange) => Err(cx.validation(Rule::FloatingRange)),
        Err(error) => cx.core(Err(error)),
    }
}

fn unrestricted_float<'s>(
    cx: &mut Conversion<'_, 's>,
    value: &Value<'s>,
) -> Result<Converted<f32>, Failure> {
    let result = cx.scope.unrestricted_float(value);
    cx.core(result).map(|value| cx.plain(value))
}

fn unrestricted_double<'s>(
    cx: &mut Conversion<'_, 's>,
    value: &Value<'s>,
) -> Result<Converted<f64>, Failure> {
    let result = cx.scope.unrestricted_double(value);
    cx.core(result).map(|value| cx.plain(value))
}

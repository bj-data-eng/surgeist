//! Standard constructed stylesheet projections for a host-authenticated Window.

use std::cell::{OnceCell, RefCell};
use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;

use surgeist_css::serialization::CssSerializationLimits;
use surgeist_cssom::{
    CssRule, CssRuleList, CssStyleSheet, CssomCapture, CssomError, CssomException,
    CssomInitialMedia, CssomMediaListId, CssomRuleFormatLimits, CssomRuleId, CssomSheetId,
    CssomSheetInit, CssomSheetText, CssomSnapshot, MediaList,
};
use surgeist_ledger::{Limits, Operation, Tariffs};

use crate::backend::{IndexedTrap, Projection};
use crate::conversion::cssom::MediaOption;
use crate::conversion::{Argument, ArgumentValue, ConversionAccount, IntegerType, Work, types};
use crate::dom_exception::DomExceptionName;
use crate::{
    Arguments, Callback, Completion, ConstructorArguments, ConstructorCallback,
    ConstructorCompletion, Error, Exception, ExceptionPhase, FunctionMetadata, IntegerKind,
    IntegerMode, IntegerValue, Intrinsic, ObjectPrototype, Primitive, PropertyDescriptor,
    PropertyKey, Scope, Value, ValueKind, WellKnownSymbol,
};

pub(crate) mod declarations;
mod indexed;
mod interface;
mod media;

pub use interface::{install, installation_storage_bytes};
pub use media::MediaEdit;

const fn ascii<const N: usize>(bytes: &[u8; N]) -> [u16; N] {
    let mut result = [0; N];
    let mut input: &[u8] = bytes;
    let mut output: &mut [u16] = &mut result;
    while let (Some((byte, rest)), Some((unit, tail))) =
        (input.split_first(), output.split_first_mut())
    {
        *unit = *byte as u16;
        input = rest;
        output = tail;
    }
    result
}

pub(crate) use declarations::{BASE_URL, DISABLED, MEDIA};
const MEDIA_TEXT: [u16; 9] = ascii(b"mediaText");

/// Canonical Window stylesheet authority. A receipt keeps the exact published
/// native CSSOM owner live while its stylesheet, rules or media are reachable.
/// Host methods must finish without running guest code. A refused operation
/// may create a typed guest exception in the entered scope.
pub trait StylesheetHost: 'static {
    /// Original native publication receipt.
    type Sheet: 'static;
    /// A live native retention debit.
    type Storage: 'static;

    /// Admit requested Rust payload backing before allocation.
    fn retain(&self, scope: &mut Scope<'_>, bytes: usize) -> Result<Self::Storage, Error>;
    /// Admit bounded projection work before observing or copying data.
    fn work(&self, scope: &mut Scope<'_>, units: usize) -> Result<(), Error>;
    /// Revalidate the original Window and publish exactly one native sheet.
    fn construct(
        &self,
        scope: &mut Scope<'_>,
        init: CssomSheetInit<'_>,
    ) -> Result<Self::Sheet, Error>;
    /// Exact identity of the publication receipt.
    fn sheet_id(sheet: &Self::Sheet) -> &CssomSheetId;
    /// Capture one revision of the original live store.
    fn snapshot(&self, scope: &mut Scope<'_>, sheet: &Self::Sheet) -> Result<CssomSnapshot, Error>;
    /// Insert into the original live sheet after its native guards.
    fn insert(
        &self,
        scope: &mut Scope<'_>,
        sheet: &Self::Sheet,
        text: Arc<Vec<u16>>,
        index: u32,
    ) -> Result<u32, Error>;
    /// Delete from the original live sheet after its native guards.
    fn delete(&self, scope: &mut Scope<'_>, sheet: &Self::Sheet, index: u32) -> Result<(), Error>;
    /// Set the native disabled flag.
    fn disable(
        &self,
        scope: &mut Scope<'_>,
        sheet: &Self::Sheet,
        disabled: bool,
    ) -> Result<(), Error>;
    /// Edit the exact media list in the original live store.
    fn edit_media(
        &self,
        scope: &mut Scope<'_>,
        sheet: &Self::Sheet,
        media: &CssomMediaListId,
        edit: MediaEdit,
        text: Arc<Vec<u16>>,
    ) -> Result<(), Error>;
    /// Preserve an original non-source CSSOM failure in native custody.
    fn refuse(&self, scope: &mut Scope<'_>, error: CssomError) -> Error;
}

struct RealmState<H: StylesheetHost> {
    host: H,
    account: ConversionAccount,
    _backing: H::Storage,
}

struct Sheet<H: StylesheetHost> {
    realm: Rc<RealmState<H>>,
    receipt: OnceCell<H::Sheet>,
    _backing: H::Storage,
}

struct SheetView<H: StylesheetHost> {
    sheet: Rc<Sheet<H>>,
    _backing: H::Storage,
}

struct Rules<H: StylesheetHost> {
    sheet: Rc<Sheet<H>>,
    native: OnceCell<CssRuleList>,
    cache: RefCell<RuleCache<H::Storage>>,
    _backing: H::Storage,
}

struct RuleCache<S> {
    entries: Vec<(CssomRuleId, Projection)>,
    backing: Option<S>,
}

struct Media<H: StylesheetHost> {
    sheet: Rc<Sheet<H>>,
    native: OnceCell<MediaList>,
    _backing: H::Storage,
}

struct Rule<H: StylesheetHost> {
    sheet: Rc<Sheet<H>>,
    rule: CssRule,
    _backing: H::Storage,
}

impl<H: StylesheetHost> Sheet<H> {
    fn receipt(&self) -> Result<&H::Sheet, Error> {
        self.receipt.get().ok_or(Error::BackendUnavailable)
    }

    fn capture(&self, scope: &mut Scope<'_>) -> Result<CssomCapture<CssStyleSheet>, Error> {
        let receipt = self.receipt()?;
        let snapshot = self.realm.host.snapshot(scope, receipt)?;
        self.result(
            scope,
            snapshot.capture_css_style_sheet(H::sheet_id(receipt)),
        )
    }

    fn result<T>(&self, scope: &mut Scope<'_>, value: Result<T, CssomError>) -> Result<T, Error> {
        value.map_err(|error| source_error(scope, &self.realm.host, error))
    }

    fn media(&self, scope: &mut Scope<'_>) -> Result<CssomCapture<MediaList>, Error> {
        let sheet = self.capture(scope)?;
        self.result(scope, sheet.media())
    }
}

#[derive(Clone, Copy)]
enum Interface {
    Sheet,
    CssSheet,
    Rules,
    Media,
    Rule,
}

impl Interface {
    fn prototype<'scope>(self, scope: &mut Scope<'scope>) -> Result<Value<'scope>, Error> {
        let root = scope.operations.cssom_prototype(self as u32)?;
        Ok(scope.value(root))
    }
}

fn state<T: 'static>(scope: &mut Scope<'_>, value: &Value<'_>) -> Result<Option<Rc<T>>, Error> {
    Ok(scope
        .operations
        .projection(&value.root)?
        .and_then(|state| state.downcast().ok()))
}

fn brand<T: 'static>(scope: &mut Scope<'_>, value: &Value<'_>) -> Result<Rc<T>, Error> {
    state(scope, value)?.ok_or_else(|| type_error(scope))
}

fn type_error(scope: &mut Scope<'_>) -> Error {
    match scope.type_error(&ascii(b"Illegal invocation")) {
        Ok(value) => Error::Exception(Exception {
            phase: ExceptionPhase::Invoke,
            value: scope.retain(&value),
            rule: None,
        }),
        Err(error) => error,
    }
}

/// Throw the exact DOMException for a native CSSOM source failure.
///
/// # Errors
/// Returns the guest exception or the original error creating its value.
pub fn stylesheet_exception(scope: &mut Scope<'_>, name: CssomException) -> Error {
    let name = match name {
        CssomException::Syntax => DomExceptionName::SyntaxError,
        CssomException::HierarchyRequest => DomExceptionName::HierarchyRequestError,
        CssomException::IndexSize => DomExceptionName::IndexSizeError,
        CssomException::Security => DomExceptionName::SecurityError,
        CssomException::NotAllowed => DomExceptionName::NotAllowedError,
        CssomException::NoModificationAllowed => DomExceptionName::NoModificationAllowedError,
        CssomException::InvalidState => DomExceptionName::InvalidStateError,
        CssomException::NotFound => DomExceptionName::NotFoundError,
        CssomException::InvalidAccess => DomExceptionName::InvalidAccessError,
    };
    match scope.dom_exception(name, &[]) {
        Ok(value) => Error::Exception(Exception {
            phase: ExceptionPhase::Invoke,
            value: scope.retain(&value),
            rule: None,
        }),
        Err(error) => error,
    }
}

fn source_error<H: StylesheetHost>(scope: &mut Scope<'_>, host: &H, error: CssomError) -> Error {
    let CssomError::Source(name) = error else {
        return host.refuse(scope, error);
    };
    stylesheet_exception(scope, name)
}

fn copy_units<H: StylesheetHost>(
    scope: &mut Scope<'_>,
    host: &H,
    units: &[u16],
) -> Result<(Arc<Vec<u16>>, H::Storage), Error> {
    let bytes = units
        .len()
        .checked_mul(size_of::<u16>())
        .and_then(|len| len.checked_add(size_of::<Vec<u16>>()))
        .ok_or(Error::Allocation)?;
    let backing = host.retain(scope, bytes)?;
    host.work(scope, units.len())?;
    let mut text = Vec::new();
    text.try_reserve_exact(units.len())
        .map_err(|_| Error::Allocation)?;
    text.extend_from_slice(units);
    Ok((Arc::new(text), backing))
}

fn js_text<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    realm: &RealmState<H>,
    text: &str,
) -> Result<Value<'scope>, Error> {
    let length = text.len();
    let bytes = length
        .checked_mul(size_of::<u16>())
        .ok_or(Error::Allocation)?;
    let _backing = realm.host.retain(scope, bytes)?;
    realm.host.work(scope, length)?;
    let mut units = Vec::new();
    units
        .try_reserve_exact(length)
        .map_err(|_| Error::Allocation)?;
    units.extend(text.encode_utf16());
    Ok(realm
        .account
        .run(scope, |conversion| conversion.copy_to_js(&units))?)
}

fn project_backing<T, H: StylesheetHost>(
    scope: &mut Scope<'_>,
    host: &H,
) -> Result<H::Storage, Error> {
    let bytes = scope
        .operations
        .projection_storage_bytes()
        .checked_add(size_of::<T>())
        .ok_or(Error::Allocation)?;
    host.retain(scope, bytes)
}

impl<H: StylesheetHost> RealmState<H> {
    fn construct<'scope>(
        self: &Rc<Self>,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let options = self.account.run(scope, |conversion| {
            conversion.sheet_options(arguments.get(0), state::<Media<H>>)
        })?;
        let invocation = arguments.select_prototype(scope)?;
        let shell = scope.object(ObjectPrototype::Object(invocation.prototype()))?;
        let shared_backing = self.host.retain(scope, size_of::<Sheet<H>>())?;
        let sheet = Rc::new(Sheet {
            realm: self.clone(),
            receipt: OnceCell::new(),
            _backing: shared_backing,
        });
        let rules = Rules::create(scope, sheet.clone())?;
        let media = Media::create(scope, sheet.clone())?;
        let backing = project_backing::<SheetView<H>, H>(scope, &self.host)?;
        scope.operations.project(
            &shell.root,
            Rc::new(SheetView {
                sheet: sheet.clone(),
                _backing: backing,
            }),
            &[&rules.root, &media.root],
        )?;
        let (base_url, _base_backing) = match options
            .base_url
            .as_ref()
            .and_then(|value| value.get().as_ref())
        {
            Some(base) => {
                let (text, backing) = copy_units(scope, &self.host, base.as_code_units())?;
                (Some(text), Some(backing))
            }
            None => (None, None),
        };
        let media_capture;
        let _media_backing;
        let media = match &options.media {
            MediaOption::Empty => CssomInitialMedia::Text("".into()),
            MediaOption::Text(value) => {
                let (text, backing) = copy_units(scope, &self.host, value.get().as_code_units())?;
                _media_backing = backing;
                CssomInitialMedia::Text(CssomSheetText::Utf16(text))
            }
            MediaOption::List(source) => {
                media_capture = source.capture(scope)?;
                CssomInitialMedia::List {
                    snapshot: media_capture.snapshot(),
                    id: media_capture.handle().id(),
                }
            }
        };
        let disabled = options.disabled;
        drop(options);
        scope.check_active()?;
        let receipt = self.host.construct(
            scope,
            CssomSheetInit {
                base_url,
                media,
                disabled,
            },
        )?;
        sheet.receipt.get_or_init(|| receipt);
        Ok(ConstructorCompletion::Return(shell))
    }
}

fn integer_argument<'scope>(
    account: &ConversionAccount,
    scope: &mut Scope<'scope>,
    arguments: &Arguments<'scope>,
    descriptor: &Argument<IntegerType>,
) -> Result<u32, Error> {
    let value = account.run(scope, |conversion| {
        conversion.argument(descriptor, arguments.get(descriptor.index()))
    })?;
    match value.get() {
        ArgumentValue::Present(IntegerValue::UnsignedLong(value)) => Ok(*value),
        _ => Err(Error::BackendUnavailable),
    }
}

fn number<'scope>(scope: &mut Scope<'scope>, value: u32) -> Result<Completion<'scope>, Error> {
    Ok(Completion::Return(
        scope.primitive(Primitive::Number(f64::from(value)))?,
    ))
}

fn nullable<'scope>(scope: &mut Scope<'scope>) -> Result<Completion<'scope>, Error> {
    Ok(Completion::Return(scope.primitive(Primitive::Null)?))
}

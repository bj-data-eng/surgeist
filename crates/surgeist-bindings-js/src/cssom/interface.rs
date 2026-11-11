//! Installation and Web IDL members of the constructed stylesheet surfaces.

use super::*;
use crate::conversion::declarations::{
    css_style_sheet_delete_rule_0_index, css_style_sheet_insert_rule_0_index,
    css_style_sheet_insert_rule_0_rule,
};

const PROTOTYPE: [u16; 9] = ascii(b"prototype");
const CONSTRUCTOR: [u16; 11] = ascii(b"constructor");
const CSS_RULES: [u16; 8] = ascii(b"cssRules");
const CSS_TEXT: [u16; 7] = ascii(b"cssText");
const LENGTH: [u16; 6] = ascii(b"length");
const ITEM: [u16; 4] = ascii(b"item");
const TYPE: [u16; 4] = ascii(b"type");

impl Interface {
    fn name(self) -> &'static [u16] {
        match self {
            Self::Sheet => declarations::STYLE_SHEET,
            Self::CssSheet => declarations::CSS_STYLE_SHEET,
            Self::Rules => declarations::CSS_RULE_LIST,
            Self::Media => declarations::MEDIA_LIST,
            Self::Rule => declarations::CSS_RULE,
        }
    }
}

/// Additional retained host and conversion target bodies before installation.
/// Generic engine functions, roots and heap allocations remain backend-owned.
pub const fn installation_storage_bytes<H: StylesheetHost>() -> usize {
    size_of::<RealmState<H>>()
        + size_of::<Rc<RealmState<H>>>()
        + ConversionAccount::retained_body_bytes()
}

/// Install the constructed stylesheet surfaces in an unpublished authenticated
/// Window. If installation fails the caller discards the unpublished realm.
///
/// # Errors
/// Propagates the exact admission, global property collision, or engine failure.
pub fn install<H: StylesheetHost>(
    scope: &mut Scope<'_>,
    host: H,
    backing: H::Storage,
) -> Result<(), Error> {
    let limits = Limits::from_fn(
        |work| match work {
            Work::Argument | Work::Value | Work::DictionaryMember => 1_000_000,
            Work::CodeUnit => 16_777_216,
            _ => 0,
        },
        |_| 262_144,
    );
    let ledger = Operation::new("window-cssom", limits, 2)
        .map_err(Error::ConversionLedger)?
        .share();
    let account = ConversionAccount::new(
        scope,
        ledger.clone(),
        ledger.root(),
        limits,
        Tariffs::identity(),
    )?;
    let state = Rc::new(RealmState {
        host,
        account,
        _backing: backing,
    });
    let global = scope.global()?;
    let sheet = interface::<H>(scope, Interface::Sheet, None, None)?;
    let css_sheet = interface(scope, Interface::CssSheet, Some(&sheet), Some(state))?;
    let rules = interface::<H>(scope, Interface::Rules, None, None)?;
    let media = interface::<H>(scope, Interface::Media, None, None)?;
    let rule = interface::<H>(scope, Interface::Rule, None, None)?;
    sheet_members::<H>(scope, &sheet.1, &css_sheet.1)?;
    list_members::<Rules<H>>(scope, &rules.1)?;
    list_members::<Media<H>>(scope, &media.1)?;
    attribute(
        scope,
        &media.1,
        &MEDIA_TEXT,
        &ascii(b"get mediaText"),
        media_text::<H>,
        None,
    )?;
    method(scope, &media.1, &ascii(b"toString"), 0, 0, media_text::<H>)?;
    rule_members::<H>(scope, &rule.1)?;
    let entries = [sheet, css_sheet, rules, media, rule];
    let prototypes = entries.each_ref().map(|(_, prototype)| &prototype.root);
    scope.operations.cssom_prototypes(&prototypes)?;
    for (entry, interface) in entries.iter().zip([
        Interface::Sheet,
        Interface::CssSheet,
        Interface::Rules,
        Interface::Media,
        Interface::Rule,
    ]) {
        scope.define_new_property(
            &global,
            PropertyKey::String(interface.name()),
            PropertyDescriptor::Data {
                value: &entry.0,
                writable: true,
                enumerable: false,
                configurable: true,
            },
        )?;
    }
    Ok(())
}

type Pair<'scope> = (Value<'scope>, Value<'scope>);

fn interface<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    kind: Interface,
    parent: Option<&Pair<'scope>>,
    state: Option<Rc<RealmState<H>>>,
) -> Result<Pair<'scope>, Error> {
    let prototype = scope.object(parent.map_or(ObjectPrototype::Default, |pair| {
        ObjectPrototype::Object(&pair.1)
    }))?;
    let body = ConstructorCallback::with_argument_limit(
        Some(if state.is_some() {
            declarations::CAPTURED_OPTIONS
        } else {
            0
        }),
        move |scope| kind.prototype(scope),
        move |scope, arguments| match &state {
            Some(state) => state.construct(scope, arguments),
            None => Err(type_error(scope)),
        },
    );
    let constructor = scope.constructor(
        &body,
        FunctionMetadata {
            name: kind.name(),
            length: declarations::CONSTRUCTOR_LENGTH,
        },
    )?;
    if let Some((parent, _)) = parent {
        scope.set_prototype(&constructor, ObjectPrototype::Object(parent))?;
    }
    data(
        scope,
        &constructor,
        PropertyKey::String(&PROTOTYPE),
        &prototype,
        false,
        false,
        false,
    )?;
    data(
        scope,
        &prototype,
        PropertyKey::String(&CONSTRUCTOR),
        &constructor,
        true,
        false,
        true,
    )?;
    let symbol = scope.well_known_symbol(WellKnownSymbol::ToStringTag)?;
    let tag = scope.string(kind.name())?;
    data(
        scope,
        &prototype,
        PropertyKey::Value(&symbol),
        &tag,
        false,
        false,
        true,
    )?;
    Ok((constructor, prototype))
}

fn data<'scope>(
    scope: &mut Scope<'scope>,
    object: &Value<'scope>,
    key: PropertyKey<'_, 'scope>,
    value: &Value<'scope>,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) -> Result<(), Error> {
    scope.define_property(
        object,
        key,
        PropertyDescriptor::Data {
            value,
            writable,
            enumerable,
            configurable,
        },
    )
}

type Member =
    for<'scope> fn(&mut Scope<'scope>, Arguments<'scope>) -> Result<Completion<'scope>, Error>;

fn method<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    key: &[u16],
    length: u32,
    capture: u32,
    member: Member,
) -> Result<(), Error> {
    let function = scope.bounded_function(
        &Callback::new(member),
        FunctionMetadata { name: key, length },
        capture,
    )?;
    data(
        scope,
        prototype,
        PropertyKey::String(key),
        &function,
        true,
        true,
        true,
    )
}

fn attribute<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    key: &[u16],
    name: &[u16],
    get: Member,
    setter: Option<(&[u16], Member)>,
) -> Result<(), Error> {
    let get =
        scope.fixed_arity_function(&Callback::new(get), FunctionMetadata { name, length: 0 })?;
    let set = setter
        .map(|(name, set)| {
            scope.fixed_arity_function(&Callback::new(set), FunctionMetadata { name, length: 1 })
        })
        .transpose()?;
    scope.define_property(
        prototype,
        PropertyKey::String(key),
        PropertyDescriptor::Accessor {
            get: Some(&get),
            set: set.as_ref(),
            enumerable: true,
            configurable: true,
        },
    )
}

fn list_members<'scope, L: indexed::List>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
) -> Result<(), Error> {
    attribute(
        scope,
        prototype,
        &LENGTH,
        &ascii(b"get length"),
        indexed::length::<L>,
        None,
    )?;
    method(scope, prototype, &ITEM, 1, 1, indexed::item::<L>)?;
    let iterator = scope.well_known_symbol(WellKnownSymbol::Iterator)?;
    let values = scope.intrinsic(Intrinsic::ArrayValues)?;
    data(
        scope,
        prototype,
        PropertyKey::Value(&iterator),
        &values,
        true,
        false,
        true,
    )
}

fn sheet_members<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    sheet: &Value<'scope>,
    css_sheet: &Value<'scope>,
) -> Result<(), Error> {
    attribute(
        scope,
        sheet,
        &TYPE,
        &ascii(b"get type"),
        sheet_type::<H>,
        None,
    )?;
    attribute(
        scope,
        sheet,
        DISABLED,
        &ascii(b"get disabled"),
        disabled::<H>,
        Some((&ascii(b"set disabled"), set_disabled::<H>)),
    )?;
    attribute(
        scope,
        sheet,
        MEDIA,
        &ascii(b"get media"),
        media::<H>,
        Some((&ascii(b"set media"), forward_media::<H>)),
    )?;
    for (key, name) in [
        (CSS_RULES.as_slice(), &ascii(b"get cssRules")[..]),
        (&ascii(b"rules")[..], &ascii(b"get rules")[..]),
    ] {
        attribute(scope, css_sheet, key, name, rules::<H>, None)?;
    }
    method(scope, css_sheet, &ascii(b"insertRule"), 1, 2, insert::<H>)?;
    method(scope, css_sheet, &ascii(b"deleteRule"), 1, 1, delete::<H>)
}

fn sheet_type<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    let sheet = view.sheet.capture(scope)?;
    let text = view.sheet.result(scope, sheet.r#type())?;
    Ok(Completion::Return(js_text(scope, &view.sheet.realm, text)?))
}

fn disabled<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    let sheet = view.sheet.capture(scope)?;
    let value = view.sheet.result(scope, sheet.disabled())?;
    Ok(Completion::Return(
        scope.primitive(Primitive::Boolean(value))?,
    ))
}

fn set_disabled<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    let disabled = args
        .get(0)
        .map(|value| {
            view.sheet
                .realm
                .account
                .run(scope, |conversion| conversion.value(&types::BOOLEAN, value))
        })
        .transpose()?
        .is_some_and(|value| *value.get());
    view.sheet
        .realm
        .host
        .disable(scope, view.sheet.receipt()?, disabled)?;
    Ok(Completion::Undefined)
}

fn rules<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    let sheet = view.sheet.capture(scope)?;
    let rules = view.sheet.result(scope, sheet.css_rules())?;
    let root = scope.operations.projection_link(&args.receiver().root, 0)?;
    let wrapper = scope.value(root);
    state::<Rules<H>>(scope, &wrapper)?
        .ok_or(Error::BackendUnavailable)?
        .bind(rules.handle());
    Ok(Completion::Return(wrapper))
}

fn media<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    let media = view.sheet.media(scope)?;
    let root = scope.operations.projection_link(&args.receiver().root, 1)?;
    let wrapper = scope.value(root);
    state::<Media<H>>(scope, &wrapper)?
        .ok_or(Error::BackendUnavailable)?
        .bind(media.handle());
    Ok(Completion::Return(wrapper))
}

fn forward_media<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    view.sheet.realm.host.work(scope, 1)?;
    let object = scope.get(args.receiver(), PropertyKey::String(MEDIA))?;
    if scope.kind(&object)? != ValueKind::Object {
        return Err(type_error(scope));
    }
    let undefined = scope.primitive(Primitive::Undefined)?;
    let value = args.get(0).unwrap_or(&undefined);
    match scope.set(&object, PropertyKey::String(&MEDIA_TEXT), value) {
        Ok(()) | Err(Error::PropertyRejected) => Ok(Completion::Undefined),
        Err(error) => Err(error),
    }
}

fn media_text<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<Media<H>>(scope, args.receiver())?;
    let media = state.capture(scope)?;
    let text = state
        .sheet
        .result(scope, media.media_text(CssSerializationLimits::default()))?;
    Ok(Completion::Return(js_text(
        scope,
        &state.sheet.realm,
        &text,
    )?))
}

fn insert<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    let account = &view.sheet.realm.account;
    let descriptor = css_style_sheet_insert_rule_0_rule();
    let text = account.run(scope, |conversion| {
        conversion.argument(&descriptor, args.get(0))
    })?;
    let index = integer_argument(
        account,
        scope,
        &args,
        &css_style_sheet_insert_rule_0_index(),
    )?;
    let ArgumentValue::Present(string) = text.get() else {
        return Err(Error::BackendUnavailable);
    };
    let host = &view.sheet.realm.host;
    let (units, _backing) = copy_units(scope, host, string.as_code_units())?;
    drop(text);
    scope.check_active()?;
    let result = scope.primitive(Primitive::Number(f64::from(index)))?;
    host.insert(scope, view.sheet.receipt()?, units, index)?;
    Ok(Completion::Return(result))
}

fn delete<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    let index = integer_argument(
        &view.sheet.realm.account,
        scope,
        &args,
        &css_style_sheet_delete_rule_0_index(),
    )?;
    view.sheet
        .realm
        .host
        .delete(scope, view.sheet.receipt()?, index)?;
    Ok(Completion::Undefined)
}

fn rule_members<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    rule: &Value<'scope>,
) -> Result<(), Error> {
    attribute(
        scope,
        rule,
        &TYPE,
        &ascii(b"get type"),
        rule_type::<H>,
        None,
    )?;
    attribute(
        scope,
        rule,
        &CSS_TEXT,
        &ascii(b"get cssText"),
        rule_text::<H>,
        None,
    )
}

fn rule_capture<H: StylesheetHost>(
    scope: &mut Scope<'_>,
    state: &Rule<H>,
) -> Result<CssomCapture<CssRule>, Error> {
    let sheet = state.sheet.capture(scope)?;
    state
        .sheet
        .result(scope, state.rule.capture(sheet.snapshot()))
}

fn rule_type<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<Rule<H>>(scope, args.receiver())?;
    let rule = rule_capture(scope, &state)?;
    let value = state.sheet.result(scope, rule.r#type())?;
    number(scope, u32::from(value))
}

fn rule_text<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<Rule<H>>(scope, args.receiver())?;
    let rule = rule_capture(scope, &state)?;
    let text = state
        .sheet
        .result(scope, rule.css_text(&CssomRuleFormatLimits::default()))?;
    Ok(Completion::Return(js_text(
        scope,
        &state.sheet.realm,
        &text,
    )?))
}

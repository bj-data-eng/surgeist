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
    list_iterator(scope, &rules.1)?;
    media::attribute_member::<H>(scope, &media.1)?;
    list_members::<Media<H>>(scope, &media.1)?;
    media::operation_members::<H>(scope, &media.1)?;
    list_iterator(scope, &media.1)?;
    rule_members::<H>(scope, &rule.1)?;
    let entries = [sheet, css_sheet, rules, media, rule];
    // Web IDL installs the prototype's constructor after its members.
    for (constructor, prototype) in &entries {
        data(
            scope,
            prototype,
            PropertyKey::String(&CONSTRUCTOR),
            constructor,
            true,
            false,
            true,
        )?;
    }
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
    let body = match state {
        Some(state) => ConstructorCallback::with_argument_limit(
            Some(declarations::CAPTURED_OPTIONS),
            |scope| Interface::CssSheet.prototype(scope),
            move |scope, arguments| state.construct(scope, arguments),
        ),
        None => ConstructorCallback::with_argument_limit(
            Some(0),
            |_| Err(Error::BackendUnavailable),
            |scope, _| Err(type_error(scope)),
        ),
    };
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

pub(super) fn method<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    key: &[u16],
    length: u32,
    capture: u32,
    member: Callback,
) -> Result<(), Error> {
    let function =
        scope.bounded_function(&member, FunctionMetadata { name: key, length }, capture)?;
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

pub(super) fn attribute<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    key: &[u16],
    name: &[u16],
    get: Callback,
    setter: Option<(&[u16], Callback)>,
) -> Result<(), Error> {
    let get = scope.fixed_arity_function(&get, FunctionMetadata { name, length: 0 })?;
    let set = setter
        .map(|(name, set)| scope.fixed_arity_function(&set, FunctionMetadata { name, length: 1 }))
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
        Callback::new(indexed::length::<L>),
        None,
    )?;
    method(
        scope,
        prototype,
        &ITEM,
        1,
        1,
        Callback::new(indexed::item::<L>),
    )
}

fn list_iterator<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
) -> Result<(), Error> {
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
        Callback::new(sheet_type::<H>),
        None,
    )?;
    attribute(
        scope,
        sheet,
        &ascii(b"href"),
        &ascii(b"get href"),
        Callback::new(sheet_href::<H>),
        None,
    )?;
    attribute(
        scope,
        sheet,
        MEDIA,
        &ascii(b"get media"),
        Callback::new(media::<H>),
        Some((&ascii(b"set media"), Callback::new(forward_media::<H>))),
    )?;
    attribute(
        scope,
        sheet,
        DISABLED,
        &ascii(b"get disabled"),
        Callback::new(disabled::<H>),
        Some((&ascii(b"set disabled"), Callback::new(set_disabled::<H>))),
    )?;
    for (key, name) in [
        (CSS_RULES.as_slice(), &ascii(b"get cssRules")[..]),
        (&ascii(b"rules")[..], &ascii(b"get rules")[..]),
    ] {
        attribute(scope, css_sheet, key, name, Callback::new(rules::<H>), None)?;
    }
    method(
        scope,
        css_sheet,
        &ascii(b"insertRule"),
        1,
        2,
        Callback::new(insert::<H>),
    )?;
    method(
        scope,
        css_sheet,
        &ascii(b"deleteRule"),
        1,
        1,
        Callback::new(delete::<H>),
    )
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

fn sheet_href<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let view = brand::<SheetView<H>>(scope, args.receiver())?;
    let sheet = view.sheet.capture(scope)?;
    match view.sheet.result(scope, sheet.href())? {
        Some(href) => Ok(Completion::Return(js_text(scope, &view.sheet.realm, href)?)),
        None => nullable(scope),
    }
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
        &CSS_TEXT,
        &ascii(b"get cssText"),
        Callback::new(rule_text::<H>),
        Some((&ascii(b"set cssText"), Callback::new(set_rule_text::<H>))),
    )?;
    attribute(
        scope,
        rule,
        &TYPE,
        &ascii(b"get type"),
        Callback::new(rule_type::<H>),
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

fn set_rule_text<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<Rule<H>>(scope, args.receiver())?;
    let undefined = scope.primitive(Primitive::Undefined)?;
    let input = args.get(0).unwrap_or(&undefined);
    state.sheet.realm.account.run(scope, |conversion| {
        conversion.value(&types::DOM_STRING, input)
    })?;
    Ok(Completion::Undefined)
}

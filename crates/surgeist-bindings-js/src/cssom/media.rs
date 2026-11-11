//! MediaList member operations over its exact live native identity.

use super::interface::{attribute, method};
use super::*;
use crate::conversion::declarations::{
    media_list_append_medium_0_medium, media_list_delete_medium_0_medium,
};

/// Authored mutation sent to the actual native MediaList.
#[derive(Clone, Copy)]
pub enum MediaEdit {
    /// Replace all media using `mediaText`.
    SetText,
    /// Append one canonical medium.
    Append,
    /// Remove the canonical medium.
    Delete,
}

pub(super) fn attribute_member<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
) -> Result<(), Error> {
    attribute(
        scope,
        prototype,
        &MEDIA_TEXT,
        &ascii(b"get mediaText"),
        Callback::new(media_text::<H>),
        Some((&ascii(b"set mediaText"), Callback::new(set_media_text::<H>))),
    )
}

pub(super) fn operation_members<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
) -> Result<(), Error> {
    method(
        scope,
        prototype,
        &ascii(b"toString"),
        0,
        0,
        Callback::new(media_text::<H>),
    )?;
    method(
        scope,
        prototype,
        &ascii(b"appendMedium"),
        1,
        1,
        Callback::new(append::<H>),
    )?;
    method(
        scope,
        prototype,
        &ascii(b"deleteMedium"),
        1,
        1,
        Callback::new(delete::<H>),
    )
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

fn set_media_text<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<Media<H>>(scope, args.receiver())?;
    let undefined = scope.primitive(Primitive::Undefined)?;
    let input = args.get(0).unwrap_or(&undefined);
    let value = state
        .sheet
        .realm
        .account
        .run(scope, |conversion| conversion.legacy_media_text(input))?;
    let host = &state.sheet.realm.host;
    let (text, _backing) = copy_units(scope, host, value.get().as_code_units())?;
    drop(value);
    commit(scope, &state, MediaEdit::SetText, text)
}

fn append<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    edit::<H>(
        scope,
        args,
        MediaEdit::Append,
        &media_list_append_medium_0_medium(),
    )
}

fn delete<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    edit::<H>(
        scope,
        args,
        MediaEdit::Delete,
        &media_list_delete_medium_0_medium(),
    )
}

fn edit<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    args: Arguments<'scope>,
    edit: MediaEdit,
    descriptor: &Argument<crate::conversion::Type<crate::DomString>>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<Media<H>>(scope, args.receiver())?;
    let value = state.sheet.realm.account.run(scope, |conversion| {
        conversion.argument(descriptor, args.get(descriptor.index()))
    })?;
    let ArgumentValue::Present(units) = value.get() else {
        return Err(Error::BackendUnavailable);
    };
    let host = &state.sheet.realm.host;
    let (text, _backing) = copy_units(scope, host, units.as_code_units())?;
    drop(value);
    commit(scope, &state, edit, text)
}

fn commit<'scope, H: StylesheetHost>(
    scope: &mut Scope<'scope>,
    state: &Media<H>,
    edit: MediaEdit,
    text: Arc<Vec<u16>>,
) -> Result<Completion<'scope>, Error> {
    scope.check_active()?;
    let media = state.native.get().ok_or(Error::BackendUnavailable)?;
    state
        .sheet
        .realm
        .host
        .edit_media(scope, state.sheet.receipt()?, media.id(), edit, text)?;
    Ok(Completion::Undefined)
}

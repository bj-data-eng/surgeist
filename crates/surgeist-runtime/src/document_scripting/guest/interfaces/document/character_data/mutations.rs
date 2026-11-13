//! Web IDL CharacterData reads and the original canonical Source edit.

use super::*;
use crate::{DomSourceDriveError, DomSourceDriveProgress, DomSourceRuntimeError};
use surgeist_bindings_js::conversion::{
    IntegerType,
    declarations::{
        character_data_append_data_0_data, character_data_data_setter,
        character_data_delete_data_0_count, character_data_delete_data_0_offset,
        character_data_insert_data_0_data, character_data_insert_data_0_offset,
        character_data_replace_data_0_count, character_data_replace_data_0_data,
        character_data_replace_data_0_offset, character_data_substring_data_0_count,
        character_data_substring_data_0_offset,
    },
};
use surgeist_bindings_js::{IntegerValue, dom_exception::DomExceptionName};
use surgeist_dom::{DomSourceEdit, DomSourceStatus};

#[derive(Clone, Copy)]
enum Operation {
    Substring,
    Append,
    Insert,
    Delete,
    Replace,
}

struct Method {
    name: &'static [u16],
    length: u8,
    offset: Option<Argument<IntegerType>>,
    count: Option<Argument<IntegerType>>,
    data: Option<Argument<Type<DomString>>>,
}

const SUBSTRING: &[u16] = &ascii(b"substringData");
const APPEND: &[u16] = &ascii(b"appendData");
const INSERT: &[u16] = &ascii(b"insertData");
const DELETE: &[u16] = &ascii(b"deleteData");
const REPLACE: &[u16] = &ascii(b"replaceData");

impl Operation {
    fn method(self) -> Method {
        match self {
            Self::Substring => Method {
                name: SUBSTRING,
                length: 2,
                offset: Some(character_data_substring_data_0_offset()),
                count: Some(character_data_substring_data_0_count()),
                data: None,
            },
            Self::Append => Method {
                name: APPEND,
                length: 1,
                offset: None,
                count: None,
                data: Some(character_data_append_data_0_data()),
            },
            Self::Insert => Method {
                name: INSERT,
                length: 2,
                offset: Some(character_data_insert_data_0_offset()),
                count: None,
                data: Some(character_data_insert_data_0_data()),
            },
            Self::Delete => Method {
                name: DELETE,
                length: 2,
                offset: Some(character_data_delete_data_0_offset()),
                count: Some(character_data_delete_data_0_count()),
                data: None,
            },
            Self::Replace => Method {
                name: REPLACE,
                length: 3,
                offset: Some(character_data_replace_data_0_offset()),
                count: Some(character_data_replace_data_0_count()),
                data: Some(character_data_replace_data_0_data()),
            },
        }
    }
}

pub(super) fn install_methods<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    for operation in [
        Operation::Substring,
        Operation::Append,
        Operation::Insert,
        Operation::Delete,
        Operation::Replace,
    ] {
        let Method { name, length, .. } = operation.method();
        let callback = capture(size_of::<(Capture, Operation)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                callback.character_operation(scope, arguments, operation)
            }),
            FunctionMetadata {
                name,
                length: u32::from(length),
            },
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

pub(super) fn data_setter<'scope>(
    scope: &mut Scope<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let setter = capture(size_of::<Capture>())?;
    Ok(scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| setter.assign_character_data(scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"set data"),
            length: 1,
        },
    )?)
}

impl Capture {
    fn character_operation<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        operation: Operation,
    ) -> Result<Completion<'scope>, Error> {
        let Some(node) = character_receiver(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"CharacterData required"))?,
            ));
        };
        let method = operation.method();
        if arguments.len() < usize::from(method.length) {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Not enough arguments"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let (offset, count, data) = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    let offset = method
                        .offset
                        .as_ref()
                        .map(|ty| cx.argument(ty, arguments.get(ty.index())))
                        .transpose()?;
                    let count = method
                        .count
                        .as_ref()
                        .map(|ty| cx.argument(ty, arguments.get(ty.index())))
                        .transpose()?;
                    let data = method
                        .data
                        .as_ref()
                        .map(|ty| cx.argument(ty, arguments.get(ty.index())))
                        .transpose()?;
                    Ok((offset, count, data))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let offset = unsigned_long(offset.as_ref())?;
            let count = unsigned_long(count.as_ref())?.unwrap_or(0);
            if matches!(operation, Operation::Substring) {
                let offset = offset.ok_or(Error::BackendUnavailable)?;
                let current = self.character_data(&owner, &events, node)?;
                let Some(remaining) = current.len().checked_sub(offset) else {
                    return index_size(scope);
                };
                let selected = current
                    .code_units()
                    .get(offset..offset + count.min(remaining))
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                let value = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| cx.copy_to_js(selected))
                    .map_err(Error::from)?;
                return Ok(Completion::Return(value));
            }
            let data = match data.as_ref().map(Converted::get) {
                Some(ArgumentValue::Present(data)) => data.as_code_units(),
                Some(ArgumentValue::Missing) => return Err(Error::BackendUnavailable.into()),
                None => &[],
            };
            let offset = match offset {
                Some(offset) => offset,
                None => self.character_data(&owner, &events, node)?.len(),
            };
            self.replace_character_data(scope, &owner, &events, node, (offset, count, data))
        })();
        character_completion(scope, &events, result)
    }

    fn assign_character_data<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(node) = character_receiver(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"CharacterData required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let absent;
            let input = match arguments.get(0) {
                Some(input) => input,
                None => {
                    absent = scope.primitive(Primitive::Undefined)?;
                    &absent
                }
            };
            let data = self
                .callbacks
                .conversion
                .run(scope, |cx| cx.value(&character_data_data_setter(), input))
                .map_err(Error::from)?;
            scope.check_active()?;
            self.replace_character_data(
                scope,
                &owner,
                &events,
                node,
                (0, usize::MAX, data.get().as_code_units()),
            )
        })();
        character_completion(scope, &events, result)
    }

    fn replace_character_data<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        node: GuestNode,
        (offset, count, units): (usize, usize, &[u16]),
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let (data, _backing) = copy_native_data(owner, events, units)?;
        let event = owner.active_event_request();
        let (mut result, parent) =
            self.callbacks
                .with_qualified_session(owner, event.as_ref(), |session, _| {
                    let mut source = session.source_execution();
                    let mut dom = session.dom();
                    let parent = dom.pending_source_edit().map(DomSourceStatus::operation);
                    let result = source.apply(
                        &mut dom,
                        DomSourceEdit::replace_data(node.node(), offset, count, data),
                        events.limits.source_edits,
                        &mut guest_source_services(&mut |_| None),
                        &owner.cancellation,
                    );
                    Ok((result, parent))
                })?;
        owner.finish_guest_style(
            &mut result,
            events.limits.source_edits,
            Some(scope.realm_authority()?.identity()),
        );
        let pending = owner
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .read_dom()
            .pending_source_edit();
        match result {
            Ok(DomSourceDriveProgress::Complete(done)) if done.cancellation_requested() => {
                Err(DocumentScriptNativeError::from(DomError::Cancelled).into())
            }
            Ok(DomSourceDriveProgress::Complete(_)) => Ok(Completion::Undefined),
            Ok(DomSourceDriveProgress::Failed(failure))
                if matches!(failure.error(), DomError::IndexSize) =>
            {
                index_size(scope)
            }
            Ok(DomSourceDriveProgress::Failed(failure)) => {
                Err(DocumentScriptNativeError::from(failure.into_parts().1).into())
            }
            Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(DomError::IndexSize)))
                if pending.map(DomSourceStatus::operation) == parent =>
            {
                index_size(scope)
            }
            Err(error) => refuse(scope, &attachment.source_failures, error).map_err(Into::into),
            Ok(_) => Err(
                pending.map_or(DocumentGuestEventError::HostRouteUnavailable, |status| {
                    DocumentGuestEventError::SourcePending(status.operation())
                }),
            ),
        }
    }
}

fn index_size<'scope>(
    scope: &mut Scope<'scope>,
) -> Result<Completion<'scope>, DocumentGuestEventError> {
    Ok(Completion::Throw(
        scope.dom_exception(DomExceptionName::IndexSizeError, &[])?,
    ))
}

fn unsigned_long(
    value: Option<&Converted<ArgumentValue<IntegerValue>>>,
) -> Result<Option<usize>, DocumentGuestEventError> {
    match value.map(Converted::get) {
        Some(ArgumentValue::Present(IntegerValue::UnsignedLong(value))) => {
            Ok(Some(usize::try_from(*value).map_err(|_| {
                DocumentScriptNativeError::RepresentationExhausted
            })?))
        }
        Some(_) => Err(Error::BackendUnavailable.into()),
        None => Ok(None),
    }
}

fn character_completion<'scope>(
    scope: &mut Scope<'scope>,
    events: &GuestEvents,
    result: Result<Completion<'scope>, DocumentGuestEventError>,
) -> Result<Completion<'scope>, Error> {
    match result {
        Ok(completion) => Ok(completion),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}

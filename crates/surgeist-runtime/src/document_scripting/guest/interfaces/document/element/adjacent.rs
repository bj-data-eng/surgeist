//! Legacy adjacent insertion through the canonical compound Source owner.

use super::super::character_data::copy_native_data;
use super::*;
use surgeist_bindings_js::conversion::{ArgumentValue, declarations};
use surgeist_dom::{DomSourceEdit, DomSourceOutput};

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    element: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<Capture>())?;
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.insert_adjacent_element(scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"insertAdjacentElement"),
            length: 2,
        },
    )?;
    define_data(
        scope,
        element,
        PropertyKey::String(&ascii(b"insertAdjacentElement")),
        &method,
        true,
        true,
        true,
    )?;
    let handler = capture(size_of::<Capture>())?;
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.insert_adjacent_text(scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"insertAdjacentText"),
            length: 2,
        },
    )?;
    define_data(
        scope,
        element,
        PropertyKey::String(&ascii(b"insertAdjacentText")),
        &method,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn insert_adjacent_element<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(receiver)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        if arguments.len() < 2 {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Two arguments required"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let position = declarations::element_insert_adjacent_element_0_where();
            let position = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    cx.argument(&position, arguments.get(position.index()))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let ArgumentValue::Present(position) = position.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            events.step()?;
            let (_, values) = arguments.into_parts();
            let value = values.into_iter().nth(1).ok_or(Error::BackendUnavailable)?;
            let Some(GuestNode::Element(element)) = tree::projected_node(scope, &value)? else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"Element required"))?,
                ));
            };
            let (position, _backing) = copy_native_data(&owner, &events, position.as_code_units())?;
            let absent = scope.primitive(Primitive::Null)?;
            let drive = self.drive_guest_tree_edit(
                &owner,
                &events,
                scope.realm_authority()?.identity(),
                DomSourceEdit::insert_adjacent_element(receiver, position, element),
            )?;
            self.finish_guest_tree_edit(
                scope,
                &owner,
                &events,
                drive.progress,
                (drive.parent, drive.pending),
                |_, done| match done.output() {
                    Some(DomSourceOutput::AdjacentElement(Some(id))) if id == element => {
                        Ok(Completion::Return(value))
                    }
                    Some(DomSourceOutput::AdjacentElement(None)) => Ok(Completion::Return(absent)),
                    _ => Err(DocumentScriptNativeError::ProjectionCustody.into()),
                },
            )
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn insert_adjacent_text<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(element)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        if arguments.len() < 2 {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Two arguments required"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let position = declarations::element_insert_adjacent_text_0_where();
            let data = declarations::element_insert_adjacent_text_0_data();
            let (position, data) = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    let position = cx.argument(&position, arguments.get(position.index()))?;
                    let data = cx.argument(&data, arguments.get(data.index()))?;
                    Ok((position, data))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let (ArgumentValue::Present(position), ArgumentValue::Present(data)) =
                (position.get(), data.get())
            else {
                return Err(Error::BackendUnavailable.into());
            };
            let (position, _position_backing) =
                copy_native_data(&owner, &events, position.as_code_units())?;
            let (data, _data_backing) = copy_native_data(&owner, &events, data.as_code_units())?;
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::insert_adjacent_text(element, position, data),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}

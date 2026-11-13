use super::*;
use surgeist_dom::{DomProcessingInstructionAttributeEdit as Edit, DomSourceEdit, DomSourceOutput};

#[derive(Clone, Copy)]
enum Method {
    HasAny,
    Names,
    Get,
    Has,
    Set,
    Remove,
    Toggle,
}

pub(in crate::document_scripting::guest::interfaces::document) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, method, length, maximum) in [
        (&ascii(b"hasAttributes")[..], Method::HasAny, 0, 0),
        (&ascii(b"getAttributeNames")[..], Method::Names, 0, 0),
        (&ascii(b"getAttribute")[..], Method::Get, 1, 1),
        (&ascii(b"hasAttribute")[..], Method::Has, 1, 1),
        (&ascii(b"setAttribute")[..], Method::Set, 2, 2),
        (&ascii(b"removeAttribute")[..], Method::Remove, 1, 1),
        (&ascii(b"toggleAttribute")[..], Method::Toggle, 1, 2),
    ] {
        let handler = capture(size_of::<(Capture, Method, u32)>())?;
        let callback = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                handler.instruction_attribute(scope, arguments, method, length)
            }),
            FunctionMetadata { name, length },
            maximum,
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &callback,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn instruction_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        method: Method,
        length: u32,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::ProcessingInstruction(node)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"ProcessingInstruction required"))?,
            ));
        };
        if arguments.len() < length as usize {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing attribute arguments"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let declaration = match method {
                Method::HasAny => {
                    let present = self.callbacks.with_read(&owner, &events, |read, _| {
                        Ok(instruction(read.node(node)?)?.attributes().len() != 0)
                    })?;
                    return Ok(Completion::Return(
                        scope.primitive(Primitive::Boolean(present))?,
                    ));
                }
                Method::Names => {
                    let (native_names, _backing) =
                        self.callbacks.with_read(&owner, &events, |read, _| {
                            let instruction = instruction(read.node(node)?)?;
                            let count = instruction.attributes().len();
                            let copied = scope.argument_copy_storage_bytes(count)?;
                            let bytes = count
                                .checked_mul(
                                    size_of::<DocumentString>() + size_of::<Value<'scope>>(),
                                )
                                .and_then(|size| size.checked_add(copied))
                                .ok_or(Error::Allocation)?;
                            let backing = read.reserve(bytes)?;
                            let mut names = reserved_names(count)?;
                            for (name, _) in instruction.attributes() {
                                read.step()?;
                                names.push(name.clone());
                            }
                            Ok((names, backing))
                        })?;
                    let mut names = reserved_names(native_names.len())?;
                    for name in native_names {
                        names.push(self.copy_dom_units(scope, name.code_units())?);
                    }
                    return Ok(Completion::Return(scope.array(&names)?));
                }
                Method::Get => declarations::processing_instruction_get_attribute_0_name(),
                Method::Has => declarations::processing_instruction_has_attribute_0_name(),
                Method::Set => declarations::processing_instruction_set_attribute_0_name(),
                Method::Remove => declarations::processing_instruction_remove_attribute_0_name(),
                Method::Toggle => declarations::processing_instruction_toggle_attribute_0_name(),
            };
            let (name, value, force) = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    let name =
                        conversion.argument(&declaration, arguments.get(declaration.index()))?;
                    let value = if matches!(method, Method::Set) {
                        let value = declarations::processing_instruction_set_attribute_0_value();
                        Some(conversion.argument(&value, arguments.get(value.index()))?)
                    } else {
                        None
                    };
                    let force = if matches!(method, Method::Toggle) {
                        let force = declarations::processing_instruction_toggle_attribute_0_force();
                        match conversion
                            .argument(&force, arguments.get(force.index()))?
                            .get()
                        {
                            ArgumentValue::Missing => None,
                            ArgumentValue::Present(force) => Some(*force),
                        }
                    } else {
                        None
                    };
                    Ok((name, value, force))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let ArgumentValue::Present(name) = name.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let units = name.as_code_units();
            if matches!(method, Method::Get | Method::Has) {
                let attribute = self.callbacks.with_read(&owner, &events, |read, _| {
                    Ok(instruction(read.node(node)?)?
                        .attribute_with(units, || read.step())?
                        .map(|(_, value)| value.clone()))
                })?;
                let observation = if matches!(method, Method::Has) {
                    Observation::Boolean(attribute.is_some())
                } else {
                    Observation::String(attribute)
                };
                return Ok(Completion::Return(self.project_observation(
                    scope,
                    &owner,
                    &events,
                    observation,
                )?));
            }
            let (name, _name_backing) = copy_native_data(&owner, &events, units)?;
            let (edit, _value_backing) = match method {
                Method::Set => {
                    let Some(ArgumentValue::Present(value)) = value.as_ref().map(Converted::get)
                    else {
                        return Err(Error::BackendUnavailable.into());
                    };
                    let (value, backing) =
                        copy_native_data(&owner, &events, value.as_code_units())?;
                    (Edit::Set { name, value }, Some(backing))
                }
                Method::Remove => (Edit::Remove { name }, None),
                Method::Toggle => (Edit::Toggle { name, force }, None),
                _ => return Err(Error::BackendUnavailable.into()),
            };
            let toggle_results = if matches!(method, Method::Toggle) {
                Some((
                    scope.primitive(Primitive::Boolean(false))?,
                    scope.primitive(Primitive::Boolean(true))?,
                ))
            } else {
                None
            };
            let drive = self.drive_guest_tree_edit(
                &owner,
                &events,
                scope.realm_authority()?.identity(),
                DomSourceEdit::edit_processing_instruction_attribute(node, edit),
            )?;
            self.finish_guest_tree_edit(
                scope,
                &owner,
                &events,
                drive.progress,
                (drive.parent, drive.pending),
                |_, done| {
                    let Some((absent, present)) = toggle_results else {
                        return Ok(Completion::Undefined);
                    };
                    match done.output() {
                        Some(DomSourceOutput::ProcessingInstructionAttributePresent(true)) => {
                            Ok(Completion::Return(present))
                        }
                        Some(DomSourceOutput::ProcessingInstructionAttributePresent(false)) => {
                            Ok(Completion::Return(absent))
                        }
                        _ => Err(DocumentScriptNativeError::ProjectionCustody.into()),
                    }
                },
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}

fn instruction(
    node: surgeist_dom::DomNode<'_>,
) -> Result<&surgeist_dom::DomProcessingInstruction, DocumentGuestEventError> {
    match node.kind() {
        DomNodeKind::ProcessingInstruction(instruction) => Ok(instruction),
        _ => Err(DocumentScriptNativeError::ProjectionCustody.into()),
    }
}

fn reserved_names<T>(count: usize) -> Result<Vec<T>, Error> {
    let mut names = Vec::new();
    names
        .try_reserve_exact(count)
        .map_err(|_| Error::Allocation)?;
    if names.capacity() != count {
        return Err(Error::Allocation);
    }
    Ok(names)
}

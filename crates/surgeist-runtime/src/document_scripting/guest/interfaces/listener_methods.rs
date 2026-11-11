//! Public listener operations over the original native registry.

use super::*;
use crate::document_scripting::guest::listeners::ListenerMatch;
use crate::document_scripting::guest::wrapper::{GuestReceiver, event_receiver};
use surgeist_bindings_js::conversion::{
    StandaloneListenerOptions,
    declarations::{
        event_target_add_event_listener_0_type, event_target_remove_event_listener_0_type,
    },
};

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Remove,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    for (operation, name) in [
        (Operation::Add, &ascii(b"addEventListener")[..]),
        (Operation::Remove, &ascii(b"removeEventListener")[..]),
    ] {
        let method_capture = capture(size_of::<(Capture, Operation)>())?;
        let function = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                method_capture.listener(scope, arguments, operation)
            }),
            FunctionMetadata { name, length: 2 },
            3,
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

impl Capture {
    fn listener<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        operation: Operation,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let Some(target) = event_receiver(&owner, scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"EventTarget required"))?,
            ));
        };
        if arguments.len() < 2 {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Two arguments required"))?,
            ));
        }
        match self.change_listener(scope, &owner, &events, &arguments, &target, operation) {
            Ok(()) => Ok(Completion::Undefined),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn change_listener<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        arguments: &Arguments<'scope>,
        target: &GuestReceiver,
        operation: Operation,
    ) -> Result<(), DocumentGuestEventError> {
        let ty = match operation {
            Operation::Add => event_target_add_event_listener_0_type(),
            Operation::Remove => event_target_remove_event_listener_0_type(),
        };
        let (ty, has_callback) = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                let ty = conversion.argument(&ty, arguments.get(ty.index()))?;
                let has_callback = conversion.event_listener_argument(arguments.get(1))?;
                Ok((ty, has_callback))
            })
            .map_err(Error::from)?;
        let ArgumentValue::Present(ty) = ty.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let callback = if has_callback {
            let callback = arguments.get(1).ok_or(Error::BackendUnavailable)?;
            Some((callback, scope.callback_object(callback)?))
        } else {
            None
        };
        let flags = self
            .callbacks
            .conversion
            .run(scope, |conversion| match operation {
                Operation::Add => conversion.standalone_add_listener_options(arguments.get(2)),
                Operation::Remove => Ok(StandaloneListenerOptions {
                    capture: conversion.event_listener_removal_capture(arguments.get(2))?,
                    ..Default::default()
                }),
            })
            .map_err(Error::from)?;
        let Some((callback, observed)) = callback else {
            return Ok(());
        };
        let callback_realm = observed.associated_realm();
        if callback_realm.identity() != self.callbacks.permit.realm {
            return Err(DocumentGuestEventError::CallbackRealmUnavailable);
        }
        scope.check_active()?;
        events.step()?;
        self.with_native(owner, |_, _| Ok(()))?;
        let native_target = target.target();
        let key = ListenerMatch {
            receiver: arguments.receiver(),
            target: &native_target,
            event_type: ty.as_code_units(),
            callback,
            capture: flags.capture,
        };
        let existing = events.find_listener(owner, scope, key)?;
        match (operation, existing) {
            (Operation::Remove, Some(id)) => {
                events.remove_registration(scope, id)?;
            }
            (Operation::Add, None) => {
                let options = DomEventListenerOptions {
                    capture: flags.capture,
                    once: flags.once,
                    passive: flags.passive,
                };
                match target {
                    GuestReceiver::Projected(projection) => events.register_traced(
                        owner,
                        scope,
                        projection.payload(),
                        key,
                        options,
                        callback_realm.clone(),
                    )?,
                    GuestReceiver::Window(_) => {
                        events.register_retained(owner, scope, key, options, observed, None)?;
                    }
                    GuestReceiver::Node { fragment, .. } => {
                        events.register_retained(
                            owner,
                            scope,
                            key,
                            options,
                            observed,
                            fragment.clone(),
                        )?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

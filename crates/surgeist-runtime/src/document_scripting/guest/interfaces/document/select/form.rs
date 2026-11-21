//! Select and Option form owners from the canonical published DOM segment.

use super::super::{reflection::Receiver, *};

#[derive(Clone, Copy)]
pub(super) enum FormControl {
    Select,
    Option,
}

impl FormControl {
    fn receiver(self) -> Receiver {
        match self {
            Self::Select => Receiver::Select,
            Self::Option => Receiver::Option,
        }
    }
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    control: FormControl,
) -> Result<(), DocumentGuestEventError> {
    let get = capture(size_of::<(Capture, FormControl)>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"form"),
        &ascii(b"get form"),
        true,
        Callback::new(move |scope, arguments| {
            get.read_select_family_form(scope, arguments, control)
        }),
    )?;
    Ok(())
}

impl Capture {
    fn read_select_family_form<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        control: FormControl,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = control.receiver();
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let form = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                let owner = match control {
                    FormControl::Select => node
                        .as_select()
                        .ok_or(DomError::WrongNodeKind)?
                        .form_owner_with(|| read.step())?,
                    FormControl::Option => node
                        .as_option()
                        .ok_or(DomError::WrongNodeKind)?
                        .form_owner_with(|| read.step())?,
                };
                Ok(Some(owner.map(|form| form.id())))
            })?;
            let Some(form) = form else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            Ok(Completion::Return(self.project_observation(
                scope,
                &owner,
                &events,
                Observation::Node(form),
            )?))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

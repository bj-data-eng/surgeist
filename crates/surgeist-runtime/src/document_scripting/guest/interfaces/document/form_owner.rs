//! Control and Legend form observations from the published DOM segment.

use super::{reflection::Receiver, *};

#[derive(Clone, Copy)]
pub(super) enum FormOwnerGetter {
    Select,
    Option,
    Button,
    Fieldset,
    Output,
    Legend,
}

impl FormOwnerGetter {
    fn receiver(self) -> Receiver {
        match self {
            Self::Select => Receiver::Select,
            Self::Option => Receiver::Option,
            Self::Button => Receiver::Button,
            Self::Fieldset => Receiver::Fieldset,
            Self::Output => Receiver::Output,
            Self::Legend => Receiver::Legend,
        }
    }
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    member: FormOwnerGetter,
) -> Result<(), DocumentGuestEventError> {
    let get = capture(size_of::<(Capture, FormOwnerGetter)>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"form"),
        &ascii(b"get form"),
        true,
        Callback::new(move |scope, arguments| get.read_form_owner(scope, arguments, member)),
    )?;
    Ok(())
}

impl Capture {
    fn read_form_owner<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: FormOwnerGetter,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = member.receiver();
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
                let owner = match member {
                    FormOwnerGetter::Select => node
                        .as_select()
                        .ok_or(DomError::WrongNodeKind)?
                        .form_owner_with(|| read.step())?,
                    FormOwnerGetter::Option => node
                        .as_option()
                        .ok_or(DomError::WrongNodeKind)?
                        .form_owner_with(|| read.step())?,
                    FormOwnerGetter::Button => node
                        .as_button()
                        .ok_or(DomError::WrongNodeKind)?
                        .form_owner_with(|| read.step())?,
                    FormOwnerGetter::Fieldset => node
                        .as_fieldset()
                        .ok_or(DomError::WrongNodeKind)?
                        .form_owner_with(|| read.step())?,
                    FormOwnerGetter::Output => node
                        .as_output()
                        .ok_or(DomError::WrongNodeKind)?
                        .form_owner_with(|| read.step())?,
                    FormOwnerGetter::Legend => {
                        node.fieldset_parent_form_owner_with(|| read.step())?
                    }
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

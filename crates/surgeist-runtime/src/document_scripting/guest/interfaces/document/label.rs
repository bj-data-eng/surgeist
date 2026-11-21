//! Live HTMLLabelElement associations and raw for reflection.

use super::{
    form_owner::{self, FormOwnerGetter},
    reflection::{self, Receiver, Reflection},
    *,
};

const MEMBERS: &[reflection::Member] = &[(
    &ascii(b"htmlFor"),
    &ascii(b"get htmlFor"),
    &ascii(b"set htmlFor"),
    &ascii(b"for"),
    Reflection::String,
)];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    form_owner::install(scope, prototype, capture, FormOwnerGetter::Label)?;
    reflection::install(scope, prototype, capture, Receiver::Label, MEMBERS)?;
    let get = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"control"),
        &ascii(b"get control"),
        true,
        Callback::new(move |scope, arguments| get.read_labeled_control(scope, arguments)),
    )?;
    Ok(())
}

impl Capture {
    fn read_labeled_control<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Label;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let control = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                Ok(Some(node.html_labeled_control_with(
                    || read.step(),
                    |bytes| read.reserve(bytes),
                )?))
            })?;
            let Some(control) = control else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            Ok(Completion::Return(self.project_observation(
                scope,
                &owner,
                &events,
                Observation::Node(control),
            )?))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

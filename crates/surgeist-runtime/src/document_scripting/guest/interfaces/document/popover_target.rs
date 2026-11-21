//! Live Button and Input popover target reflection.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_bindings_js::ValueKind;
use surgeist_dom::source::DomSourceEdit;

const TARGET: &[u16] = &ascii(b"popoverTargetElement");
const ACTION: &[reflection::Member] = &[(
    &ascii(b"popoverTargetAction"),
    &ascii(b"get popoverTargetAction"),
    &ascii(b"set popoverTargetAction"),
    &ascii(b"popovertargetaction"),
    Reflection::PopoverTargetAction,
)];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    receiver: Receiver,
) -> Result<(), DocumentGuestEventError> {
    let get = capture(size_of::<(Capture, Receiver)>())?;
    let set = capture(size_of::<(Capture, Receiver)>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| get.read_popover_target(scope, arguments, receiver)),
        FunctionMetadata {
            name: &ascii(b"get popoverTargetElement"),
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            set.write_popover_target(scope, arguments, receiver)
        }),
        FunctionMetadata {
            name: &ascii(b"set popoverTargetElement"),
            length: 1,
        },
    )?;
    define_accessor(scope, prototype, TARGET, &getter, Some(&setter), true)?;
    reflection::install(scope, prototype, capture, receiver, ACTION)
}

impl Capture {
    fn read_popover_target<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let target = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                Ok(Some(node.html_popover_target_with(
                    || read.step(),
                    |bytes| read.reserve(bytes),
                )?))
            })?;
            let Some(target) = target else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            Ok(Completion::Return(self.project_observation(
                scope,
                &owner,
                &events,
                Observation::Node(target),
            )?))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn write_popover_target<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let qualified = self.callbacks.with_read(&owner, &events, |read, _| {
                receiver.matches(read.node(id)?, || read.step())
            })?;
            if !qualified {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let target = match arguments.get(0) {
                None => None,
                Some(value)
                    if matches!(scope.kind(value)?, ValueKind::Null | ValueKind::Undefined) =>
                {
                    None
                }
                Some(value) => {
                    let Some(GuestNode::Element(target)) = tree::projected_node(scope, value)?
                    else {
                        return Ok(Completion::Throw(
                            scope.type_error(&ascii(b"Element required"))?,
                        ));
                    };
                    Some(target)
                }
            };
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_popover_target_element(id, target),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}

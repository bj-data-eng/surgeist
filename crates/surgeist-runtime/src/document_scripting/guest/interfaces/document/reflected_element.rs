//! Live Element-valued reflection for command and popover targets.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_bindings_js::ValueKind;
use surgeist_dom::source::DomSourceEdit;

const ACTION: &[reflection::Member] = &[(
    &ascii(b"popoverTargetAction"),
    &ascii(b"get popoverTargetAction"),
    &ascii(b"set popoverTargetAction"),
    &ascii(b"popovertargetaction"),
    Reflection::PopoverTargetAction,
)];

#[derive(Clone, Copy)]
pub(super) enum ElementTarget {
    CommandFor,
    ButtonPopover,
    InputPopover,
}

impl ElementTarget {
    fn receiver(self) -> Receiver {
        match self {
            Self::CommandFor | Self::ButtonPopover => Receiver::Button,
            Self::InputPopover => Receiver::Input,
        }
    }

    pub(super) fn install<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        let (key, get_name, set_name): (&[u16], &[u16], &[u16]) = match self {
            Self::CommandFor => (
                &ascii(b"commandForElement"),
                &ascii(b"get commandForElement"),
                &ascii(b"set commandForElement"),
            ),
            Self::ButtonPopover | Self::InputPopover => (
                &ascii(b"popoverTargetElement"),
                &ascii(b"get popoverTargetElement"),
                &ascii(b"set popoverTargetElement"),
            ),
        };
        let get = capture(size_of::<(Capture, Self)>())?;
        let set = capture(size_of::<(Capture, Self)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                get.read_reflected_element(scope, arguments, self)
            }),
            FunctionMetadata {
                name: get_name,
                length: 0,
            },
        )?;
        let setter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                set.write_reflected_element(scope, arguments, self)
            }),
            FunctionMetadata {
                name: set_name,
                length: 1,
            },
        )?;
        define_accessor(scope, prototype, key, &getter, Some(&setter), true)?;
        if matches!(self, Self::ButtonPopover | Self::InputPopover) {
            reflection::install(scope, prototype, capture, self.receiver(), ACTION)?;
        }
        Ok(())
    }
}

impl Capture {
    fn read_reflected_element<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: ElementTarget,
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
            let target = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                let step = || read.step();
                let reserve = |bytes| read.reserve(bytes);
                let element = match member {
                    ElementTarget::CommandFor => node.html_command_for_with(step, reserve)?,
                    ElementTarget::ButtonPopover | ElementTarget::InputPopover => {
                        node.html_popover_target_with(step, reserve)?
                    }
                };
                Ok(Some(element))
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

    fn write_reflected_element<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: ElementTarget,
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
            let edit = match member {
                ElementTarget::CommandFor => DomSourceEdit::set_command_for_element(id, target),
                ElementTarget::ButtonPopover | ElementTarget::InputPopover => {
                    DomSourceEdit::set_popover_target_element(id, target)
                }
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }
}

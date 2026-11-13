//! HTMLTemplateElement exposes the canonical inert fragment and reflected attributes.

use super::reflection::{self, Receiver, Reflection};
use super::*;

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"htmlFor"),
        &ascii(b"get htmlFor"),
        &ascii(b"set htmlFor"),
        &ascii(b"for"),
        Reflection::String,
    ),
    (
        &ascii(b"shadowRootMode"),
        &ascii(b"get shadowRootMode"),
        &ascii(b"set shadowRootMode"),
        &ascii(b"shadowrootmode"),
        Reflection::Mode,
    ),
    (
        &ascii(b"shadowRootDelegatesFocus"),
        &ascii(b"get shadowRootDelegatesFocus"),
        &ascii(b"set shadowRootDelegatesFocus"),
        &ascii(b"shadowrootdelegatesfocus"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"shadowRootSerializable"),
        &ascii(b"get shadowRootSerializable"),
        &ascii(b"set shadowRootSerializable"),
        &ascii(b"shadowrootserializable"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"shadowRootSlotAssignment"),
        &ascii(b"get shadowRootSlotAssignment"),
        &ascii(b"set shadowRootSlotAssignment"),
        &ascii(b"shadowrootslotassignment"),
        Reflection::SlotAssignment,
    ),
    (
        &ascii(b"shadowRootClonable"),
        &ascii(b"get shadowRootClonable"),
        &ascii(b"set shadowRootClonable"),
        &ascii(b"shadowrootclonable"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"shadowRootCustomElementRegistry"),
        &ascii(b"get shadowRootCustomElementRegistry"),
        &ascii(b"set shadowRootCustomElementRegistry"),
        &ascii(b"shadowrootcustomelementregistry"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"content"),
        &ascii(b"get content"),
        true,
        Callback::new(move |scope, arguments| handler.template_content(scope, arguments)),
    )?;
    reflection::install(scope, prototype, capture, Receiver::Template, MEMBERS)
}

impl Capture {
    fn template_content<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(receiver)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"HTMLTemplateElement required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let contents = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(receiver)?;
                node.as_template()
                    .map(|template| template.content_fragment_with(|| read.step()))
                    .transpose()
            })?;
            let Some(contents) = contents else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"HTMLTemplateElement required"))?,
                ));
            };
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            Ok(Completion::Return(
                public.project_node(scope, &owner, &events, contents)?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

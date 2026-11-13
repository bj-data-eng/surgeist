//! HTMLTemplateElement reads the canonical inert contents fragment.

use super::*;

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
    Ok(())
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

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
        &ascii(b"defaultView"),
        &ascii(b"get defaultView"),
        true,
        Callback::new(move |scope, arguments| handler.default_view(scope, arguments)),
    )?;
    Ok(())
}

impl Capture {
    fn default_view<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let view = public.document_window(scope, &owner, &events, document)?;
            Ok(Completion::Return(view))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

//! Direct reads of a Frame or Iframe's canonical child navigable.

use super::{reflection::Receiver, *};
use surgeist_html::security::nonce::HtmlBrowsingContextPresence;

#[derive(Clone, Copy)]
enum Member {
    Document,
    Window,
    SvgDocument,
}

enum Readback {
    WrongReceiver,
    Null,
    ChildDocument(DocumentId),
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    receiver: Receiver,
) -> Result<(), DocumentGuestEventError> {
    let getters: [(&[u16], &[u16], Member); 2] = [
        (
            &ascii(b"contentDocument"),
            &ascii(b"get contentDocument"),
            Member::Document,
        ),
        (
            &ascii(b"contentWindow"),
            &ascii(b"get contentWindow"),
            Member::Window,
        ),
    ];
    for (name, getter, member) in getters {
        let handler = capture(size_of::<(Capture, Receiver, Member)>())?;
        install_getter(
            scope,
            prototype,
            name,
            getter,
            true,
            Callback::new(move |scope, arguments| {
                handler.read_frame_content(scope, arguments, receiver, member)
            }),
        )?;
    }
    if matches!(receiver, Receiver::Iframe) {
        let name = &ascii(b"getSVGDocument");
        let handler = capture(size_of::<Capture>())?;
        let operation = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.read_frame_content(scope, arguments, Receiver::Iframe, Member::SvgDocument)
            }),
            FunctionMetadata { name, length: 0 },
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &operation,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn read_frame_content<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let required = receiver.error_message();
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(scope.type_error(required)?));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let readback = self.callbacks.with_read(&owner, &events, |read, _| {
                container_content(read, id, receiver, member)
            })?;
            match readback {
                Readback::WrongReceiver => Ok(Completion::Throw(scope.type_error(required)?)),
                Readback::Null => Ok(Completion::Return(scope.primitive(Primitive::Null)?)),
                Readback::ChildDocument(document) => {
                    let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
                    let value = if matches!(member, Member::Window) {
                        public.document_window(scope, &owner, &events, document)?
                    } else {
                        public.project_node(scope, &owner, &events, document.node())?
                    };
                    Ok(Completion::Return(value))
                }
            }
        })();
        tree::tree_completion(scope, &events, result)
    }
}

fn container_content(
    read: &NativeRead<'_>,
    id: NodeId,
    receiver: Receiver,
    member: Member,
) -> Result<Readback, DocumentGuestEventError> {
    let node = read.node(id)?;
    if !receiver.matches(node, || read.step())? {
        return Ok(Readback::WrongReceiver);
    }
    read.step()?;
    let Some(navigable) = node
        .as_element()
        .and_then(|element| element.child_navigable())
        .ok_or(DomError::WrongNodeKind)?
    else {
        return Ok(Readback::Null);
    };
    let child = read
        .snapshot
        .captured_hosted_navigable_with(navigable, &mut || read.step())?;
    let document = child
        .active_document()
        .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
    if child.container() != Some(id)
        || read
            .snapshot
            .global_document_with(child.global(), &mut || read.step())?
            != document
    {
        return Err(DocumentScriptNativeError::ProjectionCustody.into());
    }
    let DomNodeKind::Document(child) = read.node(document.node())?.kind() else {
        return Err(DomError::WrongDocument.into());
    };
    read.step()?;
    if child.document_environment().browsing_context() != HtmlBrowsingContextPresence::Present
        || !read
            .snapshot
            .hosted_creation_navigable_with(document, &mut || read.step())?
            .is_some_and(|association| association.id() == navigable)
    {
        return Err(DocumentScriptNativeError::ProjectionCustody.into());
    }
    if matches!(member, Member::Window) {
        return Ok(Readback::ChildDocument(document));
    }
    let owner = node.owner_document().ok_or(DomError::WrongDocument)?;
    let DomNodeKind::Document(owner) = read.node(owner.node())?.kind() else {
        return Err(DomError::WrongDocument.into());
    };
    read.step()?;
    if owner.origin() != child.origin() {
        return Ok(Readback::Null);
    }
    if matches!(member, Member::SvgDocument) {
        read.step()?;
        if child.content_type().code_units() == ascii(b"image/svg+xml") {
            // An SVG result also requires the original XML page-load provenance.
            return Err(DocumentGuestEventError::HostRouteUnavailable);
        }
        return Ok(Readback::Null);
    }
    Ok(Readback::ChildDocument(document))
}

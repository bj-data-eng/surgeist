use super::*;
use surgeist_html::construction::HtmlDocumentMode;

const BACK_COMPAT: &[u16] = &ascii(b"BackCompat");
const CSS1_COMPAT: &[u16] = &ascii(b"CSS1Compat");

#[derive(Clone, Copy)]
enum Member {
    Url,
    CompatMode,
    Encoding,
    ContentType,
}

enum Text {
    Usv(DocumentString),
    Native(DocumentString),
    Literal(&'static [u16]),
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (member, name, getter) in [
        (Member::Url, &ascii(b"URL")[..], &ascii(b"get URL")[..]),
        (
            Member::Url,
            &ascii(b"documentURI"),
            &ascii(b"get documentURI"),
        ),
        (
            Member::CompatMode,
            &ascii(b"compatMode"),
            &ascii(b"get compatMode"),
        ),
        (
            Member::Encoding,
            &ascii(b"characterSet"),
            &ascii(b"get characterSet"),
        ),
        (Member::Encoding, &ascii(b"charset"), &ascii(b"get charset")),
        (
            Member::Encoding,
            &ascii(b"inputEncoding"),
            &ascii(b"get inputEncoding"),
        ),
        (
            Member::ContentType,
            &ascii(b"contentType"),
            &ascii(b"get contentType"),
        ),
    ] {
        let handler = capture(size_of::<(Capture, Member)>())?;
        install_getter(
            scope,
            prototype,
            name,
            getter,
            true,
            Callback::new(move |scope, arguments| {
                handler.document_metadata(scope, arguments, member)
            }),
        )?;
    }
    Ok(())
}

impl Capture {
    fn document_metadata<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
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
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let DomNodeKind::Document(metadata) = read.node(document.node())?.kind() else {
                    return Err(DomError::WrongNodeKind.into());
                };
                Ok(match member {
                    Member::Url => Text::Usv(metadata.url().clone()),
                    Member::CompatMode => Text::Literal(match metadata.mode() {
                        HtmlDocumentMode::Quirks => BACK_COMPAT,
                        HtmlDocumentMode::NoQuirks | HtmlDocumentMode::LimitedQuirks => CSS1_COMPAT,
                    }),
                    Member::Encoding => Text::Native(metadata.encoding().clone()),
                    Member::ContentType => Text::Native(metadata.content_type().clone()),
                })
            })?;
            let value = match value {
                Text::Usv(value) => self
                    .callbacks
                    .conversion
                    .run(scope, |conversion| {
                        conversion.copy_usv_to_js(value.code_units())
                    })
                    .map_err(Error::from)?,
                Text::Native(value) => self.copy_dom_units(scope, value.code_units())?,
                Text::Literal(value) => self.copy_dom_units(scope, value)?,
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

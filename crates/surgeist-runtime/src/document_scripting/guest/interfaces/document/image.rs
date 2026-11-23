//! Readbacks over the exact live native Image request.

use super::{reflection::html_element_is, *};

#[derive(Clone, Copy)]
enum RequestRead {
    NaturalWidth,
    NaturalHeight,
    Complete,
    CurrentSrc,
}

enum Readback {
    Unsigned(u32),
    Boolean(bool),
    Url(DocumentString),
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let accessors: [(&[u16], &[u16], RequestRead); 4] = [
        (
            &ascii(b"naturalWidth"),
            &ascii(b"get naturalWidth"),
            RequestRead::NaturalWidth,
        ),
        (
            &ascii(b"naturalHeight"),
            &ascii(b"get naturalHeight"),
            RequestRead::NaturalHeight,
        ),
        (
            &ascii(b"complete"),
            &ascii(b"get complete"),
            RequestRead::Complete,
        ),
        (
            &ascii(b"currentSrc"),
            &ascii(b"get currentSrc"),
            RequestRead::CurrentSrc,
        ),
    ];
    for (name, function_name, readback) in accessors {
        let handler = capture(size_of::<(Capture, RequestRead)>())?;
        install_getter(
            scope,
            prototype,
            name,
            function_name,
            true,
            Callback::new(move |scope, arguments| {
                handler.image_request(scope, arguments, readback)
            }),
        )?;
    }
    Ok(())
}

impl Capture {
    fn image_request<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        readback: RequestRead,
    ) -> Result<Completion<'scope>, Error> {
        let required = &ascii(b"HTMLImageElement required");
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(scope.type_error(required)?));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                read.step()?;
                if !html_element_is(node, "HTMLImageElement") {
                    return Ok(None);
                }
                let image = node.as_img().ok_or(DomError::WrongNodeKind)?;
                let state = image.state();
                let request = state.current_request();
                let value = match readback {
                    RequestRead::NaturalWidth => {
                        let [width, _] = request.natural_dimensions();
                        Readback::Unsigned(width)
                    }
                    RequestRead::NaturalHeight => {
                        let [_, height] = request.natural_dimensions();
                        Readback::Unsigned(height)
                    }
                    RequestRead::Complete => {
                        let attrs = node.attributes().ok_or(DomError::WrongNodeKind)?;
                        let src = attrs
                            .get_null_namespace_attribute_with(&ascii(b"src"), || read.step())?;
                        let srcset = attrs
                            .get_null_namespace_attribute_with(&ascii(b"srcset"), || read.step())?;
                        Readback::Boolean(state.complete_with_attributes(
                            src.as_ref().map(|attribute| attribute.value().code_units()),
                            srcset.is_some(),
                        ))
                    }
                    RequestRead::CurrentSrc => Readback::Url(request.current_url().clone()),
                };
                Ok(Some(value))
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(scope.type_error(required)?));
            };
            let value = match value {
                Readback::Unsigned(value) => {
                    scope.primitive(Primitive::Number(f64::from(value)))?
                }
                Readback::Boolean(value) => scope.primitive(Primitive::Boolean(value))?,
                Readback::Url(url) => self
                    .callbacks
                    .conversion
                    .run(scope, |conversion| {
                        conversion.copy_usv_to_js(url.code_units())
                    })
                    .map_err(Error::from)?,
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

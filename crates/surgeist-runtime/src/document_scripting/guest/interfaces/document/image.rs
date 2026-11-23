//! Local reflections and live readbacks on the original native Image.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const BEFORE_REQUEST: &[reflection::Member] = &[
    (
        &ascii(b"alt"),
        &ascii(b"get alt"),
        &ascii(b"set alt"),
        &ascii(b"alt"),
        Reflection::String,
    ),
    (
        &ascii(b"srcset"),
        &ascii(b"get srcset"),
        &ascii(b"set srcset"),
        &ascii(b"srcset"),
        Reflection::UsvString,
    ),
    (
        &ascii(b"sizes"),
        &ascii(b"get sizes"),
        &ascii(b"set sizes"),
        &ascii(b"sizes"),
        Reflection::String,
    ),
    (
        &ascii(b"crossOrigin"),
        &ascii(b"get crossOrigin"),
        &ascii(b"set crossOrigin"),
        &ascii(b"crossorigin"),
        Reflection::CrossOrigin,
    ),
    (
        &ascii(b"useMap"),
        &ascii(b"get useMap"),
        &ascii(b"set useMap"),
        &ascii(b"usemap"),
        Reflection::String,
    ),
    (
        &ascii(b"isMap"),
        &ascii(b"get isMap"),
        &ascii(b"set isMap"),
        &ascii(b"ismap"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"controls"),
        &ascii(b"get controls"),
        &ascii(b"set controls"),
        &ascii(b"controls"),
        Reflection::Boolean,
    ),
];

const AFTER_REQUEST: &[reflection::Member] = &[
    (
        &ascii(b"referrerPolicy"),
        &ascii(b"get referrerPolicy"),
        &ascii(b"set referrerPolicy"),
        &ascii(b"referrerpolicy"),
        Reflection::ReferrerPolicy,
    ),
    (
        &ascii(b"decoding"),
        &ascii(b"get decoding"),
        &ascii(b"set decoding"),
        &ascii(b"decoding"),
        Reflection::Decoding,
    ),
    (
        &ascii(b"loading"),
        &ascii(b"get loading"),
        &ascii(b"set loading"),
        &ascii(b"loading"),
        Reflection::Loading,
    ),
    (
        &ascii(b"fetchPriority"),
        &ascii(b"get fetchPriority"),
        &ascii(b"set fetchPriority"),
        &ascii(b"fetchpriority"),
        Reflection::FetchPriority,
    ),
    (
        &ascii(b"name"),
        &ascii(b"get name"),
        &ascii(b"set name"),
        &ascii(b"name"),
        Reflection::String,
    ),
    (
        &ascii(b"align"),
        &ascii(b"get align"),
        &ascii(b"set align"),
        &ascii(b"align"),
        Reflection::String,
    ),
    (
        &ascii(b"hspace"),
        &ascii(b"get hspace"),
        &ascii(b"set hspace"),
        &ascii(b"hspace"),
        Reflection::UnsignedLong,
    ),
    (
        &ascii(b"vspace"),
        &ascii(b"get vspace"),
        &ascii(b"set vspace"),
        &ascii(b"vspace"),
        Reflection::UnsignedLong,
    ),
    (
        &ascii(b"border"),
        &ascii(b"get border"),
        &ascii(b"set border"),
        &ascii(b"border"),
        Reflection::StringNullToEmpty,
    ),
];

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
    reflection::install(scope, prototype, capture, Receiver::Image, BEFORE_REQUEST)?;
    install_request(scope, prototype, capture)?;
    reflection::install(scope, prototype, capture, Receiver::Image, AFTER_REQUEST)
}

fn install_request<'scope>(
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
        let receiver = Receiver::Image;
        let required = receiver.error_message();
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(scope.type_error(required)?));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
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

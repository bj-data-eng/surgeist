//! Title accessors over native direct child text and Source replacement.

use super::{element::HTML_NAMESPACE, text::TextBuffer, *};
use surgeist_bindings_js::conversion::{
    declarations::html_title_element_text_setter, types::DOM_STRING,
};
use surgeist_dom::{source::DomSourceEdit, tree::DomNode};

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let read = capture(size_of::<Capture>())?;
    let write = capture(size_of::<Capture>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| read.title_text(scope, arguments, false)),
        FunctionMetadata {
            name: &ascii(b"get text"),
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| write.title_text(scope, arguments, true)),
        FunctionMetadata {
            name: &ascii(b"set text"),
            length: 1,
        },
    )?;
    scope.define_new_property(
        prototype,
        PropertyKey::String(&ascii(b"text")),
        PropertyDescriptor::Accessor {
            get: Some(&getter),
            set: Some(&setter),
            enumerable: true,
            configurable: true,
        },
    )?;
    Ok(())
}

pub(super) fn install_document<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let read = capture(size_of::<Capture>())?;
    let write = capture(size_of::<Capture>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| read.document_title_get(scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"get title"),
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| write.document_title_set(scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"set title"),
            length: 1,
        },
    )?;
    scope.define_new_property(
        prototype,
        PropertyKey::String(&ascii(b"title")),
        PropertyDescriptor::Accessor {
            get: Some(&getter),
            set: Some(&setter),
            enumerable: true,
            configurable: true,
        },
    )?;
    Ok(())
}

impl Capture {
    fn document_title_get<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let text = self.callbacks.with_read(&owner, &events, |read, _| {
                let mut text = TextBuffer::default();
                read.snapshot.visit_document_title_with(
                    id,
                    || read.step(),
                    |bytes| read.reserve(bytes),
                    |unit| text.append_units(read, &[unit]),
                )?;
                Ok(text)
            })?;
            Ok(Completion::Return(
                self.copy_dom_units(scope, text.code_units())?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn document_title_set<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            self.callbacks.with_read(&owner, &events, |read, _| {
                read.node(id.node())?;
                Ok(())
            })?;
            let absent;
            let input = match arguments.get(0) {
                Some(value) => value,
                None => {
                    absent = scope.primitive(Primitive::Undefined)?;
                    &absent
                }
            };
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.value(&DOM_STRING, input))
                .map_err(Error::from)?;
            scope.check_active()?;
            let (value, _backing) =
                copy_native_data(&owner, &events, converted.get().as_code_units())?;
            // Source selects the target from the current tree after coercion.
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_document_title(id, value),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn title_text<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        write: bool,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"HTMLTitleElement required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let text = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !is_title(node) {
                    return Ok(None);
                }
                let mut text = TextBuffer::default();
                if !write {
                    node.visit_child_text_with(|| read.step(), |data| text.append(read, data))?;
                }
                Ok(Some(text))
            })?;
            let Some(text) = text else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"HTMLTitleElement required"))?,
                ));
            };
            if !write {
                return Ok(Completion::Return(
                    self.copy_dom_units(scope, text.code_units())?,
                ));
            }
            let absent;
            let input = match arguments.get(0) {
                Some(value) => value,
                None => {
                    absent = scope.primitive(Primitive::Undefined)?;
                    &absent
                }
            };
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.value(&html_title_element_text_setter(), input)
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let (value, _backing) =
                copy_native_data(&owner, &events, converted.get().as_code_units())?;
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_text_content(id, value),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}

fn is_title(node: DomNode<'_>) -> bool {
    node.as_element().is_some_and(|element| {
        let name = element.name();
        !element.has_failed_construction_interface()
            && name.local_name().code_units() == ascii(b"title")
            && name
                .namespace_uri()
                .is_some_and(|uri| uri.code_units() == HTML_NAMESPACE)
    })
}

//! Node value and ordinary descendant text accessors.

use super::{character_data::copy_native_data, *};
use surgeist_bindings_js::conversion::types::DOM_STRING;
use surgeist_dom::DomSourceEdit;

pub(super) fn install_whole_text<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"wholeText"),
        &ascii(b"get wholeText"),
        true,
        Callback::new(move |scope, arguments| handler.whole_text(scope, arguments)),
    )?;
    Ok(())
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (key, get_name, set_name, content) in [
        (
            &ascii(b"nodeValue")[..],
            &ascii(b"get nodeValue")[..],
            &ascii(b"set nodeValue")[..],
            false,
        ),
        (
            &ascii(b"textContent")[..],
            &ascii(b"get textContent")[..],
            &ascii(b"set textContent")[..],
            true,
        ),
    ] {
        let getter = capture(size_of::<(Capture, bool)>())?;
        let get = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                getter.read_node_text(scope, arguments, content)
            }),
            FunctionMetadata {
                name: get_name,
                length: 0,
            },
        )?;
        let setter = capture(size_of::<(Capture, bool)>())?;
        let set = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                setter.write_node_text(scope, arguments, content)
            }),
            FunctionMetadata {
                name: set_name,
                length: 1,
            },
        )?;
        scope.define_new_property(
            prototype,
            PropertyKey::String(key),
            PropertyDescriptor::Accessor {
                get: Some(&get),
                set: Some(&set),
                enumerable: true,
                configurable: true,
            },
        )?;
    }
    Ok(())
}

impl Capture {
    fn whole_text<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Text(node)) = tree::projected_node(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Text required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(node)?;
                let mut text = TextBuffer::default();
                node.visit_contiguous_text_with(|| read.step(), |data| text.append(read, data))?;
                Ok(text)
            })?;
            self.copy_dom_units(scope, &value.units)
        })();
        complete(scope, &events, result)
    }

    fn read_node_text<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        content: bool,
    ) -> Result<Completion<'scope>, Error> {
        let Some(projection) = scope.native_object::<GuestNodeProjection>(arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let id = projection
                .payload()
                .id()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id.node())?;
                id.node_type(node.kind())?;
                if content
                    && matches!(
                        id,
                        GuestNode::Fragment(_) | GuestNode::ShadowRoot(_) | GuestNode::Element(_)
                    )
                {
                    return collect_text(read, node).map(TextValue::Descendants);
                }
                let value = node
                    .as_attr()
                    .map(|attr| attr.value())
                    .or_else(|| node.data());
                Ok(TextValue::Single(value.cloned()))
            })?;
            match value {
                TextValue::Single(value) => {
                    self.project_observation(scope, &owner, &events, Observation::String(value))
                }
                TextValue::Descendants(buffer) => self.copy_dom_units(scope, &buffer.units),
            }
        })();
        complete(scope, &events, result)
    }

    fn write_node_text<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        content: bool,
    ) -> Result<Completion<'scope>, Error> {
        let Some(projection) = scope.native_object::<GuestNodeProjection>(arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
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
                .run(scope, |cx| cx.value(&DOM_STRING.nullable(), input))
                .map_err(Error::from)?;
            scope.check_active()?;
            let id = projection
                .payload()
                .id()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let attribute = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id.node())?;
                id.node_type(node.kind())?;
                Ok(node.as_attr().map(|attr| attr.id()))
            })?;
            if matches!(id, GuestNode::Document(_))
                || (!content
                    && matches!(
                        id,
                        GuestNode::Fragment(_) | GuestNode::ShadowRoot(_) | GuestNode::Element(_)
                    ))
            {
                return Ok(Completion::Undefined);
            }
            let units = converted
                .get()
                .as_ref()
                .map_or(&[][..], |value| value.as_code_units());
            let (value, _backing) = copy_native_data(&owner, &events, units)?;
            let edit = match (content, attribute) {
                (false, Some(attribute)) => DomSourceEdit::set_attribute_value(attribute, value),
                (false, None) => DomSourceEdit::replace_data(id.node(), 0, usize::MAX, value),
                (true, _) => DomSourceEdit::set_text_content(id.node(), value),
            };
            self.apply_attribute(
                scope,
                &owner,
                &events,
                edit,
                None,
                element::AttributeResult::Value,
            )
        })();
        match result {
            Ok(completion) => Ok(completion),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }
}

enum TextValue {
    Single(Option<DocumentString>),
    Descendants(TextBuffer),
}

#[derive(Default)]
struct TextBuffer {
    units: Vec<u16>,
    _backing: Option<GuestBacking>,
    _source_units: Option<NativeTextUnits>,
}

impl TextBuffer {
    fn append(
        &mut self,
        read: &NativeRead<'_>,
        data: &DocumentString,
    ) -> Result<(), DocumentGuestEventError> {
        let length = self
            .units
            .len()
            .checked_add(data.len())
            .ok_or(Error::StringTooLong)?;
        if length > self.units.capacity() {
            let capacity = length.max(self.units.capacity().saturating_mul(2));
            let bytes = capacity
                .checked_mul(size_of::<u16>())
                .ok_or(Error::StringTooLong)?;
            let backing = read.reserve(bytes)?;
            let source_units = read.retain_text_units(capacity)?;
            let mut units = Vec::new();
            units
                .try_reserve_exact(capacity)
                .map_err(|_| Error::Allocation)?;
            if units.capacity() != capacity {
                return Err(Error::Allocation.into());
            }
            for &unit in &self.units {
                read.step()?;
                units.push(unit);
            }
            *self = Self {
                units,
                _backing: Some(backing),
                _source_units: source_units,
            };
        }
        for &unit in data.code_units() {
            read.step()?;
            self.units.push(unit);
        }
        Ok(())
    }
}

fn collect_text(
    read: &NativeRead<'_>,
    node: surgeist_dom::DomNode<'_>,
) -> Result<TextBuffer, DocumentGuestEventError> {
    let mut text = TextBuffer::default();
    node.visit_descendant_text_with(
        || read.step(),
        |bytes| read.reserve(bytes),
        |data| text.append(read, data),
    )?;
    Ok(text)
}

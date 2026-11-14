//! Attr getters over the canonical DOM.

use super::*;
use character_data::copy_native_data;
use surgeist_dom::DomSourceEdit;

const ATTR: &[u16] = &ascii(b"Attr");

#[derive(Clone, Copy)]
enum AttrMember {
    Namespace,
    Prefix,
    LocalName,
    Name,
    Value,
    OwnerElement,
    Specified,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    node: &Value<'scope>,
    node_constructor: &Value<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
    let (_, prototype) = install_derived_interface(
        scope,
        node,
        node_constructor,
        FunctionMetadata {
            name: ATTR,
            length: 0,
        },
        &illegal_constructor(funding.capture)?,
    )?;
    for (member, key, name) in [
        (
            AttrMember::Namespace,
            &ascii(b"namespaceURI")[..],
            &ascii(b"get namespaceURI")[..],
        ),
        (
            AttrMember::Prefix,
            &ascii(b"prefix")[..],
            &ascii(b"get prefix")[..],
        ),
        (
            AttrMember::LocalName,
            &ascii(b"localName")[..],
            &ascii(b"get localName")[..],
        ),
        (
            AttrMember::Name,
            &ascii(b"name")[..],
            &ascii(b"get name")[..],
        ),
        (
            AttrMember::Value,
            &ascii(b"value")[..],
            &ascii(b"get value")[..],
        ),
        (
            AttrMember::OwnerElement,
            &ascii(b"ownerElement")[..],
            &ascii(b"get ownerElement")[..],
        ),
        (
            AttrMember::Specified,
            &ascii(b"specified")[..],
            &ascii(b"get specified")[..],
        ),
    ] {
        let capture = (funding.capture)(size_of::<(Capture, AttrMember)>())?;
        let get = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| capture.attr_member(scope, arguments, member)),
            FunctionMetadata { name, length: 0 },
        )?;
        let set = if matches!(member, AttrMember::Value) {
            let setter = (funding.capture)(size_of::<Capture>())?;
            Some(scope.fixed_arity_function(
                &Callback::new(move |scope, arguments| set_value(scope, arguments, &setter)),
                FunctionMetadata {
                    name: &ascii(b"set value"),
                    length: 1,
                },
            )?)
        } else {
            None
        };
        scope.define_new_property(
            &prototype,
            PropertyKey::String(key),
            PropertyDescriptor::Accessor {
                get: Some(&get),
                set: set.as_ref(),
                enumerable: true,
                configurable: true,
            },
        )?;
    }
    let observed = scope.callback_object(&prototype)?;
    scope.retain_callback_object(observed, funding.retain)
}

impl Capture {
    fn attr_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: AttrMember,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Attr(id)) = receiver.and_then(|node| node.payload().id()) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Attr required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let observation = self.callbacks.with_read(&owner, &events, |read, _| {
                let attribute = read
                    .node(id)?
                    .as_attr()
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                let name = attribute.name();
                Ok(match member {
                    AttrMember::Namespace => Observation::String(name.namespace_uri().cloned()),
                    AttrMember::Prefix => Observation::String(name.prefix().cloned()),
                    AttrMember::LocalName => Observation::String(Some(name.local_name().clone())),
                    AttrMember::Name => Observation::AttrName(name.clone()),
                    AttrMember::Value => Observation::String(Some(attribute.value().clone())),
                    AttrMember::OwnerElement => Observation::Node(attribute.owner_element()),
                    AttrMember::Specified => Observation::Boolean(true),
                })
            })?;
            self.project_observation(scope, &owner, &events, observation)
        })();
        complete(scope, &events, result)
    }

    pub(super) fn copy_attr_name<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        name: &DomQualifiedName,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let local = name.local_name();
        let Some(prefix) = name.prefix() else {
            return self.copy_dom_units(scope, local.code_units());
        };
        let length = prefix
            .len()
            .checked_add(1)
            .and_then(|length| length.checked_add(local.len()))
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let units = prefix
            .code_units()
            .iter()
            .copied()
            .chain([u16::from(b':')])
            .chain(local.code_units().iter().copied());
        self.copy_name_units(scope, owner, events, length, units)
    }
}

pub(super) fn set_value<'scope>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
    capture: &Capture,
) -> Result<Completion<'scope>, Error> {
    let projection = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
    let Some(GuestNode::Attr(node)) = projection.and_then(|node| node.payload().id()) else {
        return Ok(Completion::Throw(
            scope.type_error(&ascii(b"Attr required"))?,
        ));
    };
    let (owner, events) = capture.active(scope)?;
    let result = (|| -> Result<_, DocumentGuestEventError> {
        let absent;
        let input = match arguments.get(0) {
            Some(value) => value,
            None => {
                absent = scope.primitive(Primitive::Undefined)?;
                &absent
            }
        };
        let value = capture
            .callbacks
            .conversion
            .run(scope, |conversion| {
                conversion.value(&surgeist_bindings_js::conversion::types::DOM_STRING, input)
            })
            .map_err(Error::from)?;
        scope.check_active()?;
        let attribute = capture.callbacks.with_read(&owner, &events, |read, _| {
            let attr = read
                .node(node)?
                .as_attr()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            Ok(attr.id())
        })?;
        let (value, _backing) = copy_native_data(&owner, &events, value.get().as_code_units())?;
        capture.apply_attribute(
            scope,
            &owner,
            &events,
            DomSourceEdit::set_attribute_value(attribute, value),
            None,
            element::AttributeResult::Value,
        )
    })();
    match result {
        Ok(completion) => Ok(completion),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}

//! CDATASection exposure through the original detached-node creation owner.

use super::*;
use surgeist_bindings_js::{
    conversion::declarations::document_create_cdata_section_0_data, dom_exception::DomExceptionName,
};

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    (text_constructor, text_prototype): (&Value<'scope>, &Value<'scope>),
    funding: &OriginInterfaceFunding<'_>,
) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
    let (_, prototype) = install_derived_interface(
        scope,
        text_prototype,
        text_constructor,
        FunctionMetadata {
            name: &ascii(b"CDATASection"),
            length: 0,
        },
        &illegal_constructor(funding.capture)?,
    )?;
    let handler = (funding.capture)(size_of::<Capture>())?;
    let name = &ascii(b"createCDATASection");
    let factory = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.create_cdata_section(scope, arguments)),
        FunctionMetadata { name, length: 1 },
    )?;
    define_data(
        scope,
        document,
        PropertyKey::String(name),
        &factory,
        true,
        true,
        true,
    )?;
    let observed = scope.callback_object(&prototype)?;
    scope.retain_callback_object(observed, funding.retain)
}

impl Capture {
    fn create_cdata_section<'scope>(
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
        let Some(input) = arguments.get(0) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"CDATASection data required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let descriptor = document_create_cdata_section_0_data();
            let converted = self.character_argument(scope, &descriptor, Some(input))?;
            scope.check_active()?;
            let ArgumentValue::Present(data) = converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let (data, _backing) = copy_native_data(&owner, &events, data.as_code_units())?;
            let creation = DomDetachedNodeCreation::CdataSection {
                data,
                fallback: creation_realm_fallback(&owner)?,
            };
            let publish = |scope: &mut Scope<'scope>, prototypes: &DocumentPrototypes| {
                public.document.ordinary.create_detached(
                    scope,
                    self,
                    (&owner, &events),
                    ordinary::NodePrototype::Creation(&prototypes.character_data.cdata_section),
                    (document, creation),
                )
            };
            if let Some(global) = public.other_document_origin(&owner, &events, document)? {
                public.with_origin(&events, global, |origin| publish(scope, &origin.documents))
            } else {
                publish(scope, &public.document.prototypes)
            }
        })();
        let exception = match &result {
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::InvalidCharacter),
            ))) => Some(DomExceptionName::InvalidCharacterError),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::NotSupported),
            ))) => Some(DomExceptionName::NotSupportedError),
            _ => None,
        };
        if let Some(name) = exception {
            Ok(Completion::Throw(scope.dom_exception(name, &[])?))
        } else {
            complete(scope, &events, result)
        }
    }
}

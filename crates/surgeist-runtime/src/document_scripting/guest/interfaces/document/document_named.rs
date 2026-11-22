//! Legacy named properties on the exact canonical Document wrapper.

use super::tag_collection::{Keys, key_limits};
use super::*;
use surgeist_bindings_js::NativeNamedPropertyCallbacks;
use surgeist_dom::tree::DomDocumentNamedValue;

const UNFORGEABLE_NAMES: &[&[u16]] = &[&ascii(b"location")];

pub(super) struct Projection {
    read: RetainedCallbackObject,
    keys: RetainedCallbackObject,
}

impl Projection {
    pub(super) fn open<'scope>(
        &self,
        scope: &mut Scope<'scope>,
    ) -> Result<(Value<'scope>, Value<'scope>), Error> {
        Ok((
            scope.open_callback_object(&self.read)?,
            scope.open_callback_object(&self.keys)?,
        ))
    }
}

pub(super) fn callbacks<'value, 'scope>(
    read: &'value Value<'scope>,
    keys: &'value Value<'scope>,
) -> NativeNamedPropertyCallbacks<'value, 'scope> {
    NativeNamedPropertyCallbacks {
        read,
        keys,
        unforgeable_names: UNFORGEABLE_NAMES,
    }
}

pub(super) fn install(
    scope: &mut Scope<'_>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<Projection, DocumentGuestEventError> {
    let handler = (funding.capture)(size_of::<Capture>())?;
    let read = scope.fixed_arity_function(
        &Callback::new(move |scope, args| handler.document_named_property(scope, args)),
        FunctionMetadata {
            name: &[],
            length: 2,
        },
    )?;
    let handler = (funding.capture)(size_of::<Capture>())?;
    let keys = scope.fixed_arity_function(
        &Callback::new(move |scope, args| handler.document_named_keys(scope, args)),
        FunctionMetadata {
            name: &[],
            length: 0,
        },
    )?;
    let read = scope.callback_object(&read)?;
    let read = scope.retain_callback_object(read, funding.retain)?;
    let keys = scope.callback_object(&keys)?;
    let keys = scope.retain_callback_object(keys, funding.retain)?;
    Ok(Projection { read, keys })
}

impl Capture {
    fn document_named_property<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        args: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = args.receiver();
        let Some(GuestNode::Document(document)) = tree::projected_node(scope, receiver)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let key = args.get(0).ok_or(Error::BackendUnavailable)?;
            let name = scope.dom_string(key)?;
            let units = name.as_code_units();
            let (name, backing) = character_data::copy_native_data(&owner, &events, units)?;
            scope.check_active()?;
            if args.get(1).is_some() {
                let present = self.callbacks.with_read(&owner, &events, |read, _| {
                    let dom = read
                        .dom
                        .try_borrow()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    dom.raw().document_has_named_property(
                        document,
                        &name,
                        events.limits.work,
                        || events.budget.charge_work(read.account_scope, 1),
                        read.cancel,
                    )
                })?;
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Boolean(present))?,
                ));
            }
            let resolved = self.callbacks.with_read(&owner, &events, |read, _| {
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                dom.raw().document_named_value(
                    document,
                    name,
                    units.len(),
                    events.limits.work,
                    || events.budget.charge_work(read.account_scope, 1),
                    read.cancel,
                )
            })?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            match resolved {
                DomDocumentNamedValue::Absent => Ok(Completion::Undefined),
                DomDocumentNamedValue::Element(node) => Ok(Completion::Return(
                    public.project_node(scope, &owner, &events, node)?,
                )),
                DomDocumentNamedValue::Child(global) => {
                    let proxy = public
                        .child_window(scope, &owner, &events, global)?
                        .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                    Ok(Completion::Return(proxy))
                }
                DomDocumentNamedValue::Collection(collection) => {
                    let origin = public.other_document_origin(&owner, &events, document)?;
                    let value =
                        public.with_collection_projection(&events, origin, |projection| {
                            projection.project_for_root(
                                scope,
                                (&owner, &events, collection),
                                receiver,
                                None,
                                Some(backing),
                            )
                        })?;
                    Ok(Completion::Return(value))
                }
            }
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn document_named_keys<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        args: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) = tree::projected_node(scope, args.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let keys = self.callbacks.with_read(&owner, &events, |read, _| {
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                dom.raw().document_supported_names(
                    document,
                    key_limits(&events, true),
                    || events.budget.charge_work(read.account_scope, 1),
                    read.cancel,
                )
            })?;
            self.project_collection_keys(scope, &owner, &events, Keys::Names(keys))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

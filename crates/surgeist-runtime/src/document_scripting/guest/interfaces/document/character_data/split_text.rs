use super::*;
use surgeist_bindings_js::{IntegerValue, conversion::declarations};
use surgeist_dom::{DomSourceEdit, DomSourceOutput};

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let name = &ascii(b"splitText");
    let handler = capture(size_of::<Capture>())?;
    let callback = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.split_text(scope, arguments)),
        FunctionMetadata { name, length: 1 },
    )?;
    define_data(
        scope,
        prototype,
        PropertyKey::String(name),
        &callback,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn split_text<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Text(node)) = tree::projected_node(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Text required"))?,
            ));
        };
        if arguments.is_empty() {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing text offset"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let descriptor = declarations::text_split_text_0_offset();
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&descriptor, arguments.get(descriptor.index()))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let ArgumentValue::Present(IntegerValue::UnsignedLong(offset)) = converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let offset = usize::try_from(*offset)
                .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
            let origin = self
                .callbacks
                .with_read(&owner, &events, |read, attached| {
                    let document = read
                        .node(node)?
                        .owner_document()
                        .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                    read.step()?;
                    let origin = read
                        .node(document.node())?
                        .relevant_global()
                        .map_err(DomHostedRuntimeError::Dom)
                        .map_err(DocumentScriptNativeError::from)?;
                    Ok(origin.filter(|origin| *origin != attached))
                })?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let realm = scope.realm_authority()?.identity();
            let mut driven = None;
            let mut project = |prototypes: &DocumentPrototypes| {
                public.document.ordinary.project_source_node(
                    scope,
                    (&owner, &events),
                    prototypes.character_data.prototype(CharacterKind::Text),
                    |output| {
                        let drive = self.drive_guest_watched_node(
                            (&owner, &events),
                            realm,
                            DomSourceEdit::split_text(node, offset),
                            output,
                        )?;
                        let node = match drive.terminal_output() {
                            Some(DomSourceOutput::Node(node)) => Some(GuestNode::Text(node)),
                            _ => None,
                        };
                        driven = Some(drive);
                        node.ok_or_else(|| DocumentScriptNativeError::ProjectionCustody.into())
                    },
                )
            };
            let projection = if let Some(global) = origin {
                public.with_origin(&events, global, |origin| project(&origin.documents))
            } else {
                project(&public.document.prototypes)
            };
            let Some(drive) = driven else {
                return projection.map(Completion::Return);
            };
            self.finish_guest_tree_edit(
                scope,
                &owner,
                &events,
                drive.progress,
                (drive.parent, drive.pending),
                |_, _| projection.map(Completion::Return),
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}

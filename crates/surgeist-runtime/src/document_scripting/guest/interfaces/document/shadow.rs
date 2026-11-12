//! Canonical ShadowRoot interface and its native host relationship.

use super::*;
use surgeist_dom::{HtmlShadowRootMode, HtmlSlotAssignment};

const OPEN: &[u16] = &ascii(b"open");
const CLOSED: &[u16] = &ascii(b"closed");
const MANUAL: &[u16] = &ascii(b"manual");
const NAMED: &[u16] = &ascii(b"named");

#[derive(Clone, Copy)]
enum Member {
    Mode,
    DelegatesFocus,
    Serializable,
    SlotAssignment,
    Clonable,
    Host,
}

enum Observation {
    Host(NodeId),
    Boolean(bool),
    Keyword(&'static [u16]),
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    fragment_constructor: &Value<'scope>,
    fragment: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let (_, prototype) = install_derived_node(
        scope,
        fragment,
        fragment_constructor,
        &ascii(b"ShadowRoot"),
        &illegal_node_constructor(),
    )?;
    for (member, key, name) in [
        (Member::Mode, &ascii(b"mode")[..], &ascii(b"get mode")[..]),
        (
            Member::DelegatesFocus,
            &ascii(b"delegatesFocus")[..],
            &ascii(b"get delegatesFocus")[..],
        ),
        (
            Member::Serializable,
            &ascii(b"serializable")[..],
            &ascii(b"get serializable")[..],
        ),
        (
            Member::SlotAssignment,
            &ascii(b"slotAssignment")[..],
            &ascii(b"get slotAssignment")[..],
        ),
        (
            Member::Clonable,
            &ascii(b"clonable")[..],
            &ascii(b"get clonable")[..],
        ),
        (Member::Host, &ascii(b"host")[..], &ascii(b"get host")[..]),
    ] {
        let handler = capture(size_of::<(Capture, Member)>())?;
        install_getter(
            scope,
            &prototype,
            key,
            name,
            true,
            Callback::new(move |scope, arguments| handler.shadow_member(scope, arguments, member)),
        )?;
    }
    Ok(prototype)
}

impl Capture {
    fn shadow_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::ShadowRoot(root)) = receiver.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"ShadowRoot required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let observation = self.with_native(&owner, |session, _| {
                let snapshot = session.snapshot();
                let node = snapshot.node_with(root, || events.step())?;
                let DomNodeKind::ShadowRoot(state) = node.kind() else {
                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                };
                events.step()?;
                Ok(match member {
                    Member::Mode => Observation::Keyword(match state.mode() {
                        HtmlShadowRootMode::Open => OPEN,
                        HtmlShadowRootMode::Closed => CLOSED,
                    }),
                    Member::DelegatesFocus => Observation::Boolean(state.delegates_focus()),
                    Member::Serializable => Observation::Boolean(state.serializable()),
                    Member::SlotAssignment => Observation::Keyword(match state.slot_assignment() {
                        HtmlSlotAssignment::Manual => MANUAL,
                        HtmlSlotAssignment::Named => NAMED,
                    }),
                    Member::Clonable => Observation::Boolean(state.clonable()),
                    Member::Host => Observation::Host(
                        node.shadow_host()
                            .ok_or(DocumentScriptNativeError::ProjectionCustody)?,
                    ),
                })
            })?;
            match observation {
                Observation::Host(host) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_node(scope, &owner, &events, host),
                Observation::Boolean(value) => Ok(scope.primitive(Primitive::Boolean(value))?),
                Observation::Keyword(value) => Ok(scope.string(value)?),
            }
        })();
        complete(scope, &events, result)
    }
}

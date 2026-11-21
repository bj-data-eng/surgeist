//! Shared HTMLMediaElement readbacks from the canonical unloaded state.

use super::{reflection::Receiver, *};

const NETWORK_EMPTY: u16 = 0;
const HAVE_NOTHING: u16 = 0;
const CONSTANTS: &[(&[u16], u16)] = &[
    (&ascii(b"NETWORK_EMPTY"), NETWORK_EMPTY),
    (&ascii(b"NETWORK_IDLE"), 1),
    (&ascii(b"NETWORK_LOADING"), 2),
    (&ascii(b"NETWORK_NO_SOURCE"), 3),
    (&ascii(b"HAVE_NOTHING"), HAVE_NOTHING),
    (&ascii(b"HAVE_METADATA"), 1),
    (&ascii(b"HAVE_CURRENT_DATA"), 2),
    (&ascii(b"HAVE_FUTURE_DATA"), 3),
    (&ascii(b"HAVE_ENOUGH_DATA"), 4),
];

#[derive(Clone, Copy)]
enum MediaReadback {
    CurrentSrc,
    NetworkState,
    ReadyState,
    Paused,
}

type Accessor = (&'static [u16], &'static [u16], MediaReadback);

const ACCESSORS: &[Accessor] = &[
    (
        &ascii(b"currentSrc"),
        &ascii(b"get currentSrc"),
        MediaReadback::CurrentSrc,
    ),
    (
        &ascii(b"networkState"),
        &ascii(b"get networkState"),
        MediaReadback::NetworkState,
    ),
    (
        &ascii(b"readyState"),
        &ascii(b"get readyState"),
        MediaReadback::ReadyState,
    ),
    (
        &ascii(b"paused"),
        &ascii(b"get paused"),
        MediaReadback::Paused,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    constructor: &Value<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for &(name, getter, readback) in ACCESSORS {
        let capture = capture(size_of::<(Capture, MediaReadback)>())?;
        install_getter(
            scope,
            prototype,
            name,
            getter,
            true,
            Callback::new(move |scope, arguments| {
                capture.read_media_status(scope, arguments, readback)
            }),
        )?;
    }
    for &(name, code) in CONSTANTS {
        let value = scope.primitive(Primitive::Number(f64::from(code)))?;
        for target in [constructor, prototype] {
            define_data(
                scope,
                target,
                PropertyKey::String(name),
                &value,
                false,
                true,
                false,
            )?;
        }
    }
    Ok(())
}

impl Capture {
    fn read_media_status<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        readback: MediaReadback,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Media;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let state = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                read.step()?;
                let state = node
                    .as_element()
                    .and_then(|element| element.initial_media_state())
                    .ok_or(DomError::WrongNodeKind)?;
                Ok(Some(state))
            })?;
            let Some(state) = state else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            let value = match readback {
                MediaReadback::CurrentSrc => self.copy_dom_units(scope, state.current_src())?,
                MediaReadback::NetworkState if state.network_is_empty() => {
                    scope.primitive(Primitive::Number(f64::from(NETWORK_EMPTY)))?
                }
                MediaReadback::ReadyState if !state.has_media_data() => {
                    scope.primitive(Primitive::Number(f64::from(HAVE_NOTHING)))?
                }
                MediaReadback::Paused => scope.primitive(Primitive::Boolean(state.paused()))?,
                _ => return Err(DomError::WrongNodeKind.into()),
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

//! Ordinary string reflections for HTML, Base, Mod and Map.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

#[derive(Clone, Copy)]
pub(super) enum Interface {
    Html,
    Base,
    Mod,
    Map,
}

impl Interface {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Html => "HTMLHtmlElement",
            Self::Base => "HTMLBaseElement",
            Self::Mod => "HTMLModElement",
            Self::Map => "HTMLMapElement",
        }
    }

    pub(super) fn error_message(self) -> &'static [u16] {
        match self {
            Self::Html => const { &ascii(b"HTMLHtmlElement required") },
            Self::Base => const { &ascii(b"HTMLBaseElement required") },
            Self::Mod => const { &ascii(b"HTMLModElement required") },
            Self::Map => const { &ascii(b"HTMLMapElement required") },
        }
    }

    pub(super) fn install<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        let member: reflection::Member = match self {
            Self::Html => (
                &const { ascii(b"version") },
                &const { ascii(b"get version") },
                &const { ascii(b"set version") },
                &const { ascii(b"version") },
                Reflection::String,
            ),
            Self::Base => (
                &const { ascii(b"target") },
                &const { ascii(b"get target") },
                &const { ascii(b"set target") },
                &const { ascii(b"target") },
                Reflection::String,
            ),
            Self::Mod => (
                &const { ascii(b"dateTime") },
                &const { ascii(b"get dateTime") },
                &const { ascii(b"set dateTime") },
                &const { ascii(b"datetime") },
                Reflection::String,
            ),
            Self::Map => (
                &const { ascii(b"name") },
                &const { ascii(b"get name") },
                &const { ascii(b"set name") },
                &const { ascii(b"name") },
                Reflection::String,
            ),
        };
        reflection::install(
            scope,
            prototype,
            capture,
            Receiver::LocalString(self),
            &[member],
        )
    }
}

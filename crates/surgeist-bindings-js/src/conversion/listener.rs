//! Event listener argument unions for the standalone native Window profile.

use crate::{Value, ValueKind};

use super::{Conversion, Failure, Rule, Work, declarations, types};

/// Converted listener flags. An absent passive member remains distinct from false.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StandaloneListenerOptions {
    /// Capture phase selection.
    pub capture: bool,
    /// Native once registration.
    pub once: bool,
    /// Explicit passive override, if supplied.
    pub passive: Option<bool>,
}

const CAPTURE: &[u16] = &[99, 97, 112, 116, 117, 114, 101];
const ONCE: &[u16] = &[111, 110, 99, 101];
const PASSIVE: &[u16] = &[112, 97, 115, 115, 105, 118, 101];
const SIGNAL: &[u16] = &[115, 105, 103, 110, 97, 108];

impl<'scope> Conversion<'_, 'scope> {
    /// Admit the required nullable EventListener argument. Returns false for
    /// explicit null or undefined, and true for the exact original object.
    /// Object observation and Realm custody belong to the native adapter.
    pub fn event_listener_argument(
        &mut self,
        input: Option<&Value<'scope>>,
    ) -> Result<bool, Failure> {
        self.work(Work::Argument, 1).map_err(Failure)?;
        let Some(input) = input else {
            return Err(self.validation(Rule::RequiredArgument("EventTarget listener")));
        };
        self.work(Work::Value, 1).map_err(Failure)?;
        let kind = self.scope.kind(input);
        match self.core(kind)? {
            ValueKind::Null | ValueKind::Undefined => Ok(false),
            ValueKind::Object => Ok(true),
            _ => Err(self.validation(Rule::CallbackInterface("EventListener"))),
        }
    }

    /// Convert the add-listener options union in declaration order. This
    /// profile has no AbortSignal binding, so a supplied signal cannot pass
    /// the interface brand check. An omitted or undefined signal is valid.
    pub fn standalone_add_listener_options(
        &mut self,
        input: Option<&Value<'scope>>,
    ) -> Result<StandaloneListenerOptions, Failure> {
        self.work(Work::Argument, 1).map_err(Failure)?;
        let Some(input) = input else {
            return Ok(StandaloneListenerOptions::default());
        };
        let kind = self.scope.kind(input);
        match self.core(kind)? {
            ValueKind::Object => {
                self.work(Work::Value, 1).map_err(Failure)?;
                let capture = self.listener_flag(input, CAPTURE)?.unwrap_or(false);
                let once = self.listener_flag(input, ONCE)?.unwrap_or(false);
                let passive = self.listener_flag(input, PASSIVE)?;
                if self.member(input, true, SIGNAL)?.is_some() {
                    return Err(self.validation(Rule::Interface("AbortSignal")));
                }
                Ok(StandaloneListenerOptions {
                    capture,
                    once,
                    passive,
                })
            }
            ValueKind::Null | ValueKind::Undefined => Ok(StandaloneListenerOptions::default()),
            _ => Ok(StandaloneListenerOptions {
                capture: *self.value(&types::BOOLEAN, input)?.get(),
                ..StandaloneListenerOptions::default()
            }),
        }
    }

    /// Convert the remove-listener options union. Only the declared capture
    /// member participates in removal and other properties are not read.
    pub fn event_listener_removal_capture(
        &mut self,
        input: Option<&Value<'scope>>,
    ) -> Result<bool, Failure> {
        self.work(Work::Argument, 1).map_err(Failure)?;
        let Some(input) = input else {
            return Ok(false);
        };
        let kind = self.scope.kind(input);
        match self.core(kind)? {
            ValueKind::Object => Ok(self
                .value(&declarations::event_listener_options(), input)?
                .get()
                .capture),
            ValueKind::Null | ValueKind::Undefined => Ok(false),
            _ => Ok(*self.value(&types::BOOLEAN, input)?.get()),
        }
    }

    fn listener_flag(
        &mut self,
        input: &Value<'scope>,
        name: &'static [u16],
    ) -> Result<Option<bool>, Failure> {
        self.member(input, true, name)?
            .map(|value| self.value(&types::BOOLEAN, &value).map(|flag| *flag.get()))
            .transpose()
    }
}

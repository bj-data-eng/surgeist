use crate::{DomString, Value, ValueKind};

use super::{Conversion, Converted, Failure, Rule, Work, declarations, types};

/// The selected ToggleEventInit fields, with the Element's native identity
/// and original scoped object supplied together.
pub struct ToggleEventOptions<'scope, Element> {
    /// Inherited bubbling flag.
    pub bubbles: bool,
    /// Inherited cancelability flag.
    pub cancelable: bool,
    /// Inherited composed flag.
    pub composed: bool,
    /// Exact converted new state. The declared default is the empty string.
    pub new_state: Option<Converted<DomString>>,
    /// Exact converted old state. The declared default is the empty string.
    pub old_state: Option<Converted<DomString>>,
    /// Native Element selected from the original scoped object.
    pub source: Option<(Element, Value<'scope>)>,
}

const NEW_STATE: &[u16] = &[110, 101, 119, 83, 116, 97, 116, 101];
const OLD_STATE: &[u16] = &[111, 108, 100, 83, 116, 97, 116, 101];
const SOURCE: &[u16] = &[115, 111, 117, 114, 99, 101];

impl<'scope> Conversion<'_, 'scope> {
    /// Convert inherited EventInit fields before the derived members. The
    /// adapter selects Element from the exact branded native projection.
    pub fn toggle_event_init<Projection: 'static, Element>(
        &mut self,
        input: Option<&Value<'scope>>,
        select_element: impl FnOnce(&Projection) -> Option<Element>,
    ) -> Result<ToggleEventOptions<'scope, Element>, Failure> {
        self.work(Work::Argument, 1).map_err(Failure)?;
        let object = match input {
            Some(input) => self.dictionary("ToggleEventInit", input)?,
            None => false,
        };
        let Some(input) = input.filter(|_| object) else {
            return Ok(ToggleEventOptions {
                bubbles: false,
                cancelable: false,
                composed: false,
                new_state: None,
                old_state: None,
                source: None,
            });
        };
        let flags = self.value(&declarations::event_init(), input)?;
        let new_state = self
            .member(input, true, NEW_STATE)?
            .map(|value| self.value(&types::DOM_STRING, &value))
            .transpose()?;
        let old_state = self
            .member(input, true, OLD_STATE)?
            .map(|value| self.value(&types::DOM_STRING, &value))
            .transpose()?;
        let source = if let Some(value) = self.member(input, true, SOURCE)? {
            self.work(Work::Value, 1).map_err(Failure)?;
            let kind = self.scope.kind(&value);
            if self.core(kind)? == ValueKind::Null {
                None
            } else {
                let projection = self.scope.native_object::<Projection>(&value);
                let element = self
                    .core(projection)?
                    .and_then(|projection| select_element(projection.payload()))
                    .ok_or_else(|| self.validation(Rule::Interface("Element")))?;
                Some((element, value))
            }
        } else {
            None
        };
        Ok(ToggleEventOptions {
            bubbles: flags.get().bubbles,
            cancelable: flags.get().cancelable,
            composed: flags.get().composed,
            new_state,
            old_state,
            source,
        })
    }
}

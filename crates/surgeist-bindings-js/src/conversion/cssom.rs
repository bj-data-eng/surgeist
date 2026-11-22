//! Conversion of the CSSStyleSheet constructor dictionary in Web IDL order.

use super::{Conversion, Converted, Failure, Work, types};
use crate::{DomString, Scope, Value};

pub(crate) struct SheetOptions<M> {
    pub(crate) base_url: Option<Converted<Option<DomString>>>,
    pub(crate) disabled: bool,
    pub(crate) media: MediaOption<M>,
}

pub(crate) enum MediaOption<M> {
    Empty,
    Text(Converted<DomString>),
    List(M),
}

impl<'scope> Conversion<'_, 'scope> {
    pub(crate) fn sheet_options<M>(
        &mut self,
        options: Option<&Value<'scope>>,
        media_brand: impl FnOnce(&mut Scope<'scope>, &Value<'scope>) -> Result<Option<M>, crate::Error>,
    ) -> Result<SheetOptions<M>, Failure> {
        self.work(Work::Argument, 1).map_err(Failure)?;
        let options = if let Some(input) = options {
            let kind = self.scope.kind(input);
            (self.core(kind)? != crate::ValueKind::Undefined).then_some(input)
        } else {
            None
        };
        let Some(options) = options else {
            return Ok(SheetOptions {
                base_url: None,
                disabled: false,
                media: MediaOption::Empty,
            });
        };
        self.work(Work::Value, 1).map_err(Failure)?;
        let object = self.dictionary("CSSStyleSheetInit", options)?;
        let base_url = self
            .member(options, object, crate::cssom::BASE_URL)?
            .map(|input| self.value(&types::DOM_STRING.nullable(), &input))
            .transpose()?;
        let disabled = self
            .member(options, object, crate::cssom::DISABLED)?
            .map(|input| self.value(&types::BOOLEAN, &input))
            .transpose()?
            .is_some_and(|value| *value.get());
        let media = match self.member(options, object, crate::cssom::MEDIA)? {
            None => MediaOption::Empty,
            Some(input) => {
                self.work(Work::Value, 1).map_err(Failure)?;
                let branded = media_brand(self.scope, &input);
                match self.core(branded)? {
                    Some(media) => MediaOption::List(media),
                    None => MediaOption::Text(self.value(&types::DOM_STRING, &input)?),
                }
            }
        };
        Ok(SheetOptions {
            base_url,
            disabled,
            media,
        })
    }
}

impl<'scope> Conversion<'_, 'scope> {
    pub(crate) fn legacy_media_text(
        &mut self,
        input: &Value<'scope>,
    ) -> Result<Converted<DomString>, Failure> {
        let result = self.scope.kind(input);
        if self.core(result)? == crate::ValueKind::Null {
            self.work(Work::Value, 1).map_err(Failure)?;
            return Ok(self.plain(DomString::new(Vec::new())));
        }
        self.value(&types::DOM_STRING, input)
    }
}

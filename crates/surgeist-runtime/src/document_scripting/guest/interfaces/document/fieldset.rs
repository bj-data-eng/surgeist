//! Local Fieldset attributes over the canonical element.

use super::{
    barred_control::BarredControl,
    custom_validity::CustomValidityControl,
    form_owner::{self, FormOwnerGetter},
    reflection::{self, Receiver, Reflection},
    *,
};

const DISABLED: reflection::Member = (
    &ascii(b"disabled"),
    &ascii(b"get disabled"),
    &ascii(b"set disabled"),
    &ascii(b"disabled"),
    Reflection::Boolean,
);
const NAME: reflection::Member = (
    &ascii(b"name"),
    &ascii(b"get name"),
    &ascii(b"set name"),
    &ascii(b"name"),
    Reflection::String,
);

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Fieldset, &[DISABLED])?;
    form_owner::install(scope, prototype, capture, FormOwnerGetter::Fieldset)?;
    reflection::install(scope, prototype, capture, Receiver::Fieldset, &[NAME])?;
    BarredControl::Fieldset.install_type(scope, prototype, capture)?;
    BarredControl::Fieldset.install_validation(scope, prototype, capture)?;
    CustomValidityControl::Fieldset.install(scope, prototype, capture)
}

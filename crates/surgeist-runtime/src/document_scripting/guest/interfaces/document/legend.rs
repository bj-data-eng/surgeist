//! HTMLLegendElement members over the current native Legend.

use super::{
    form_owner::{self, FormOwnerGetter},
    reflection::{self, Receiver, Reflection},
    *,
};

const MEMBERS: &[reflection::Member] = &[(
    &ascii(b"align"),
    &ascii(b"get align"),
    &ascii(b"set align"),
    &ascii(b"align"),
    Reflection::String,
)];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    form_owner::install(scope, prototype, capture, FormOwnerGetter::Legend)?;
    reflection::install(scope, prototype, capture, Receiver::Legend, MEMBERS)
}

use crate::diagnostic::error::owned_name;
owned_name!(
    CssPropertyName,
    "A diagnostic-phase identity preserving a decoded authored CSS property name; it does not parse, resolve, or apply the property."
);

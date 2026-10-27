use crate::diagnostic::error::owned_name;

owned_name!(
    CssAtRuleName,
    "A diagnostic-phase identity preserving a decoded authored CSS at-rule name; it does not parse the rule or select recovery."
);
owned_name!(
    CssDescriptorName,
    "A diagnostic-phase identity preserving a decoded authored CSS descriptor name; it does not parse, resolve, or apply the descriptor."
);

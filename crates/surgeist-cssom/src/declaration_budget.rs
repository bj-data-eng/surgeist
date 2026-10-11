//! Conservative accounting for a composed declaration request.
use crate::CssomError;
use surgeist_css::*;

/// Work reserved for one composed source request. Each provider receives a disjoint allowance.
#[derive(Clone, Copy, Debug)]
pub struct CssomDeclarationRequestLimits {
    pub provider: CssSpecifiedValueSerializationLimits,
    pub max_entries: usize,
    pub max_input_bytes: usize,
    pub max_input_components: usize,
    pub max_nesting_depth: u32,
}
impl Default for CssomDeclarationRequestLimits {
    fn default() -> Self {
        Self {
            provider: CssSpecifiedValueSerializationLimits::default(),
            max_entries: 1_000_000,
            max_input_bytes: 16 * 1024 * 1024,
            max_input_components: 1_000_000,
            max_nesting_depth: 256,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct DeclarationBudget {
    pub limits: CssomDeclarationRequestLimits,
    remaining_calls: usize,
    input: usize,
    projection: usize,
    bytes: usize,
    entries: usize,
    text_bytes: usize,
}
impl DeclarationBudget {
    pub fn new(limits: CssomDeclarationRequestLimits, calls: usize) -> Self {
        Self {
            limits,
            remaining_calls: calls,
            input: limits.provider.max_input_nodes(),
            projection: limits.provider.max_projection_nodes(),
            bytes: limits.provider.max_css_bytes(),
            entries: limits.max_entries,
            text_bytes: limits.max_input_bytes,
        }
    }
    /// Reserve provider maxima up front because CSS intentionally exposes no mutable work meter.
    /// Actual work cannot exceed the disjoint reservations; unused reservations are not refunded.
    pub fn provider(&mut self) -> Result<CssSpecifiedValueSerializationLimits, CssomError> {
        if self.remaining_calls == 0 {
            return Err(CssomError::Limit {
                resource: "declaration provider phases",
                maximum: 0,
            });
        }
        let n = self.remaining_calls;
        let a = self.input / n;
        let b = self.projection / n;
        let c = self.bytes / n;
        self.input -= a;
        self.projection -= b;
        self.bytes -= c;
        self.remaining_calls -= 1;
        Ok(CssSpecifiedValueSerializationLimits::new(a, b, c))
    }
    pub fn phases(&mut self, n: usize) {
        self.remaining_calls = n;
    }
    pub fn expansion(&mut self, declaration: &CssDeclaration) -> Result<(), CssomError> {
        let components = declaration.value_components().component_count();
        // Bound new expansion allocation from the CSS schema before calling the provider.
        // Four-side modes have independently supplied schema members; a conservative
        // maximum avoids interpreting the value or duplicating its mode grammar.
        let members = match declaration.known() {
            None => 1,
            Some(known) => match known
                .grammar()
                .metadata()
                .map_err(CssomError::Metadata)?
                .kind()
            {
                CssPropertyKindRef::Longhand(_) => 1,
                CssPropertyKindRef::Shorthand(meta) => meta.members().len(),
                CssPropertyKindRef::FourSideShorthand(meta) => meta
                    .members(CssBoxSideKind::Physical)
                    .len()
                    .max(meta.members(CssBoxSideKind::Logical).len()),
                CssPropertyKindRef::UniversalReset(_) => CssKnownProperty::all().len(),
                _ => {
                    return Err(CssomError::InvalidInput(
                        "unavailable expansion metadata branch",
                    ));
                }
            },
        };
        let work = components.checked_add(members).ok_or(CssomError::Limit {
            resource: "declaration expansion capacity",
            maximum: self.limits.max_entries,
        })?;
        self.input = self
            .input
            .checked_sub(components)
            .ok_or(CssomError::Limit {
                resource: "declaration input work",
                maximum: self.limits.provider.max_input_nodes(),
            })?;
        self.projection = self
            .projection
            .checked_sub(members)
            .ok_or(CssomError::Limit {
                resource: "declaration projection work",
                maximum: self.limits.provider.max_projection_nodes(),
            })?;
        self.scan(work)
    }
    pub fn scan(&mut self, n: usize) -> Result<(), CssomError> {
        self.entries = self.entries.checked_sub(n).ok_or(CssomError::Limit {
            resource: "declaration request entries",
            maximum: self.limits.max_entries,
        })?;
        Ok(())
    }
    pub fn scalar_output(&mut self, value: &str) -> Result<(), CssomError> {
        self.bytes = self
            .bytes
            .checked_sub(value.len())
            .ok_or(CssomError::Limit {
                resource: "declaration output bytes",
                maximum: self.limits.provider.max_css_bytes(),
            })?;
        Ok(())
    }
    pub fn text(&mut self, source: &str) -> Result<(), CssomError> {
        self.text_bytes = self
            .text_bytes
            .checked_sub(source.len())
            .ok_or(CssomError::Limit {
                resource: "declaration input bytes",
                maximum: self.limits.max_input_bytes,
            })?;
        Ok(())
    }
    pub fn input(&mut self, source: &str) -> Result<CssComponentValueLimits, CssomError> {
        self.text(source)?;
        let components = self.limits.max_input_components.min(self.input / 3);
        self.input -= components;
        CssComponentValueLimits::try_new(
            self.limits.max_nesting_depth,
            components,
            self.limits.max_input_bytes,
        )
        .ok_or(CssomError::InvalidInput("declaration nesting bound"))
    }
}

use crate::parser::CssNamespaceBindings;
// Shared checked media construction uses the same private admission engine.
use crate::rule::{CssNamespaceName, CssNamespacePrefix, CssRule, CssSheet};

/// Immutable namespace bindings used to parse authored selector and rule fragments.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CssNamespaceContext(pub(crate) CssNamespaceBindings);
impl CssNamespaceContext {
    /// Takes bindings in authored order; the last binding for each prefix wins.
    pub fn from_bindings(
        bindings: impl IntoIterator<Item = (Option<CssNamespacePrefix>, CssNamespaceName)>,
    ) -> Self {
        let mut result = Self::default();
        for (prefix, name) in bindings {
            result.0.activate(prefix, name);
        }
        result
    }
    /// Copies the retained top-level namespace declarations from a stylesheet.
    pub fn from_sheet(sheet: &CssSheet) -> Self {
        Self::from_bindings(sheet.rules().iter().filter_map(|rule| match rule {
            CssRule::Namespace(rule) => Some((rule.prefix().cloned(), rule.name().clone())),
            _ => None,
        }))
    }
    /// Returns the default binding, distinguishing absence from the empty namespace.
    pub fn default_namespace(&self) -> Option<&CssNamespaceName> {
        self.0.default.as_ref()
    }
    /// Returns the case-sensitive named binding.
    pub fn named_namespace(&self, prefix: &CssNamespacePrefix) -> Option<&CssNamespaceName> {
        self.0
            .named
            .iter()
            .find(|(p, _)| p == prefix)
            .map(|(_, name)| name)
    }
}

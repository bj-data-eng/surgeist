//! Sheets, rules and rule graphs whose serialization is charged to a host's
//! ledger scope.

use crate::accounting::{CssAccount, CssAccountingError, CssFront, CssHostScope};
use crate::rule::{CssRule, CssSheet, CssStyleAncestor};
use crate::selector::CssNamespaceContext;
use crate::serialization::specified_serialization::SerializationWork;
use crate::serialization::{
    CssRuleGraphInput, CssRuleSerializationError, CssRuleSerializationLimits,
    CssSerializationError, CssSerializationLimits,
};

/// A sheet, rule or rule graph whose work is charged below a host's scope.
///
/// `accounted` on [`CssSheet`], [`CssRule`] and [`CssRuleGraphInput`] makes
/// one. Each method does exactly what the method of the same name on the
/// value does, and returns its result inside `Ok`. When the call returns it
/// charges the host's operation once, in the `css-serialize` scope, with
/// [`CssWork::SerializedInputNode`](crate::accounting::CssWork::SerializedInputNode),
/// [`CssWork::SerializedProjectionNode`](crate::accounting::CssWork::SerializedProjectionNode)
/// and [`CssWork::SerializedByte`](crate::accounting::CssWork::SerializedByte):
/// the input nodes, projection nodes and bytes the call admitted against its
/// limits. The call runs inside one phase of that scope. The number of ledger
/// calls does not depend on the value.
///
/// A call that a limit refuses is charged the work admitted before the
/// refusal and returns the refusal inside `Ok`. Shortest-form comparisons
/// that a serialization makes run outside its limits and are not charged.
///
/// Serialization of a single value or declaration is not charged on its own:
/// its own work is a few hundred nanoseconds, and one charge would add a large
/// share of it. It is part of the charge of the sheet, rule or graph that
/// contains it.
///
/// # Errors
///
/// Each method returns a [`CssAccountingError`] in place of its result when
/// the host's operation does not admit the charge, or when the ledger cannot
/// open the phase; in the second case nothing is serialized.
///
/// ```
/// use surgeist_css::accounting::{CssFront, CssHostScope, CssLimits, CssWork};
/// use surgeist_css::serialization::CssSerializationLimits;
/// use surgeist_css::CssParser;
/// use surgeist_ledger::{Limits, Operation};
///
/// let limits: CssLimits = Limits::unlimited();
/// let operation = Operation::new("host", limits, 1 + CssFront::ALL.len())?.share();
/// let host = CssHostScope::open(operation.clone(), operation.root())?;
///
/// let sheet = CssParser::default().parse_sheet("a { color: red }").into_parts().0;
/// let text = sheet
///     .accounted(&host)
///     .serialize_specified(CssSerializationLimits::default())??;
/// assert_eq!(text, "a { color: red; }");
/// let usage = operation.usage(operation.root())?;
/// assert_eq!(usage.work(CssWork::SerializedByte), 17);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CssAccounted<'host, T> {
    value: T,
    account: CssAccount<'host>,
}

impl CssSheet {
    /// This sheet, with its work charged below `host`.
    #[must_use]
    pub const fn accounted<'host>(&self, host: &'host CssHostScope) -> CssAccounted<'host, &Self> {
        CssAccounted {
            value: self,
            account: CssAccount::host(host),
        }
    }
}

impl CssRule {
    /// This rule, with its work charged below `host`.
    #[must_use]
    pub const fn accounted<'host>(&self, host: &'host CssHostScope) -> CssAccounted<'host, &Self> {
        CssAccounted {
            value: self,
            account: CssAccount::host(host),
        }
    }
}

impl<'graph> CssRuleGraphInput<'graph> {
    /// This rule graph, with its work charged below `host`.
    #[must_use]
    pub const fn accounted<'host>(self, host: &'host CssHostScope) -> CssAccounted<'host, Self> {
        CssAccounted {
            value: self,
            account: CssAccount::host(host),
        }
    }
}

impl CssAccounted<'_, &CssSheet> {
    /// [`CssSheet::serialize_specified`], charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The sheet's own refusal is returned inside `Ok`.
    pub fn serialize_specified(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<Result<String, CssSerializationError>, CssAccountingError> {
        let sheet = self.value;
        self.charged(|| {
            crate::serialization::specified_rule_serialization::serialize_specified_counted(
                sheet, limits,
            )
        })
    }

    /// [`CssSheet::serialize_cssom`], charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The sheet's own refusal is returned inside `Ok`.
    pub fn serialize_cssom(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<Result<String, CssRuleSerializationError>, CssAccountingError> {
        let sheet = self.value;
        self.charged(|| sheet.serialize_cssom_counted(limits))
    }
}

impl CssAccounted<'_, &CssRule> {
    /// [`CssRule::serialize_specified`], charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The rule's own refusal is returned inside `Ok`.
    pub fn serialize_specified(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<Result<String, CssSerializationError>, CssAccountingError> {
        let rule = self.value;
        self.charged(|| {
            crate::serialization::specified_rule_serialization::serialize_specified_counted(
                rule, limits,
            )
        })
    }

    /// [`CssRule::serialize_cssom`], charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The rule's own refusal is returned inside `Ok`.
    pub fn serialize_cssom(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<Result<String, CssRuleSerializationError>, CssAccountingError> {
        let rule = self.value;
        self.charged(|| rule.serialize_cssom_counted(limits))
    }
}

impl CssAccounted<'_, CssRuleGraphInput<'_>> {
    /// [`CssRuleGraphInput::serialize_specified`], charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The graph's own refusal is returned inside `Ok`.
    pub fn serialize_specified(
        &self,
        limits: CssRuleSerializationLimits,
    ) -> Result<Result<String, CssRuleSerializationError>, CssAccountingError> {
        let graph = self.value;
        self.charged(|| graph.serialize_specified_counted(limits))
    }

    /// [`CssRuleGraphInput::serialize_cssom`], charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The graph's own refusal is returned inside `Ok`.
    pub fn serialize_cssom(
        &self,
        namespaces: &CssNamespaceContext,
        ancestor: CssStyleAncestor,
        limits: CssRuleSerializationLimits,
    ) -> Result<Result<String, CssRuleSerializationError>, CssAccountingError> {
        let graph = self.value;
        self.charged(|| graph.serialize_cssom_counted(namespaces, ancestor, limits))
    }
}

impl<T> CssAccounted<'_, T> {
    /// Runs `serialize` inside a phase of the serialization scope, then
    /// charges the work it admitted, whether it succeeded or not.
    fn charged<R, E>(
        &self,
        serialize: impl FnOnce() -> (Result<R, E>, SerializationWork),
    ) -> Result<Result<R, E>, CssAccountingError> {
        let (result, work) = self.account.timed(CssFront::Serialization, serialize)?;
        self.account
            .charge(CssFront::Serialization, &serialization_changes(work))?;
        Ok(result)
    }
}

/// The changes that charge `work` to the serialization scope.
fn serialization_changes(work: SerializationWork) -> [crate::accounting::CssChange; 3] {
    use crate::accounting::{CssChange, CssWork};
    let amount = |count: usize| u64::try_from(count).unwrap_or(u64::MAX);
    [
        CssChange::Work(CssWork::SerializedInputNode, amount(work.input_nodes)),
        CssChange::Work(
            CssWork::SerializedProjectionNode,
            amount(work.projection_nodes),
        ),
        CssChange::Work(CssWork::SerializedByte, amount(work.css_bytes)),
    ]
}

#[cfg(test)]
mod tests;

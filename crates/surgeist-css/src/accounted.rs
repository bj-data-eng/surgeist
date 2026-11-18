//! Sheets, rules, rule graphs and colours whose serialization, normalization
//! and colour resolution are charged to a host's ledger scope.

use crate::accounting::{
    CssAccount, CssAccountingError, CssChange, CssFront, CssHostScope, CssWork,
};
use crate::color::{
    CssAuthoredColorOrdinaryLimits, CssAuthoredColorResult, CssColor, CssColorFactIdentity,
    CssColorFacts, CssColorGrammarRequest, CssColorInput, CssColorOrdinaryLimits,
    CssColorOrdinaryResult, CssColorResolveError,
};
use crate::normalize::normalization::NormalizationWork;
use crate::normalize::{CssNormalizationError, CssNormalizationLimits, CssNormalizedSheet};
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
/// value does, and returns its result inside `Ok`. The call runs inside one
/// phase of its front's scope, and when it returns it charges that scope once
/// with what the call admitted against its limits. The number of ledger calls
/// does not depend on the value.
///
/// - The serialization methods charge the `css-serialize` scope with
///   [`CssWork::SerializedInputNode`](crate::accounting::CssWork::SerializedInputNode),
///   [`CssWork::SerializedProjectionNode`](crate::accounting::CssWork::SerializedProjectionNode)
///   and [`CssWork::SerializedByte`](crate::accounting::CssWork::SerializedByte):
///   the input nodes, projection nodes and bytes. Shortest-form comparisons
///   that a serialization makes run outside its limits and are not charged.
/// - [`normalize`](CssAccounted::normalize) on a sheet charges the
///   `css-normalize` scope with
///   [`CssWork::NormalizedRule`](crate::accounting::CssWork::NormalizedRule),
///   [`CssWork::NormalizedDeclaration`](crate::accounting::CssWork::NormalizedDeclaration)
///   and [`CssWork::NormalizedContribution`](crate::accounting::CssWork::NormalizedContribution):
///   the rules, declarations and longhand contributions.
///
/// A call that a limit refuses is charged the work admitted before the
/// refusal and returns the refusal inside `Ok`.
///
/// [`parse_used_color`](CssAccounted::parse_used_color) on a colour input
/// charges the `css-color` scope
/// [`CssWork::ColorInputUnit`](crate::accounting::CssWork::ColorInputUnit),
/// the length of the input, and
/// [`CssWork::ColorAction`](crate::accounting::CssWork::ColorAction), the
/// actions its driver admitted. [`resolve`](CssAccounted::resolve) on an
/// authored colour charges only the actions: the colour is an already
/// parsed value, whose source text the parse that built it charged.
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
/// open the phase; in the second case the value's own method has not run.
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

impl<'source> CssColorInput<'source> {
    /// This colour input, with its resolution charged below `host`.
    #[must_use]
    pub const fn accounted<'host>(self, host: &'host CssHostScope) -> CssAccounted<'host, Self> {
        CssAccounted {
            value: self,
            account: CssAccount::host(host),
        }
    }
}

impl CssColor {
    /// This authored colour, with its resolution charged below `host`.
    #[must_use]
    pub const fn accounted<'host>(&self, host: &'host CssHostScope) -> CssAccounted<'host, &Self> {
        CssAccounted {
            value: self,
            account: CssAccount::host(host),
        }
    }
}

impl<'source> CssAccounted<'_, CssColorInput<'source>> {
    /// [`parse_used_color`](crate::color::parse_used_color) over this input,
    /// charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The colour's own refusal is returned inside `Ok`, and is
    /// charged the input and the actions admitted before it.
    pub fn parse_used_color<'facts>(
        &self,
        request: CssColorGrammarRequest,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
        limits: CssColorOrdinaryLimits,
    ) -> Result<
        Result<CssColorOrdinaryResult<'source, 'facts>, CssColorResolveError>,
        CssAccountingError,
    > {
        let input = self.value;
        self.charged_to(
            CssFront::Color,
            || {
                crate::color::ordinary::parse_used_color_counted(
                    input, request, facts, expected, limits,
                )
            },
            |actions| {
                [
                    CssChange::Work(CssWork::ColorInputUnit, amount(input.len())),
                    CssChange::Work(CssWork::ColorAction, amount(actions)),
                ]
            },
        )
    }
}

impl<'authored> CssAccounted<'_, &'authored CssColor> {
    /// [`CssColor::resolve`], charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The colour's own refusal is returned inside `Ok`, and is
    /// charged the actions admitted before it.
    pub fn resolve<'facts>(
        &self,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
        limits: CssAuthoredColorOrdinaryLimits,
    ) -> Result<
        Result<CssAuthoredColorResult<'authored, 'facts>, CssColorResolveError>,
        CssAccountingError,
    > {
        let color = self.value;
        self.charged_to(
            CssFront::Color,
            || {
                crate::color::authored_resolution::ordinary::resolve_authored_color_counted(
                    color, facts, expected, limits,
                )
            },
            |actions| [CssChange::Work(CssWork::ColorAction, amount(actions))],
        )
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

    /// [`CssSheet::normalize`], charged to the host.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The sheet's own refusal is returned inside `Ok`, and is
    /// charged the rules, declarations and contributions admitted before it.
    pub fn normalize(
        &self,
        limits: CssNormalizationLimits,
    ) -> Result<Result<CssNormalizedSheet, CssNormalizationError>, CssAccountingError> {
        let sheet = self.value;
        self.charged_to(
            CssFront::Normalization,
            || sheet.normalize_counted(limits),
            normalization_changes,
        )
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
        self.charged_to(CssFront::Serialization, serialize, serialization_changes)
    }

    /// Runs `run` inside a phase of the scope of `front`, then charges that
    /// scope the changes of the work it admitted, whether it succeeded or not.
    fn charged_to<R, E, W, const N: usize>(
        &self,
        front: CssFront,
        run: impl FnOnce() -> (Result<R, E>, W),
        changes: impl FnOnce(W) -> [CssChange; N],
    ) -> Result<Result<R, E>, CssAccountingError> {
        let (result, work) = self.account.timed(front, run)?;
        self.account.charge(front, &changes(work))?;
        Ok(result)
    }
}

/// A count as a ledger amount. A count beyond `u64` is charged as `u64::MAX`.
fn amount(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}

/// The changes that charge `work` to the normalization scope.
fn normalization_changes(work: NormalizationWork) -> [CssChange; 3] {
    [
        CssChange::Work(CssWork::NormalizedRule, amount(work.rules)),
        CssChange::Work(CssWork::NormalizedDeclaration, amount(work.declarations)),
        CssChange::Work(CssWork::NormalizedContribution, amount(work.contributions)),
    ]
}

/// The changes that charge `work` to the serialization scope.
fn serialization_changes(work: SerializationWork) -> [CssChange; 3] {
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

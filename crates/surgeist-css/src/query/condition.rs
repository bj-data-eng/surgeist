//! One boolean condition model over grammar-specific leaves (#1388).
//!
//! Media Queries 4 §3 defines `<media-condition>`, Conditional 3 defines
//! `<supports-condition>`, and Conditional 5 defines `<boolean-condition>` (§3,
//! "a boolean algebra a la Media Queries 4 § 3"), `<container-query>`,
//! `<style-query>` and `<scroll-state-query>` (§5.4). They share one shape: a
//! negated operand, an explicitly parenthesized condition, operands joined by one
//! repeated operator, or a grammar-specific leaf. Each grammar names its leaf type;
//! this module owns the shape and the provenance every condition keeps.
use crate::component::values::serialize::CssCanonicalBuilder;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssSerializedValue,
    CssValueOrigin,
};
use crate::query::CssWhenLeaf;
use crate::query::container::{CssContainerLeaf, CssScrollStateLeaf, CssStyleQueryLeaf};
use crate::query::media::CssMediaLeaf;
use crate::query::media::construction::MediaConditionSyntax;
use crate::query::supports::CssSupportsLeaf;
use crate::query::supports::construction::SupportsLexical;
use crate::source::{
    CssRetainedSources, CssSourcePosition, CssSourceRetention, CssSourceRetentionError,
};

mod sealed {
    pub trait Sealed {}
}

/// The leaf type of one condition grammar: the tests that grammar's boolean algebra joins.
///
/// Only the leaf types of this crate implement it. The associated backing ties each
/// leaf to what its conditions retain of their authored text, so an accessor of one
/// backing cannot be called on a condition of the other:
///
/// ```compile_fail,E0599
/// use surgeist_css::query::CssCondition;
/// use surgeist_css::query::media::CssMediaLeaf;
/// fn components(value: &CssCondition<CssMediaLeaf>) {
///     let _ = value.components();
/// }
/// ```
pub trait CssConditionLeaf: sealed::Sealed {
    /// What a condition of this grammar retains of its authored text: a region of the shared
    /// lexical root for the component-list grammars, or the per-operand syntax of a media
    /// condition. The sealed trait keeps this an implementation detail of the crate's leaves.
    type Backing: Clone + std::fmt::Debug + PartialEq;
}

/// A region of the shared lexical root of a component-list grammar.
#[derive(Clone, Debug, PartialEq)]
pub struct LexicalBacking {
    lexical: SupportsLexical,
    form: SupportsAuthoredForm,
}

/// The syntax a media condition kept for each operand and operator.
#[derive(Clone, Debug, PartialEq)]
pub struct MediaBacking(MediaConditionSyntax);

macro_rules! condition_leaf {
    ($backing:ty => $($leaf:ty),+) => {
        $(
            impl sealed::Sealed for $leaf {}
            impl CssConditionLeaf for $leaf {
                type Backing = $backing;
            }
        )+
    };
}
condition_leaf!(
    LexicalBacking => CssSupportsLeaf,
    CssContainerLeaf,
    CssStyleQueryLeaf,
    CssScrollStateLeaf,
    CssWhenLeaf
);
condition_leaf!(MediaBacking => CssMediaLeaf);

/// One authored boolean condition over the leaves `L` of its grammar.
///
/// The authored shape is exposed through [`Self::kind`]. Private fields keep a
/// caller from attaching a forged source position or a mixture of boolean
/// operators that the grammar does not admit; a condition comes from parsing or
/// from the checked construction of its grammar.
///
/// ```compile_fail,E0616
/// use surgeist_css::query::{CssCondition, CssConditionKind};
/// use surgeist_css::query::supports::CssSupportsLeaf;
/// fn forge(mut value: CssCondition<CssSupportsLeaf>, kind: CssConditionKind<CssSupportsLeaf>) {
///     value.kind = Box::new(kind);
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssCondition<L: CssConditionLeaf> {
    kind: Box<CssConditionKind<L>>,
    origin: CssValueOrigin,
    backing: L::Backing,
}

/// How a supports condition was written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SupportsAuthoredForm {
    Condition,
    BareDeclaration,
    BareName,
}

/// The authored shape of one condition: its operator and operands, or its leaf.
///
/// This is the public structure of a condition, not an error kind.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssConditionKind<L: CssConditionLeaf> {
    /// `not <condition>`: the negation of one operand.
    Not(Box<CssCondition<L>>),
    /// Operands joined by `and`, in source order.
    And(CssConditionList<L>),
    /// Operands joined by `or`, in source order.
    Or(CssConditionList<L>),
    /// An explicit grouping pair around a condition, including redundant pairs.
    Parenthesized(Box<CssCondition<L>>),
    /// A grammar-specific leaf test.
    Leaf(L),
}

/// Two or more operands of one operator, in source order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssConditionList<L: CssConditionLeaf> {
    conditions: Vec<CssCondition<L>>,
}

impl<L: CssConditionLeaf> CssConditionList<L> {
    /// Returns `None` for fewer than two operands.
    #[must_use]
    pub fn try_new(conditions: Vec<CssCondition<L>>) -> Option<Self> {
        if conditions.len() < 2 {
            None
        } else {
            Some(Self::new(conditions))
        }
    }

    #[must_use]
    pub(crate) fn new(conditions: Vec<CssCondition<L>>) -> Self {
        debug_assert!(conditions.len() >= 2);
        Self { conditions }
    }

    /// Returns the operands in source order; there are at least two.
    #[must_use]
    pub fn conditions(&self) -> &[CssCondition<L>] {
        &self.conditions
    }
}

impl<L: CssConditionLeaf> CssCondition<L> {
    /// The authored shape of this condition.
    #[must_use]
    pub const fn kind(&self) -> &CssConditionKind<L> {
        &self.kind
    }

    /// The original or programmatic origin of the condition's first operand or keyword.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }

    /// The parsed position of [`Self::origin`], or `None` for a constructed condition.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(&self.origin)
    }

    pub(crate) fn into_kind(self) -> CssConditionKind<L> {
        *self.kind
    }
}

impl<L: CssConditionLeaf<Backing = LexicalBacking>> CssCondition<L> {
    /// A condition over a region of a shared lexical root.
    pub(crate) fn lexical(kind: CssConditionKind<L>, lexical: SupportsLexical) -> Self {
        Self::lexical_form(kind, lexical, SupportsAuthoredForm::Condition)
    }

    pub(crate) fn lexical_form(
        kind: CssConditionKind<L>,
        lexical: SupportsLexical,
        form: SupportsAuthoredForm,
    ) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: Box::new(kind),
            origin,
            backing: LexicalBacking { lexical, form },
        }
    }

    /// The lexical region this condition selects from its shared root.
    pub(crate) fn lexical_region(&self) -> &SupportsLexical {
        &self.backing.lexical
    }

    pub(crate) fn authored_form(&self) -> SupportsAuthoredForm {
        self.backing.form
    }
}

impl CssCondition<CssMediaLeaf> {
    pub(crate) fn media(
        kind: CssConditionKind<CssMediaLeaf>,
        origin: CssValueOrigin,
        syntax: MediaConditionSyntax,
    ) -> Self {
        Self {
            kind: Box::new(kind),
            origin,
            backing: MediaBacking(syntax),
        }
    }

    pub(crate) fn media_syntax(&self) -> &MediaConditionSyntax {
        &self.backing.0
    }
}

// Retained source handles: a component-list condition shares its lexical root; a
// media condition lists its own operands.
macro_rules! lexical_condition {
    ($leaf:ty) => {
        impl CssCondition<$leaf> {
            /// Borrows the complete selected lexical region, including authored
            /// operators, grouping and trivia, without copying enclosing or sibling
            /// subtrees.
            #[must_use]
            pub fn components(&self) -> &[CssComponentValue] {
                self.lexical_region().items()
            }
        }
        impl CssRetainedSources for CssCondition<$leaf> {
            fn visit_retained_sources(
                &self,
                sources: &mut CssSourceRetention,
            ) -> Result<(), CssSourceRetentionError> {
                sources.visit(&self.origin)?;
                sources.visit(self.lexical_region())
            }
        }
    };
}
lexical_condition!(CssSupportsLeaf);
lexical_condition!(CssContainerLeaf);
lexical_condition!(CssStyleQueryLeaf);
lexical_condition!(CssScrollStateLeaf);
lexical_condition!(CssWhenLeaf);

// Container, style, scroll-state and `@when` conditions serialize their lexical
// region verbatim; a supports condition adds the parentheses of a bare import form.
macro_rules! verbatim_condition {
    ($leaf:ty) => {
        impl CssCondition<$leaf> {
            /// Preserves original spelling, trivia and every grouping pair. Delimiters
            /// implied by parser recovery are emitted explicitly.
            ///
            /// `limits.max_css_bytes()` caps the output, including inserted syntax; the
            /// other component limits do not apply.
            ///
            /// # Errors
            ///
            /// Returns [`CssComponentValueError`] when the byte limit is exhausted.
            pub fn serialize_tokens(
                &self,
                limits: CssComponentValueLimits,
            ) -> Result<CssSerializedValue, CssComponentValueError> {
                let mut out = CssCanonicalBuilder::new(limits.max_css_bytes());
                out.push_components(self.components())?;
                out.finish()
            }
        }
    };
}
verbatim_condition!(CssContainerLeaf);
verbatim_condition!(CssStyleQueryLeaf);
verbatim_condition!(CssScrollStateLeaf);
verbatim_condition!(CssWhenLeaf);

impl CssRetainedSources for CssCondition<CssMediaLeaf> {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention<'_>,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.origin())?;
        match self.media_syntax() {
            MediaConditionSyntax::Feature(value) => {
                sources.visit(&value.component)?;
                sources.visit(&value.name)?;
                for values in &value.separators {
                    sources.visit(values.as_slice())?;
                }
            }
            MediaConditionSyntax::Group { closing } => sources.visit(closing)?,
            MediaConditionSyntax::Junction(values) => sources.visit(values.as_slice())?,
            MediaConditionSyntax::Enclosed | MediaConditionSyntax::Not => {}
        }
        match self.kind() {
            CssConditionKind::Parenthesized(value) | CssConditionKind::Not(value) => {
                sources.visit(value.as_ref())
            }
            CssConditionKind::And(value) | CssConditionKind::Or(value) => {
                for value in value.conditions() {
                    sources.visit(value)?;
                }
                Ok(())
            }
            CssConditionKind::Leaf(CssMediaLeaf::CustomMediaReference(value)) => {
                sources.visit(value.component())
            }
            CssConditionKind::Leaf(CssMediaLeaf::UnknownFeature(value)) => {
                sources.visit(value.component())
            }
            CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(value)) => {
                sources.visit(value.component())
            }
            // The private grammar derives the feature from the retained syntax.
            CssConditionKind::Leaf(CssMediaLeaf::Feature(_)) => Ok(()),
        }
    }
}

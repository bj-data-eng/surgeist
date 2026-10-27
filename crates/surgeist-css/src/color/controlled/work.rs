//! The actual finite computation selected by one continuation action.
//!
//! These identify fixed kernels and one-item folds; none denotes a whole token,
//! expression, profile array, parse, or serialized number. Library scalar math
//! is atomic and is not described as an invented internal instruction count.

use crate::numeric::CssMathFunction;

/// A finite operation whose variable operands have already been lent by actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorKernel {
    Numerical(surgeist_color::Kernel),
    /// One bounded UTF decoding/preprocessing transition.
    Preprocess,
    /// One bounded token-state transition, including held lookahead.
    Tokenize,
    /// Classify one fixed cached keyword/unit candidate.
    Classify,
    /// One fixed color or arithmetic grammar transition.
    Grammar,
    /// Transfer a fixed checked descriptor between stages.
    Transition,
    /// Select a decimal search boundary or update fixed search registers.
    DecimalSearch,
    /// Initialize one boundary digit or transition its fixed multiplication state.
    DecimalDigit,
    /// Compare fixed decimal lengths/scales or advance its completed comparison.
    DecimalCompare,
    /// Multiply one gradian coefficient digit by nine with fixed carry.
    ScaleDigit,
    /// Apply the canonical fixed unit/type conversion to one literal.
    Literal,
    /// Bind one actual relative channel from its fixed resolved origin.
    RelativeChannel,
    /// One checked numeric addition, subtraction, multiplication or division.
    Arithmetic,
    /// One fixed-arity scalar math kernel (at most three operands).
    Math(CssMathFunction),
    /// One supplied operand updates all three cached min/max/hypot aggregates.
    VariadicFolds,
    /// Resolve/convert one fixed three-channel color.
    ColorConversion,
    /// Resolve one fixed channel/alpha domain.
    Channel,
    /// One fixed source-ordered two-color interpolation fold.
    Interpolation,
    /// Update fixed normalization registers for one mix weight.
    Weight,
    /// Execute the source-selected fixed black/white contrast policy.
    Contrast,
    /// Select/validate one already read fact record or one exact compared unit.
    Fact,
}
impl CssColorKernel {
    /// Actual scalar fold count in this action. The three aggregates are
    /// independent useful computations, not one unknown-function quantum.
    #[must_use]
    pub const fn variadic_fold_count(self) -> usize {
        if matches!(self, Self::VariadicFolds) {
            3
        } else {
            0
        }
    }
}

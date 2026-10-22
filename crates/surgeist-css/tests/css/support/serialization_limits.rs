//! The bounded-serialization limit ladder shared by every serializer test.
//!
//! A bounded serializer receives three independent limits: authored input
//! nodes, projection nodes and output bytes. A ladder states the exact node
//! counts and the expected output, whose byte length is the exact byte limit.
//! It checks that those exact limits serialize the expected text, that each
//! limit one lower fails twice with that limit's own kind, that the exact
//! limits succeed again after every failure, and that the default limits
//! serialize the same text. The optional zero-limit rungs repeat the check
//! with each limit set to zero. A rung whose limit would be negative does not
//! exist, so an empty expected output has no byte rung.

use std::fmt::Debug;

use surgeist_css::*;

pub type Limits = CssSpecifiedValueSerializationLimits;
pub type LimitKind = CssSpecifiedValueSerializationErrorKind;

/// A serialization failure that may report one exhausted limit.
pub trait LimitFailure: Debug {
    /// The exhausted limit, or `None` when the failure is not a resource
    /// exhaustion.
    fn limit_kind(&self) -> Option<LimitKind>;
}

impl LimitFailure for CssSpecifiedValueSerializationError {
    fn limit_kind(&self) -> Option<LimitKind> {
        Some(self.kind())
    }
}

impl LimitFailure for CssSpecifiedRuleSerializationError {
    fn limit_kind(&self) -> Option<LimitKind> {
        match self.kind() {
            CssSpecifiedRuleSerializationErrorKind::Resource(kind) => Some(kind),
            _ => None,
        }
    }
}

impl LimitFailure for CssRuleCssomSerializationError {
    fn limit_kind(&self) -> Option<LimitKind> {
        match self.kind() {
            CssRuleCssomSerializationErrorKind::Resource(kind) => Some(kind),
            _ => None,
        }
    }
}

impl LimitFailure for CssMediaCssomSerializationError {
    fn limit_kind(&self) -> Option<LimitKind> {
        match self {
            CssMediaCssomSerializationError::Resource { error, .. } => Some(error.kind()),
            _ => None,
        }
    }
}

/// The exact limits of one serialization and the rungs one below them.
#[derive(Clone, Copy, Debug)]
pub struct LimitLadder<'a> {
    expected: &'a str,
    input: usize,
    projection: usize,
    zero_limits: bool,
}

impl<'a> LimitLadder<'a> {
    /// The ladder of `expected` text produced from exactly `input` authored
    /// nodes and `projection` projection nodes.
    pub const fn new(expected: &'a str, input: usize, projection: usize) -> Self {
        Self {
            expected,
            input,
            projection,
            zero_limits: false,
        }
    }

    /// Also checks each limit set to zero while the other two stay exact.
    #[must_use]
    pub const fn zero_limits(mut self) -> Self {
        self.zero_limits = true;
        self
    }

    /// The exact limits: both node counts and the expected byte length.
    pub fn exact(&self) -> Limits {
        Limits::new(self.input, self.projection, self.expected.len())
    }

    /// Every failing rung with the kind it must report.
    pub fn failing_limits(&self) -> Vec<(Limits, LimitKind)> {
        let (input, projection, bytes) = (self.input, self.projection, self.expected.len());
        assert!(
            input > 0 && projection > 0,
            "a ladder needs at least one input and one projection node"
        );
        let mut rungs = vec![
            (
                Limits::new(input - 1, projection, bytes),
                LimitKind::InputNodeLimit,
            ),
            (
                Limits::new(input, projection - 1, bytes),
                LimitKind::ProjectionNodeLimit,
            ),
        ];
        if bytes > 0 {
            rungs.push((
                Limits::new(input, projection, bytes - 1),
                LimitKind::ByteLimit,
            ));
        }
        if self.zero_limits {
            rungs.push((Limits::new(0, projection, bytes), LimitKind::InputNodeLimit));
            rungs.push((Limits::new(input, 0, bytes), LimitKind::ProjectionNodeLimit));
            if bytes > 0 {
                rungs.push((Limits::new(input, projection, 0), LimitKind::ByteLimit));
            }
        }
        rungs
    }

    /// Checks the ladder against `serialize`.
    pub fn assert<E: LimitFailure>(&self, serialize: impl Fn(Limits) -> Result<String, E>) {
        self.assert_with(serialize, |_, _, _| {});
    }

    /// Checks the ladder and passes every failure, with its limits and the
    /// kind it reported, to `inspect` for the caller's further checks.
    pub fn assert_with<E: LimitFailure>(
        &self,
        serialize: impl Fn(Limits) -> Result<String, E>,
        mut inspect: impl FnMut(Limits, LimitKind, &E),
    ) {
        let exact = self.exact();
        let expected = self.expected;
        assert_eq!(
            serialize(exact).as_deref().unwrap(),
            expected,
            "exact {exact:?}"
        );
        for (limits, kind) in self.failing_limits() {
            for _ in 0..2 {
                let error = serialize(limits)
                    .expect_err(&format!("{limits:?} must fail with {kind:?}: {expected}"));
                assert_eq!(
                    error.limit_kind(),
                    Some(kind),
                    "{limits:?}: {expected}: {error:?}"
                );
                inspect(limits, kind, &error);
            }
            assert_eq!(
                serialize(exact).as_deref().unwrap(),
                expected,
                "exact retry after {limits:?}"
            );
        }
        assert_eq!(
            serialize(Limits::default()).as_deref().unwrap(),
            expected,
            "default limits"
        );
    }

    /// Checks the ladder against `serialize(value, limits)` and that no
    /// attempt changes `value`.
    pub fn assert_unchanged<T, E>(
        &self,
        value: &T,
        serialize: impl Fn(&T, Limits) -> Result<String, E>,
    ) where
        T: Clone + Debug + PartialEq,
        E: LimitFailure,
    {
        self.assert_unchanged_with(value, serialize, |_, _, _| {});
    }

    /// [`Self::assert_unchanged`] that also passes every failure to `inspect`.
    pub fn assert_unchanged_with<T, E>(
        &self,
        value: &T,
        serialize: impl Fn(&T, Limits) -> Result<String, E>,
        mut inspect: impl FnMut(Limits, LimitKind, &E),
    ) where
        T: Clone + Debug + PartialEq,
        E: LimitFailure,
    {
        let before = value.clone();
        self.assert_with(
            |limits| serialize(value, limits),
            |limits, kind, error| {
                assert_eq!(value, &before, "{limits:?} changed the value");
                inspect(limits, kind, error);
            },
        );
        assert_eq!(value, &before);
    }

    /// Checks the ladder against a whole sheet whose exhaustion is only
    /// cumulative: every rule serializes separately within each failing
    /// limit, and the sheet failure names rule `rule_index`.
    pub fn assert_cumulative_sheet(&self, sheet: &CssSheet, rule_index: usize) {
        self.assert_unchanged_with(
            sheet,
            CssSheet::to_specified_css_with_limits,
            |limits, _, error| {
                assert!(
                    sheet
                        .rules()
                        .iter()
                        .all(|rule| rule.to_specified_css_with_limits(limits).is_ok()),
                    "each rule fits {limits:?} separately"
                );
                assert_eq!(error.rule_index(), Some(rule_index), "{limits:?}");
            },
        );
    }
}

/// [`LimitLadder::assert`] of `expected` with exact `input` and `projection`
/// node counts.
pub fn assert_limits<E: LimitFailure>(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(Limits) -> Result<String, E>,
) {
    LimitLadder::new(expected, input, projection).assert(serialize);
}

/// [`assert_limits`] of a serialization whose input and projection node
/// counts are both exactly `nodes`.
pub fn assert_node_limits<E: LimitFailure>(
    expected: &str,
    nodes: usize,
    serialize: impl Fn(Limits) -> Result<String, E>,
) {
    LimitLadder::new(expected, nodes, nodes).assert(serialize);
}

/// [`assert_limits`] that also checks every zero-limit rung.
pub fn assert_limits_with_zero<E: LimitFailure>(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(Limits) -> Result<String, E>,
) {
    LimitLadder::new(expected, input, projection)
        .zero_limits()
        .assert(serialize);
}

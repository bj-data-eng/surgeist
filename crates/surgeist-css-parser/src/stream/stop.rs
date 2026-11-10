//! The stop tokens a list of component values can end at.

use std::ops::BitOr;

/// A token that ends a list of component values (section 5.5.7, "consume a list
/// of component values" with a stop token).
///
/// Only a token in the list itself can stop it: a function or simple block is
/// one component, so a stop token inside one never matches.
///
/// ```
/// use surgeist_css_parser::stream::{StopToken, StopTokens};
///
/// let stops = StopToken::Comma | StopToken::ExclamationMark;
/// assert!(stops.contains(StopToken::Comma));
/// assert!(!stops.contains(StopToken::Semicolon));
/// assert_eq!(StopTokens::from(StopToken::Semicolon), StopTokens::NONE | StopToken::Semicolon);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum StopToken {
    /// A `<comma-token>`.
    Comma,
    /// A `<semicolon-token>`.
    Semicolon,
    /// The `<delim-token>` for U+0021 EXCLAMATION MARK, which starts the
    /// importance of a declaration (section 5.5.6, step 6).
    ExclamationMark,
    /// A `<{-token>`: a `{}`-block, whether it opened a group or was denied.
    OpenCurly,
}

impl StopToken {
    const fn bit(self) -> u8 {
        match self {
            Self::Comma => 1,
            Self::Semicolon => 2,
            Self::ExclamationMark => 4,
            Self::OpenCurly => 8,
        }
    }
}

/// A set of [`StopToken`]s. Combine them with `|`.
///
/// ```
/// use surgeist_css_parser::stream::{StopToken, StopTokens};
///
/// let none = StopTokens::NONE;
/// assert!(none.is_empty());
/// let some = none | StopToken::Comma | StopToken::OpenCurly;
/// assert!(some.contains(StopToken::OpenCurly));
/// assert!(!some.is_empty());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct StopTokens(u8);

impl StopTokens {
    /// The empty set: a list that nothing ends.
    pub const NONE: Self = Self(0);

    /// Returns whether `stop` is in the set.
    #[must_use]
    pub const fn contains(self, stop: StopToken) -> bool {
        self.0 & stop.bit() != 0
    }

    /// Returns whether no stop token is in the set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl From<StopToken> for StopTokens {
    fn from(stop: StopToken) -> Self {
        Self(stop.bit())
    }
}

impl BitOr for StopToken {
    type Output = StopTokens;

    fn bitor(self, other: Self) -> StopTokens {
        StopTokens(self.bit() | other.bit())
    }
}

impl BitOr<StopToken> for StopTokens {
    type Output = Self;

    fn bitor(self, stop: StopToken) -> Self {
        Self(self.0 | stop.bit())
    }
}

impl BitOr for StopTokens {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

//! HTML 877d #mq and #the-source-element surrounding rules, without CSS parsing.

use crate::internal::microsyntax::tokens::{ascii_case_insensitive_unit, is_ascii_whitespace};

pub(crate) mod native;

/// Original presence/content classification; non-ASCII whitespace is content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaValueClass {
    /// The attribute is missing.
    Absent,
    /// The attribute is present and has no code units.
    Empty,
    /// The attribute is present, nonempty, and made only of ASCII whitespace (U+0009, U+000A,
    /// U+000C, U+000D, U+0020).
    AsciiWhitespaceOnly,
    /// The attribute contains at least one code unit that is not ASCII whitespace. Non-ASCII
    /// whitespace counts as content.
    Nontrivial,
}

/// One bounded action. A controlled caller admits it before advancing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaCursorAction {
    /// Inspect the code unit at this UTF-16 offset in the value.
    InspectUnit(usize),
    /// Fixed finish: at most the three units of the source keyword `all`.
    Finish,
    /// The scan is done and `result` is available.
    Complete,
}

/// Finished local observations, without CSS grammar or environment claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMediaValue {
    class: HtmlMediaValueClass,
    restrictive_picture_value: bool,
}
impl HtmlMediaValue {
    /// The presence and content class of the attribute.
    pub fn class(self) -> HtmlMediaValueClass {
        self.class
    }
    /// Whether the value has non-whitespace content other than exactly `all`, compared ASCII
    /// case-insensitively, so it could exclude a `source` in a `picture`. This does not parse CSS.
    pub fn restrictive_picture_value(self) -> bool {
        self.restrictive_picture_value
    }
    /// Whether the value is specified and empty or ASCII-whitespace-only. An absent attribute
    /// returns false, so the caller handles absence separately.
    pub fn matches_unconditionally(self) -> bool {
        matches!(
            self.class,
            HtmlMediaValueClass::Empty | HtmlMediaValueClass::AsciiWhitespaceOnly
        )
    }
}

enum Pending<'a> {
    Inspect(usize, &'a [u16]),
    Finish,
    Complete,
}

/// Allocation-free borrowed scan. Each advance performs one announced action.
#[derive(Debug)]
pub struct HtmlMediaValueCursor<'a> {
    units: Option<&'a [u16]>,
    position: usize,
    first: Option<usize>,
    end: usize,
    result: Option<HtmlMediaValue>,
}
impl<'a> HtmlMediaValueCursor<'a> {
    /// Starts a scan of the UTF-16 code units of the attribute, or `None` when the attribute is
    /// absent.
    pub fn new(units: Option<&'a [u16]>) -> Self {
        Self {
            units,
            position: 0,
            first: None,
            end: 0,
            result: None,
        }
    }
    /// The one decision behind both the announced and the performed action;
    /// an inspection carries the units it reads.
    fn pending(&self) -> Pending<'a> {
        if self.result.is_some() {
            Pending::Complete
        } else if let Some(units) = self.units.filter(|u| self.position < u.len()) {
            Pending::Inspect(self.position, units)
        } else {
            Pending::Finish
        }
    }
    /// The action the next `advance` performs: one inspection with its offset, the fixed finish of
    /// at most three further units, or `Complete`. The caller admits it first.
    pub fn next_action(&self) -> HtmlMediaCursorAction {
        match self.pending() {
            Pending::Complete => HtmlMediaCursorAction::Complete,
            Pending::Inspect(index, _) => HtmlMediaCursorAction::InspectUnit(index),
            Pending::Finish => HtmlMediaCursorAction::Finish,
        }
    }
    /// Performs the announced action. It does nothing once the scan is complete.
    pub fn advance(&mut self) {
        match self.pending() {
            Pending::Inspect(index, units) => {
                let unit = units[index];
                if !is_ascii_whitespace(unit) {
                    self.first.get_or_insert(index);
                    self.end = index + 1;
                }
                self.position += 1;
            }
            Pending::Finish => {
                let class = match self.units {
                    None => HtmlMediaValueClass::Absent,
                    Some([]) => HtmlMediaValueClass::Empty,
                    Some(_) if self.first.is_none() => HtmlMediaValueClass::AsciiWhitespaceOnly,
                    Some(_) => HtmlMediaValueClass::Nontrivial,
                };
                let all = self.first.is_some_and(|first| {
                    self.end - first == 3
                        && self.units.is_some_and(|u| {
                            ascii_case_insensitive_unit(u[first], b'a')
                                && ascii_case_insensitive_unit(u[first + 1], b'l')
                                && ascii_case_insensitive_unit(u[first + 2], b'l')
                        })
                });
                self.result = Some(HtmlMediaValue {
                    class,
                    restrictive_picture_value: self.first.is_some() && !all,
                });
            }
            Pending::Complete => {}
        }
    }
    /// The finished observations, or `None` while the scan is incomplete.
    pub fn result(&self) -> Option<HtmlMediaValue> {
        self.result
    }
}

/// Placement only; this does not validate CSS media grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaMediaPlacement {
    /// The `media` attribute is not specified on the `meta` element.
    NotSpecified,
    /// The `media` attribute is specified, and the `meta` element's `name` is `theme-color`, the
    /// one case in which it affects processing (HTML877d #attr-meta-media).
    AllowedForThemeColor,
    /// The `media` attribute is specified on a `meta` element whose `name` is not `theme-color`,
    /// where it has no effect and authors must not use it (HTML877d #attr-meta-media).
    Forbidden,
}

pub(crate) mod theme {
    use super::ascii_case_insensitive_unit;
    /// One fixed-name comparison step; shared by ordinary and controlled owners.
    #[derive(Debug)]
    pub struct HtmlThemeColorNameCursor<'a> {
        /// Empty when the name is absent, which is never equal.
        units: &'a [u16],
        position: usize,
        equal: bool,
    }
    impl<'a> HtmlThemeColorNameCursor<'a> {
        /// Starts a comparison of the `name` attribute's UTF-16 code units with `theme-color`,
        /// ignoring ASCII case; `None` means the attribute is absent and never equal.
        pub fn new(units: Option<&'a [u16]>) -> Self {
            let units = units.unwrap_or(&[]);
            Self {
                units,
                position: 0,
                equal: units.len() == 11,
            }
        }
        /// Whether the comparison has reached its answer: a length or code unit mismatch, or all 11
        /// units.
        pub fn is_complete(&self) -> bool {
            !self.equal || self.position == 11
        }
        /// Compares one more code unit. It does nothing once the comparison is complete.
        pub fn advance(&mut self) {
            if !self.is_complete() {
                self.equal = ascii_case_insensitive_unit(
                    self.units[self.position],
                    b"theme-color"[self.position],
                );
                self.position += 1;
            }
        }
        /// Whether the name equals `theme-color` ASCII case-insensitively; false while the
        /// comparison is incomplete.
        pub fn result(&self) -> bool {
            self.is_complete() && self.equal
        }
    }
}

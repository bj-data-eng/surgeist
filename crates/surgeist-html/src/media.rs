//! HTML 877d #mq and #the-source-element surrounding rules, without CSS parsing.

/// Original presence/content classification; non-ASCII whitespace is content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaValueClass {
    Absent,
    Empty,
    AsciiWhitespaceOnly,
    Nontrivial,
}

/// One bounded action. A controlled caller admits it before advancing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaCursorAction {
    InspectUnit(usize),
    /// Fixed finish: at most the three units of the source keyword `all`.
    Finish,
    Complete,
}

/// Finished local observations, without CSS grammar or environment claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMediaValue {
    class: HtmlMediaValueClass,
    restrictive_picture_value: bool,
}
impl HtmlMediaValue {
    pub fn class(self) -> HtmlMediaValueClass {
        self.class
    }
    pub fn restrictive_picture_value(self) -> bool {
        self.restrictive_picture_value
    }
    pub fn matches_unconditionally(self) -> bool {
        matches!(
            self.class,
            HtmlMediaValueClass::Empty | HtmlMediaValueClass::AsciiWhitespaceOnly
        )
    }
}

/// Allocation-free borrowed scan. Each advance performs one announced action.
pub struct HtmlMediaValueCursor<'a> {
    units: Option<&'a [u16]>,
    position: usize,
    first: Option<usize>,
    end: usize,
    result: Option<HtmlMediaValue>,
}
impl<'a> HtmlMediaValueCursor<'a> {
    pub fn new(units: Option<&'a [u16]>) -> Self {
        Self {
            units,
            position: 0,
            first: None,
            end: 0,
            result: None,
        }
    }
    pub fn next_action(&self) -> HtmlMediaCursorAction {
        if self.result.is_some() {
            HtmlMediaCursorAction::Complete
        } else if self.units.is_some_and(|u| self.position < u.len()) {
            HtmlMediaCursorAction::InspectUnit(self.position)
        } else {
            HtmlMediaCursorAction::Finish
        }
    }
    pub fn advance(&mut self) {
        match self.next_action() {
            HtmlMediaCursorAction::InspectUnit(index) => {
                let unit = self.units.expect("announced present input")[index];
                if !matches!(unit, 9 | 10 | 12 | 13 | 32) {
                    self.first.get_or_insert(index);
                    self.end = index + 1;
                }
                self.position += 1;
            }
            HtmlMediaCursorAction::Finish => {
                let class = match self.units {
                    None => HtmlMediaValueClass::Absent,
                    Some([]) => HtmlMediaValueClass::Empty,
                    Some(_) if self.first.is_none() => HtmlMediaValueClass::AsciiWhitespaceOnly,
                    Some(_) => HtmlMediaValueClass::Nontrivial,
                };
                let all = self.first.is_some_and(|first| {
                    self.end - first == 3
                        && self.units.is_some_and(|u| {
                            ascii_equal(u[first], b'a')
                                && ascii_equal(u[first + 1], b'l')
                                && ascii_equal(u[first + 2], b'l')
                        })
                });
                self.result = Some(HtmlMediaValue {
                    class,
                    restrictive_picture_value: self.first.is_some() && !all,
                });
            }
            HtmlMediaCursorAction::Complete => {}
        }
    }
    pub fn result(&self) -> Option<HtmlMediaValue> {
        self.result
    }
}

pub(crate) fn ascii_equal(unit: u16, byte: u8) -> bool {
    u8::try_from(unit).is_ok_and(|unit| unit.eq_ignore_ascii_case(&byte))
}

/// Placement only; this does not validate CSS media grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaMediaPlacement {
    NotSpecified,
    AllowedForThemeColor,
    Forbidden,
}

/// One fixed-name comparison step; shared by ordinary and controlled owners.
pub(crate) struct ThemeColorNameCursor<'a> {
    units: Option<&'a [u16]>,
    position: usize,
    equal: bool,
}
impl<'a> ThemeColorNameCursor<'a> {
    pub(crate) fn new(units: Option<&'a [u16]>) -> Self {
        Self {
            units,
            position: 0,
            equal: units.is_some_and(|u| u.len() == 11),
        }
    }
    pub(crate) fn is_complete(&self) -> bool {
        !self.equal || self.position == 11
    }
    pub(crate) fn advance(&mut self) {
        if !self.is_complete() {
            self.equal = ascii_equal(
                self.units.expect("fixed name input")[self.position],
                b"theme-color"[self.position],
            );
            self.position += 1;
        }
    }
    pub(crate) fn result(&self) -> bool {
        self.is_complete() && self.equal
    }
}

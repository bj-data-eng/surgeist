//! One borrowed owner for HTML script content meaning and preparation classification.
//! HTML877d #attr-script-type and #script-processing-prepare; MIME39aa535 §4.6.
//! The sixteen essence strings are incorporated under the WHATWG BSD permission
//! recorded by this crate's NOTICE. This is not MIME parsing or author validation.

use crate::internal::accounting::{HtmlChange, HtmlResource, HtmlWork};
use crate::internal::microsyntax::tokens::{ascii_case_insensitive_unit, is_ascii_whitespace};
use crate::internal::values::DocumentString;
use surgeist_ledger::{Change, Charge};

/// A script kind selected by HTML's type/language classification step.
/// This value alone does not establish preparation, saved metadata or execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptKind {
    /// A classic script: the type string matches a JavaScript MIME type essence, ignoring ASCII
    /// case (#prepare-the-script-element step 9). In preparation an explicit `type` also matches
    /// after leading and trailing ASCII whitespace is stripped. An absent or empty selection is
    /// Classic as well.
    Classic,
    /// A module script: the type string is ASCII case-insensitively `module`.
    Module,
    /// An import map: the type string is ASCII case-insensitively `importmap`.
    ImportMap,
    /// A speculation rules script: the type string is ASCII case-insensitively
    /// `speculationrules`.
    SpeculationRules,
}

// Sole selected MIME §4.6 essence table. Parameters are never removed.
const ESSENCES: [&[u8]; 16] = [
    b"application/ecmascript",
    b"application/javascript",
    b"application/x-ecmascript",
    b"application/x-javascript",
    b"text/ecmascript",
    b"text/javascript",
    b"text/javascript1.0",
    b"text/javascript1.1",
    b"text/javascript1.2",
    b"text/javascript1.3",
    b"text/javascript1.4",
    b"text/javascript1.5",
    b"text/jscript",
    b"text/livescript",
    b"text/x-ecmascript",
    b"text/x-javascript",
];
const KEYWORDS: [(&[u8], HtmlScriptKind); 3] = [
    (b"module", HtmlScriptKind::Module),
    (b"importmap", HtmlScriptKind::ImportMap),
    (b"speculationrules", HtmlScriptKind::SpeculationRules),
];

/// Required admission before the cursor's next actual action, charged as the
/// schema's record visits or inspected bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlScriptTypeWork {
    /// Charge this many visits before a read: two for the initial selection of the input, then,
    /// for each candidate comparison, one to start it and one for its length step, which precedes
    /// any unit comparison.
    RecordVisit(u64),
    /// Actual inspected bytes, including UTF-16 operands and ASCII keywords.
    InspectByte(u64),
}
impl HtmlScriptTypeWork {
    /// The schema change that admits this work.
    pub(crate) fn change(self) -> HtmlChange {
        match self {
            Self::RecordVisit(units) => Change::Work(HtmlWork::RecordVisit, units),
            Self::InspectByte(units) => Change::Work(HtmlWork::InspectByte, units),
        }
    }
}
/// Borrowed ASCII-insensitive virtual-prefix/range matcher consumed by
/// [`HtmlScriptTypeCursor`]. Credits precede length comparison and each read;
/// a completed result has no further work.
#[derive(Debug)]
pub(crate) struct ScriptTypeMatchCursor<'a> {
    prefix: &'static [u8],
    value: &'a DocumentString,
    start: usize,
    end: usize,
    expected: &'static [u8],
    position: usize,
    length_pending: bool,
    result: Option<bool>,
}
impl<'a> ScriptTypeMatchCursor<'a> {
    pub(crate) fn new(
        prefix: &'static [u8],
        value: &'a DocumentString,
        start: usize,
        end: usize,
        expected: &'static [u8],
    ) -> Self {
        Self {
            prefix,
            value,
            start,
            end,
            expected,
            position: 0,
            length_pending: true,
            result: None,
        }
    }
    /// The next admitted read, or the completed comparison result.
    pub(crate) fn pending(&self) -> Result<HtmlScriptTypeWork, bool> {
        if let Some(result) = self.result {
            Err(result)
        } else if self.length_pending {
            Ok(HtmlScriptTypeWork::RecordVisit(1))
        } else {
            Ok(HtmlScriptTypeWork::InspectByte(
                if self.position < self.prefix.len() {
                    2
                } else {
                    3
                },
            ))
        }
    }
    pub(crate) fn advance(&mut self) {
        if self.length_pending {
            self.length_pending = false;
            if self.prefix.len() + self.end - self.start != self.expected.len() {
                self.result = Some(false);
            } else if self.expected.is_empty() {
                self.result = Some(true);
            }
        } else {
            let actual = if self.position < self.prefix.len() {
                u16::from(self.prefix[self.position])
            } else {
                self.value.code_units()[self.start + self.position - self.prefix.len()]
            };
            if !ascii_case_insensitive_unit(actual, self.expected[self.position]) {
                self.result = Some(false);
            } else {
                self.position += 1;
                if self.position == self.expected.len() {
                    self.result = Some(true);
                }
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Content,
    Preparation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pass {
    RawEssences,
    TrimmedEssences,
    Keywords,
}
/// A scan stage; only the matching stage owns a matcher.
#[derive(Debug)]
enum Stage<'a> {
    Candidate,
    Match(ScriptTypeMatchCursor<'a>),
    TrimStart,
    TrimEnd,
}

/// A classification over one established nonempty input.
#[derive(Debug)]
struct Scan<'a> {
    value: &'a DocumentString,
    prefix: &'static [u8],
    /// Preparation of an explicit type retries essences after JS whitespace trim.
    trims: bool,
    pass: Pass,
    stage: Stage<'a>,
    candidate: usize,
    start: usize,
    end: usize,
}

impl<'a> Scan<'a> {
    fn expected(&self) -> &'static [u8] {
        match self.pass {
            Pass::RawEssences | Pass::TrimmedEssences => ESSENCES[self.candidate],
            Pass::Keywords => KEYWORDS[self.candidate].0,
        }
    }
    // The zero-work transitions inspect only established cursor fields, not
    // actual units. Scalar progress cannot substitute for an admitted matcher
    // read. Err is the finished classification.
    fn pending(&mut self) -> Result<HtmlScriptTypeWork, Option<HtmlScriptKind>> {
        loop {
            match &self.stage {
                Stage::Match(matcher) => match matcher.pending() {
                    Ok(work) => return Ok(work),
                    Err(true) => {
                        return Err(Some(if self.pass == Pass::Keywords {
                            KEYWORDS[self.candidate].1
                        } else {
                            HtmlScriptKind::Classic
                        }));
                    }
                    Err(false) => {
                        self.candidate += 1;
                        self.stage = Stage::Candidate;
                    }
                },
                Stage::Candidate
                    if self.pass != Pass::Keywords && self.candidate == ESSENCES.len() =>
                {
                    if self.pass == Pass::RawEssences && self.trims {
                        self.stage = Stage::TrimStart;
                    } else {
                        self.pass = Pass::Keywords;
                        self.candidate = 0;
                        self.start = 0;
                        self.end = self.value.len();
                    }
                }
                Stage::Candidate
                    if self.pass == Pass::Keywords && self.candidate == KEYWORDS.len() =>
                {
                    return Err(None);
                }
                Stage::Candidate => return Ok(HtmlScriptTypeWork::RecordVisit(1)),
                Stage::TrimStart if self.start == self.end => self.stage = Stage::TrimEnd,
                Stage::TrimEnd if self.start == self.end => {
                    self.pass = Pass::TrimmedEssences;
                    self.candidate = 0;
                    self.stage = Stage::Candidate;
                }
                Stage::TrimStart | Stage::TrimEnd => {
                    return Ok(HtmlScriptTypeWork::InspectByte(2));
                }
            }
        }
    }
    /// Performs the action described by a successful `pending`.
    fn advance(&mut self) {
        match &mut self.stage {
            Stage::Candidate => {
                self.stage = Stage::Match(ScriptTypeMatchCursor::new(
                    self.prefix,
                    self.value,
                    self.start,
                    self.end,
                    self.expected(),
                ));
            }
            Stage::Match(matcher) => matcher.advance(),
            Stage::TrimStart => {
                if is_ascii_whitespace(self.value.code_units()[self.start]) {
                    self.start += 1;
                } else {
                    self.stage = Stage::TrimEnd;
                }
            }
            Stage::TrimEnd => {
                if is_ascii_whitespace(self.value.code_units()[self.end - 1]) {
                    self.end -= 1;
                } else {
                    self.pass = Pass::TrimmedEssences;
                    self.candidate = 0;
                    self.stage = Stage::Candidate;
                }
            }
        }
    }
}

#[derive(Debug)]
enum State<'a> {
    Initial {
        explicit: Option<&'a DocumentString>,
        language: Option<&'a DocumentString>,
        mode: Mode,
    },
    Scan(Scan<'a>),
    Done(Option<HtmlScriptKind>),
}

/// Stack-only borrowed cursor. Each `advance` charges the work of its action first.
/// Raw units are read only by advance, never by credit inspection.
/// Ordinary native reports use the same actions under their ordinary contract.
/// Finite work: sixteen raw matches, at most two linear trim passes and sixteen
/// stripped matches for explicit preparation, then three exact keyword matches.
/// A virtual language prefix never allocates a concatenated string.
#[derive(Debug)]
pub struct HtmlScriptTypeCursor<'a> {
    state: State<'a>,
}
impl<'a> HtmlScriptTypeCursor<'a> {
    /// Classifies the current content type without preparation-only trimming.
    pub fn content(explicit: Option<&'a DocumentString>) -> Self {
        Self::new(explicit, None, Mode::Content)
    }
    /// Classifies explicit type or the virtual language prefix for preparation.
    pub fn preparation(
        explicit: Option<&'a DocumentString>,
        language: Option<&'a DocumentString>,
    ) -> Self {
        Self::new(explicit, language, Mode::Preparation)
    }
    fn new(
        explicit: Option<&'a DocumentString>,
        language: Option<&'a DocumentString>,
        mode: Mode,
    ) -> Self {
        Self {
            state: State::Initial {
                explicit,
                language,
                mode,
            },
        }
    }
    fn pending(&mut self) -> Result<HtmlScriptTypeWork, Option<HtmlScriptKind>> {
        match &mut self.state {
            State::Initial { .. } => Ok(HtmlScriptTypeWork::RecordVisit(2)),
            State::Scan(scan) => scan.pending().inspect_err(|&result| {
                self.state = State::Done(result);
            }),
            State::Done(result) => Err(*result),
        }
    }
    /// Charges the next read as one schema change, then performs it. A completed cursor charges
    /// nothing and remains completed. A refused charge leaves the cursor at the same read.
    pub fn advance<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        if let Ok(work) = self.pending() {
            charge.charge(&[work.change()])?;
        }
        self.step();
        Ok(())
    }
    /// Performs one admitted action; a completed cursor remains completed.
    fn step(&mut self) {
        if self.pending().is_err() {
            return;
        }
        match &mut self.state {
            State::Initial {
                explicit,
                language,
                mode,
            } => {
                // An empty or absent selection is the Classic early return;
                // otherwise the scanned value and its virtual prefix are fixed.
                let input = match (*explicit, *language, *mode) {
                    (Some(explicit), _, _) => {
                        (!explicit.is_empty()).then_some((explicit, &b""[..]))
                    }
                    (None, Some(language), Mode::Preparation) => {
                        (!language.is_empty()).then_some((language, &b"text/"[..]))
                    }
                    (None, _, _) => None,
                };
                self.state = match input {
                    None => State::Done(Some(HtmlScriptKind::Classic)),
                    Some((value, prefix)) => State::Scan(Scan {
                        value,
                        prefix,
                        trims: explicit.is_some() && *mode == Mode::Preparation,
                        pass: Pass::RawEssences,
                        stage: Stage::Candidate,
                        candidate: 0,
                        start: 0,
                        end: value.len(),
                    }),
                };
            }
            State::Scan(scan) => scan.advance(),
            State::Done(_) => {}
        }
    }
    /// Complete None is the preparation early-return branch, not missing evidence.
    pub fn result(&self) -> Option<Option<HtmlScriptKind>> {
        match self.state {
            State::Done(result) => Some(result),
            _ => None,
        }
    }
    pub(crate) fn finish_ordinary(mut self) -> Option<HtmlScriptKind> {
        loop {
            match self.pending() {
                Ok(_) => self.step(),
                Err(result) => return result,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_matcher_admits_length_and_only_reached_utf16_comparisons() {
        let mut records = 0;
        let mut units = 0;
        for (raw, expected) in [("HiDdEn", true), ("hiddEX", false), ("other", false)] {
            let value = DocumentString::from(raw);
            let mut cursor = ScriptTypeMatchCursor::new(b"", &value, 0, value.len(), b"hidden");
            let result = loop {
                match cursor.pending() {
                    Ok(HtmlScriptTypeWork::RecordVisit(count)) => records += count,
                    Ok(HtmlScriptTypeWork::InspectByte(count)) => units += count,
                    Err(result) => break result,
                }
                cursor.advance();
            };
            assert_eq!(result, expected);
        }
        assert_eq!(records, 3);
        assert_eq!(units, 36);
    }

    #[test]
    fn borrowed_modes_preserve_only_the_explicit_js_whitespace_exception() {
        let js = DocumentString::from(" \tTEXT/JAVASCRIPT\r\n\x0c ");
        assert_eq!(
            HtmlScriptTypeCursor::content(Some(&js)).finish_ordinary(),
            None
        );
        assert_eq!(
            HtmlScriptTypeCursor::preparation(Some(&js), None).finish_ordinary(),
            Some(HtmlScriptKind::Classic)
        );
        for value in [
            " module ",
            " importmap ",
            " speculationrules ",
            "   ",
            "\u{a0}text/javascript\u{a0}",
        ] {
            let value = DocumentString::from(value);
            assert_eq!(
                HtmlScriptTypeCursor::preparation(Some(&value), None).finish_ordinary(),
                None
            );
        }
    }
    #[test]
    fn virtual_language_and_type_override_are_exact_without_allocating_prefix() {
        let js = DocumentString::from("javascript");
        let bad = DocumentString::from("unknown");
        assert_eq!(
            HtmlScriptTypeCursor::preparation(None, Some(&js)).finish_ordinary(),
            Some(HtmlScriptKind::Classic)
        );
        assert_eq!(
            HtmlScriptTypeCursor::preparation(Some(&bad), Some(&js)).finish_ordinary(),
            None
        );
        let empty = DocumentString::from("");
        assert_eq!(
            HtmlScriptTypeCursor::preparation(Some(&empty), Some(&bad)).finish_ordinary(),
            Some(HtmlScriptKind::Classic)
        );
    }
}

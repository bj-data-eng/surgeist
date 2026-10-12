//! One borrowed owner for HTML script content meaning and preparation classification.
//! HTML877d #attr-script-type and #script-processing-prepare; MIME39aa535 §4.6.
//! The sixteen essence strings are incorporated under the WHATWG BSD permission
//! recorded by this crate's NOTICE. This is not MIME parsing or author validation.

use crate::DocumentString;

/// A script kind selected by HTML's type/language classification step.
/// This value alone does not establish preparation, saved metadata or execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptKind {
    Classic,
    Module,
    ImportMap,
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

/// Required admission before the cursor's next actual action. Parser adapters
/// map these concrete credits to their existing categories; no controller is held.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptTypeWork {
    RecordVisit(u64),
    InspectUnit(u64),
}
/// Borrowed ASCII-insensitive virtual-prefix/range matcher, also consumed by
/// the construction owner's actual hidden-input comparison. Credits precede
/// length comparison and each read; a completed result has no further work.
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
    pub(crate) fn required_work(&self) -> Option<ScriptTypeWork> {
        if self.result.is_some() {
            None
        } else if self.length_pending {
            Some(ScriptTypeWork::RecordVisit(1))
        } else {
            Some(ScriptTypeWork::InspectUnit(
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
            let folded = if (0x41..=0x5a).contains(&actual) {
                actual + 0x20
            } else {
                actual
            };
            if folded != u16::from(self.expected[self.position].to_ascii_lowercase()) {
                self.result = Some(false);
            } else {
                self.position += 1;
                if self.position == self.expected.len() {
                    self.result = Some(true);
                }
            }
        }
    }
    pub(crate) fn result(&self) -> Option<bool> {
        self.result
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    Initial,
    Candidate,
    Match,
    TrimStart,
    TrimEnd,
    Done,
}

/// Stack-only borrowed cursor. Each `advance` follows its `required_work`
/// admission. Raw units are read only by advance, never by credit inspection.
/// Ordinary native reports use the same actions under their ordinary contract.
/// Finite work: sixteen raw matches, at most two linear trim passes and sixteen
/// stripped matches for explicit preparation, then three exact keyword matches.
/// A virtual language prefix never allocates a concatenated string.
pub(crate) struct ScriptTypeCursor<'a> {
    explicit: Option<&'a DocumentString>,
    language: Option<&'a DocumentString>,
    mode: Mode,
    pass: Pass,
    stage: Stage,
    candidate: usize,
    start: usize,
    end: usize,
    matcher: Option<ScriptTypeMatchCursor<'a>>,
    result: Option<HtmlScriptKind>,
}
impl<'a> ScriptTypeCursor<'a> {
    pub(crate) fn content(explicit: Option<&'a DocumentString>) -> Self {
        Self::new(explicit, None, Mode::Content)
    }
    pub(crate) fn preparation(
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
            explicit,
            language,
            mode,
            pass: Pass::RawEssences,
            stage: Stage::Initial,
            candidate: 0,
            start: 0,
            end: 0,
            matcher: None,
            result: None,
        }
    }
    fn value(&self) -> &'a DocumentString {
        self.explicit
            .or(self.language)
            .expect("initial stage selected a nonempty input")
    }
    fn prefix(&self) -> &'static [u8] {
        if self.explicit.is_none() && self.mode == Mode::Preparation {
            b"text/"
        } else {
            b""
        }
    }
    fn expected(&self) -> &'static [u8] {
        match self.pass {
            Pass::RawEssences | Pass::TrimmedEssences => ESSENCES[self.candidate],
            Pass::Keywords => KEYWORDS[self.candidate].0,
        }
    }
    // These zero-work transitions inspect only established cursor fields, not
    // actual units. Scalar progress cannot substitute for an admitted matcher read.
    fn settle(&mut self) {
        loop {
            match self.stage {
                Stage::Match => match self
                    .matcher
                    .as_ref()
                    .expect("candidate installed matcher")
                    .result()
                {
                    Some(true) => {
                        self.result = Some(if self.pass == Pass::Keywords {
                            KEYWORDS[self.candidate].1
                        } else {
                            HtmlScriptKind::Classic
                        });
                        self.stage = Stage::Done;
                    }
                    Some(false) => {
                        self.candidate += 1;
                        self.stage = Stage::Candidate;
                    }
                    None => break,
                },
                Stage::Candidate
                    if self.pass != Pass::Keywords && self.candidate == ESSENCES.len() =>
                {
                    if self.pass == Pass::RawEssences
                        && self.mode == Mode::Preparation
                        && self.explicit.is_some()
                    {
                        self.stage = Stage::TrimStart;
                    } else {
                        self.pass = Pass::Keywords;
                        self.candidate = 0;
                        self.start = 0;
                        self.end = self.value().len();
                    }
                }
                Stage::Candidate
                    if self.pass == Pass::Keywords && self.candidate == KEYWORDS.len() =>
                {
                    self.stage = Stage::Done
                }
                Stage::TrimStart if self.start == self.end => self.stage = Stage::TrimEnd,
                Stage::TrimEnd if self.start == self.end => {
                    self.pass = Pass::TrimmedEssences;
                    self.candidate = 0;
                    self.stage = Stage::Candidate;
                }
                _ => break,
            }
        }
    }
    pub(crate) fn required_work(&mut self) -> Option<ScriptTypeWork> {
        self.settle();
        match self.stage {
            Stage::Initial => Some(ScriptTypeWork::RecordVisit(2)),
            Stage::Candidate => Some(ScriptTypeWork::RecordVisit(1)),
            Stage::Match => self
                .matcher
                .as_ref()
                .expect("candidate installed matcher")
                .required_work(),
            Stage::TrimStart | Stage::TrimEnd => Some(ScriptTypeWork::InspectUnit(2)),
            Stage::Done => None,
        }
    }
    pub(crate) fn advance(&mut self) {
        match self.stage {
            Stage::Initial => {
                if self.explicit.is_some_and(DocumentString::is_empty)
                    || (self.explicit.is_none()
                        && (self.mode == Mode::Content
                            || self.language.is_none_or(DocumentString::is_empty)))
                {
                    self.result = Some(HtmlScriptKind::Classic);
                    self.stage = Stage::Done;
                } else {
                    self.end = self.value().len();
                    self.stage = Stage::Candidate;
                }
            }
            Stage::Candidate => {
                self.matcher = Some(ScriptTypeMatchCursor::new(
                    self.prefix(),
                    self.value(),
                    self.start,
                    self.end,
                    self.expected(),
                ));
                self.stage = Stage::Match;
            }
            Stage::Match => self
                .matcher
                .as_mut()
                .expect("candidate installed matcher")
                .advance(),
            Stage::TrimStart => {
                if matches!(
                    self.value().code_units()[self.start],
                    0x09 | 0x0a | 0x0c | 0x0d | 0x20
                ) {
                    self.start += 1;
                } else {
                    self.stage = Stage::TrimEnd;
                }
            }
            Stage::TrimEnd => {
                if matches!(
                    self.value().code_units()[self.end - 1],
                    0x09 | 0x0a | 0x0c | 0x0d | 0x20
                ) {
                    self.end -= 1;
                } else {
                    self.pass = Pass::TrimmedEssences;
                    self.candidate = 0;
                    self.stage = Stage::Candidate;
                }
            }
            Stage::Done => unreachable!("only a pending cursor action can advance"),
        }
    }
    /// Complete None is the preparation early-return branch, not missing evidence.
    pub(crate) fn result(&self) -> Option<Option<HtmlScriptKind>> {
        (self.stage == Stage::Done).then_some(self.result)
    }
    pub(crate) fn finish_ordinary(mut self) -> Option<HtmlScriptKind> {
        while self.required_work().is_some() {
            self.advance();
        }
        self.result().expect("all cursor actions completed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borrowed_modes_preserve_only_the_explicit_js_whitespace_exception() {
        let js = DocumentString::from(" \tTEXT/JAVASCRIPT\r\n\x0c ");
        assert_eq!(ScriptTypeCursor::content(Some(&js)).finish_ordinary(), None);
        assert_eq!(
            ScriptTypeCursor::preparation(Some(&js), None).finish_ordinary(),
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
                ScriptTypeCursor::preparation(Some(&value), None).finish_ordinary(),
                None
            );
        }
    }
    #[test]
    fn virtual_language_and_type_override_are_exact_without_allocating_prefix() {
        let js = DocumentString::from("javascript");
        let bad = DocumentString::from("unknown");
        assert_eq!(
            ScriptTypeCursor::preparation(None, Some(&js)).finish_ordinary(),
            Some(HtmlScriptKind::Classic)
        );
        assert_eq!(
            ScriptTypeCursor::preparation(Some(&bad), Some(&js)).finish_ordinary(),
            None
        );
        let empty = DocumentString::from("");
        assert_eq!(
            ScriptTypeCursor::preparation(Some(&empty), Some(&bad)).finish_ordinary(),
            Some(HtmlScriptKind::Classic)
        );
    }
}

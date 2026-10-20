//! Replay of the 13 bundled html5lib-tests `f994590f` tokenizer `.test` files:
//! every top-level upstream tokenizer file except `xmlViolation.test`, which
//! is not bundled because it describes infoset coercion, not this tokenizer.
//!
//! Each case runs through the actual private tokenizer with the shared
//! [`collect`] owner checks. Expectations come from the retained corpus:
//! coalesced character data, start tags (attribute set and self-closing flag),
//! end-tag names, comments, doctypes (force-quirks as corpus correctness) and
//! the ordered error codes. Error line/column metadata is not compared. EOF is
//! not a corpus token; processing-instruction tokens remain actual output, so a
//! corpus bogus-comment expectation for `<?` diverges instead of being erased.
//! Such a case is a spec exclusion whose tokens and ordered error codes are
//! instead derived by hand from the pinned HTML877d tokenizer and asserted
//! exactly. A tokenizer panic fails every case, excluded or not.
//!
//! Inputs keep exact UTF-16 code units, including isolated surrogates. Every
//! case runs from UTF-16 code units, and also from UTF-8 when its input has
//! only scalar values.

use super::*;

/// Why one bundled case is not asserted to match.
#[derive(Clone, Copy, Debug)]
enum Exclusion {
    /// The pinned HTML edition defines different output. The case must still
    /// diverge from the corpus, and every run, in each listed initial state
    /// and both encodings, must equal the tokens (rendered as
    /// [`show_tokens`]) and ordered error codes derived from the pinned
    /// edition, so a stale record, a panic or any unrelated divergence fails.
    Spec {
        reason: &'static str,
        tokens: &'static str,
        errors: &'static [&'static str],
    },
    /// A tracked divergence without a fix yet. Its output is not asserted
    /// either way, but a panic still fails.
    Defect(&'static str),
}

/// HTML877d tag open state: `<?` sets the temporary buffer to empty and enters
/// the processing instruction open state, not the historical bogus comment
/// with unexpected-question-mark-instead-of-tag-name the corpus expects.
/// Processing instruction open: ASCII alpha or `_` is reconsumed as the
/// target; EOF is eof-in-processing-instruction and emits only end-of-file;
/// anything else is invalid-first-character-of-processing-instruction-target,
/// converts the temporary buffer to a comment whose data is `?` and reconsumes
/// in the bogus comment state. A target runs over ASCII alphanumerics, `-`
/// and `_` to whitespace, `?` or `>`; EOF in the target state also emits only
/// end-of-file.
const fn pi(tokens: &'static str, errors: &'static [&'static str]) -> Exclusion {
    Exclusion::Spec {
        reason: "HTML877d tag open state: U+003F enters processing-instruction open state",
        tokens,
        errors,
    }
}
const INVALID_FIRST: &[&str] = &["invalid-first-character-of-processing-instruction-target"];
const EOF_IN_PI: &[&str] = &["eof-in-processing-instruction"];
/// Bogus comment state: U+0000 is unexpected-null-character, appending U+FFFD.
const INVALID_FIRST_THEN_NULL: &[&str] = &[
    "invalid-first-character-of-processing-instruction-target",
    "unexpected-null-character",
];

/// Exclusions keyed by bundled file and one-based case ordinal. Every spec
/// case lists only the data state. Spec renderings are [`show`] ASCII.
const EXCLUSIONS: &[(&str, usize, Exclusion)] = &[
    // Preprocessing normalizes CR and CRLF to LF before the PI open state.
    ("domjs.test", 1, pi(r#"Comment("?\u{a}")"#, INVALID_FIRST)),
    ("domjs.test", 2, pi(r#"Comment("?\u{a}")"#, INVALID_FIRST)),
    (
        "domjs.test",
        3,
        pi(r#"Comment("?\u{a}\u{a}")"#, INVALID_FIRST),
    ),
    // `>` ends the target; after-target reconsumes it in PI data, which emits.
    ("test2.test", 32, pi(r#"Pi("namespace", "")"#, &[])),
    ("test2.test", 33, pi(r#"Pi("foo--", "")"#, &[])),
    (
        "test3.test",
        80,
        Exclusion::Defect(
            "#1345: control-character-in-input-stream for the character after `<!` is reported after incorrectly-opened-comment; the corpus reports it first",
        ),
    ),
    ("test3.test", 1159, pi("", EOF_IN_PI)),
    (
        "test3.test",
        1160,
        pi(r#"Comment("?\u{fffd}")"#, INVALID_FIRST_THEN_NULL),
    ),
    (
        "test3.test",
        1161,
        pi(r#"Comment("?\u{9}")"#, INVALID_FIRST),
    ),
    (
        "test3.test",
        1162,
        pi(r#"Comment("?\u{a}")"#, INVALID_FIRST),
    ),
    // The input-stream error for U+000B is reported when the PI open state
    // consumes it, before that state's own error.
    (
        "test3.test",
        1163,
        pi(
            r#"Comment("?\u{b}")"#,
            &[
                "control-character-in-input-stream",
                "invalid-first-character-of-processing-instruction-target",
            ],
        ),
    ),
    (
        "test3.test",
        1164,
        pi(r#"Comment("?\u{c}")"#, INVALID_FIRST),
    ),
    ("test3.test", 1165, pi(r#"Comment("? ")"#, INVALID_FIRST)),
    (
        "test3.test",
        1166,
        pi(r#"Comment("? \u{fffd}")"#, INVALID_FIRST_THEN_NULL),
    ),
    ("test3.test", 1167, pi(r#"Comment("?!")"#, INVALID_FIRST)),
    (
        "test3.test",
        1168,
        pi(r#"Comment("?\u{22}")"#, INVALID_FIRST),
    ),
    ("test3.test", 1169, pi(r#"Comment("?&")"#, INVALID_FIRST)),
    ("test3.test", 1170, pi(r#"Comment("?'")"#, INVALID_FIRST)),
    ("test3.test", 1171, pi(r#"Comment("?-")"#, INVALID_FIRST)),
    ("test3.test", 1172, pi(r#"Comment("?/")"#, INVALID_FIRST)),
    ("test3.test", 1173, pi(r#"Comment("?0")"#, INVALID_FIRST)),
    ("test3.test", 1174, pi(r#"Comment("?1")"#, INVALID_FIRST)),
    ("test3.test", 1175, pi(r#"Comment("?9")"#, INVALID_FIRST)),
    ("test3.test", 1176, pi(r#"Comment("?<")"#, INVALID_FIRST)),
    ("test3.test", 1177, pi(r#"Comment("?=")"#, INVALID_FIRST)),
    // Bogus comment state: the reconsumed `>` emits the `?` comment.
    ("test3.test", 1178, pi(r#"Comment("?")"#, INVALID_FIRST)),
    ("test3.test", 1179, pi(r#"Comment("??")"#, INVALID_FIRST)),
    ("test3.test", 1180, pi(r#"Comment("?@")"#, INVALID_FIRST)),
    // An ASCII alpha starts the target; EOF there emits no token.
    ("test3.test", 1181, pi("", EOF_IN_PI)),
    ("test3.test", 1182, pi("", EOF_IN_PI)),
    ("test3.test", 1183, pi("", EOF_IN_PI)),
    ("test3.test", 1184, pi("", EOF_IN_PI)),
    ("test3.test", 1185, pi(r#"Comment("?`")"#, INVALID_FIRST)),
    ("test3.test", 1186, pi("", EOF_IN_PI)),
    ("test3.test", 1187, pi("", EOF_IN_PI)),
    ("test3.test", 1188, pi("", EOF_IN_PI)),
    ("test3.test", 1189, pi("", EOF_IN_PI)),
    ("test3.test", 1190, pi(r#"Comment("?{")"#, INVALID_FIRST)),
    // U+100000 is a private-use scalar value, not a noncharacter.
    (
        "test3.test",
        1191,
        pi(r#"Comment("?\u{100000}")"#, INVALID_FIRST),
    ),
];

#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number,
    String(Vec<u16>),
    Array(Vec<Json>),
    Object(Vec<(Vec<u16>, Json)>),
}

/// Minimal test-only JSON reader. String escapes produce exact UTF-16 code
/// units, so a `\uD800` escape stays an isolated surrogate.
struct Reader<'a> {
    text: &'a str,
    at: usize,
}
impl Reader<'_> {
    fn parse(text: &str) -> Result<Json, String> {
        let mut reader = Reader { text, at: 0 };
        let value = reader.value()?;
        reader.space();
        if reader.at != text.len() {
            return Err(format!("trailing data at byte {}", reader.at));
        }
        Ok(value)
    }
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }
    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }
    fn expect(&mut self, byte: u8) -> Result<(), String> {
        self.space();
        if self.peek() == Some(byte) {
            self.at += 1;
            Ok(())
        } else {
            Err(format!("expected {:?} at byte {}", byte as char, self.at))
        }
    }
    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self.text[self.at..].starts_with(word) {
            self.at += word.len();
            Ok(value)
        } else {
            Err(format!("invalid literal at byte {}", self.at))
        }
    }
    fn value(&mut self) -> Result<Json, String> {
        self.space();
        match self.peek() {
            Some(b'{') => {
                self.at += 1;
                let mut members = Vec::new();
                self.space();
                if self.peek() == Some(b'}') {
                    self.at += 1;
                    return Ok(Json::Object(members));
                }
                loop {
                    self.space();
                    let Json::String(key) = self.value()? else {
                        return Err(format!("object key at byte {}", self.at));
                    };
                    self.expect(b':')?;
                    members.push((key, self.value()?));
                    self.space();
                    match self.peek() {
                        Some(b',') => self.at += 1,
                        Some(b'}') => {
                            self.at += 1;
                            return Ok(Json::Object(members));
                        }
                        _ => return Err(format!("object at byte {}", self.at)),
                    }
                }
            }
            Some(b'[') => {
                self.at += 1;
                let mut items = Vec::new();
                self.space();
                if self.peek() == Some(b']') {
                    self.at += 1;
                    return Ok(Json::Array(items));
                }
                loop {
                    items.push(self.value()?);
                    self.space();
                    match self.peek() {
                        Some(b',') => self.at += 1,
                        Some(b']') => {
                            self.at += 1;
                            return Ok(Json::Array(items));
                        }
                        _ => return Err(format!("array at byte {}", self.at)),
                    }
                }
            }
            Some(b'"') => {
                self.at += 1;
                self.string()
            }
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'n') => self.literal("null", Json::Null),
            Some(b'-' | b'0'..=b'9') => {
                while matches!(
                    self.peek(),
                    Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                ) {
                    self.at += 1;
                }
                Ok(Json::Number)
            }
            _ => Err(format!("value at byte {}", self.at)),
        }
    }
    fn string(&mut self) -> Result<Json, String> {
        let mut units = Vec::new();
        loop {
            let ch = self.text[self.at..]
                .chars()
                .next()
                .ok_or("unterminated string")?;
            self.at += ch.len_utf8();
            match ch {
                '"' => return Ok(Json::String(units)),
                '\\' => {
                    let escape = self.peek().ok_or("unterminated escape")?;
                    self.at += 1;
                    units.push(match escape {
                        b'"' => 0x22,
                        b'\\' => 0x5c,
                        b'/' => 0x2f,
                        b'b' => 0x08,
                        b'f' => 0x0c,
                        b'n' => 0x0a,
                        b'r' => 0x0d,
                        b't' => 0x09,
                        b'u' => {
                            let hex = self
                                .text
                                .get(self.at..self.at + 4)
                                .ok_or("short unicode escape")?;
                            self.at += 4;
                            u16::from_str_radix(hex, 16).map_err(|e| e.to_string())?
                        }
                        _ => return Err(format!("escape at byte {}", self.at)),
                    });
                }
                ch if (ch as u32) < 0x20 => return Err("control character in string".into()),
                ch => units.extend(ch.encode_utf16(&mut [0; 2]).iter()),
            }
        }
    }
}

/// The corpus `doubleEscaped` second round: each `\uHHHH` sequence becomes
/// that code unit, preserving isolated surrogates.
fn unescape(units: &[u16]) -> Vec<u16> {
    let mut result = Vec::with_capacity(units.len());
    let mut i = 0;
    while i < units.len() {
        if units[i] == 0x5c
            && units.get(i + 1) == Some(&0x75)
            && let Some(hex) = units.get(i + 2..i + 6)
            && let Ok(hex) = String::from_utf16(hex)
            && let Ok(unit) = u16::from_str_radix(&hex, 16)
        {
            result.push(unit);
            i += 6;
        } else {
            result.push(units[i]);
            i += 1;
        }
    }
    result
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Character(Vec<u16>),
    StartTag(Vec<u16>, Vec<(Vec<u16>, Vec<u16>)>, bool),
    EndTag(Vec<u16>),
    Comment(Vec<u16>),
    Doctype(Option<Vec<u16>>, Option<Vec<u16>>, Option<Vec<u16>>, bool),
    /// Actual output only; the corpus has no processing-instruction token.
    Pi(Vec<u16>, Vec<u16>),
}

struct Case {
    ordinal: usize,
    description: String,
    input: Vec<u16>,
    output: Vec<Token>,
    states: Vec<String>,
    last_start: Option<String>,
    errors: Vec<String>,
}

fn member<'a>(object: &'a [(Vec<u16>, Json)], key: &str) -> Option<&'a Json> {
    let key = units(key);
    object.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
}
fn string(value: &Json, double: bool) -> Result<Vec<u16>, String> {
    match value {
        Json::String(units) if double => Ok(unescape(units)),
        Json::String(units) => Ok(units.clone()),
        other => Err(format!("expected string, found {other:?}")),
    }
}
fn optional(value: &Json, double: bool) -> Result<Option<Vec<u16>>, String> {
    match value {
        Json::Null => Ok(None),
        value => string(value, double).map(Some),
    }
}
fn scalar(value: &Json) -> Result<String, String> {
    String::from_utf16(&string(value, false)?).map_err(|e| e.to_string())
}

fn expected_token(value: &Json, double: bool) -> Result<Token, String> {
    let Json::Array(items) = value else {
        return Err(format!("token {value:?}"));
    };
    let kind = items.first().map(scalar).transpose()?.unwrap_or_default();
    let field = |i: usize| items.get(i).ok_or(format!("{kind} field {i}"));
    Ok(match (kind.as_str(), items.len()) {
        ("Character", 2) => Token::Character(string(field(1)?, double)?),
        ("Comment", 2) => Token::Comment(string(field(1)?, double)?),
        ("EndTag", 2) => Token::EndTag(string(field(1)?, double)?),
        ("StartTag", 3 | 4) => {
            let Json::Object(attributes) = field(2)? else {
                return Err("start-tag attributes".into());
            };
            let mut attributes = attributes
                .iter()
                .map(|(name, value)| {
                    let name = if double { unescape(name) } else { name.clone() };
                    Ok((name, string(value, double)?))
                })
                .collect::<Result<Vec<_>, String>>()?;
            attributes.sort();
            let self_closing = match items.get(3) {
                None => false,
                Some(Json::Bool(flag)) => *flag,
                Some(other) => return Err(format!("self-closing {other:?}")),
            };
            Token::StartTag(string(field(1)?, double)?, attributes, self_closing)
        }
        ("DOCTYPE", 5) => {
            let Json::Bool(correct) = field(4)? else {
                return Err("doctype correctness".into());
            };
            Token::Doctype(
                optional(field(1)?, double)?,
                optional(field(2)?, double)?,
                optional(field(3)?, double)?,
                *correct,
            )
        }
        _ => return Err(format!("unknown token {value:?}")),
    })
}

fn cases(raw: &str) -> Result<Vec<Case>, String> {
    let Json::Object(top) = Reader::parse(raw)? else {
        return Err("top-level object".into());
    };
    let [(key, Json::Array(tests))] = top.as_slice() else {
        return Err("single tests member".into());
    };
    if *key != units("tests") {
        return Err("tests member".into());
    }
    tests
        .iter()
        .enumerate()
        .map(|(index, test)| {
            let Json::Object(test) = test else {
                return Err(format!("case {}", index + 1));
            };
            let double = matches!(member(test, "doubleEscaped"), Some(Json::Bool(true)));
            let field = |key: &str| member(test, key).ok_or(format!("case {}: {key}", index + 1));
            let Json::Array(output) = field("output")? else {
                return Err("output".into());
            };
            let states = match member(test, "initialStates") {
                None => vec!["Data state".to_owned()],
                Some(Json::Array(states)) => states.iter().map(scalar).collect::<Result<_, _>>()?,
                Some(other) => return Err(format!("initialStates {other:?}")),
            };
            let errors = match member(test, "errors") {
                None => Vec::new(),
                Some(Json::Array(errors)) => errors
                    .iter()
                    .map(|error| match error {
                        Json::Object(error) => scalar(member(error, "code").ok_or("error code")?),
                        other => Err(format!("error {other:?}")),
                    })
                    .collect::<Result<_, _>>()?,
                Some(other) => return Err(format!("errors {other:?}")),
            };
            Ok(Case {
                ordinal: index + 1,
                description: scalar(field("description")?)?,
                input: string(field("input")?, double)?,
                output: output
                    .iter()
                    .map(|token| expected_token(token, double))
                    .collect::<Result<_, _>>()?,
                states,
                last_start: member(test, "lastStartTag").map(scalar).transpose()?,
                errors,
            })
        })
        .collect()
}

fn state(name: &str) -> Result<State, String> {
    Ok(match name {
        "Data state" => State::Data,
        "PLAINTEXT state" => State::Plaintext,
        "RCDATA state" => State::Raw(Raw::Rcdata),
        "RAWTEXT state" => State::Raw(Raw::Rawtext),
        "Script data state" => State::Raw(Raw::Script),
        "CDATA section state" => State::Cdata,
        other => return Err(format!("unknown initial state {other:?}")),
    })
}

/// Corpus-shaped actual tokens: adjacent character data coalesced, attribute
/// set sorted by name, end-tag attributes and flags not carried, EOF dropped.
fn actual_tokens(seen: Vec<Seen>) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    for seen in seen {
        let token = match seen {
            Seen::Text(text) => {
                if let Some(Token::Character(previous)) = tokens.last_mut() {
                    previous.extend(text);
                    continue;
                }
                Token::Character(text)
            }
            Seen::Tag(TagKind::Start, name, self_closing, mut attributes, _) => {
                attributes.sort();
                Token::StartTag(name, attributes, self_closing)
            }
            Seen::Tag(TagKind::End, name, ..) => Token::EndTag(name),
            Seen::Comment(data) => Token::Comment(data),
            Seen::Doctype(name, public, system, force_quirks) => {
                Token::Doctype(name, public, system, !force_quirks)
            }
            Seen::Pi(target, data) => Token::Pi(target, data),
            Seen::Eof => continue,
        };
        tokens.push(token);
    }
    tokens
}

/// Printable ASCII other than `"` and `\` stays literal; every other scalar
/// value is `\u{…}` and an isolated surrogate is `\u{HHHH}`, so a rendering is
/// unambiguous ASCII.
fn show(units: &[u16]) -> String {
    char::decode_utf16(units.iter().copied())
        .map(|r| match r {
            Ok(c @ ' '..='~') if c != '"' && c != '\\' => c.to_string(),
            Ok(c) => c.escape_unicode().to_string(),
            Err(e) => format!("\\u{{{:04X}}}", e.unpaired_surrogate()),
        })
        .collect()
}
fn show_tokens(tokens: &[Token]) -> String {
    let show_opt = |v: &Option<Vec<u16>>| {
        v.as_deref()
            .map_or("null".into(), |v| format!("\"{}\"", show(v)))
    };
    tokens
        .iter()
        .map(|t| match t {
            Token::Character(d) => format!("Character(\"{}\")", show(d)),
            Token::StartTag(n, a, s) => format!(
                "StartTag(\"{}\", [{}]{})",
                show(n),
                a.iter()
                    .map(|(k, v)| format!("\"{}\"=\"{}\"", show(k), show(v)))
                    .collect::<Vec<_>>()
                    .join(", "),
                if *s { ", self-closing" } else { "" }
            ),
            Token::EndTag(n) => format!("EndTag(\"{}\")", show(n)),
            Token::Comment(d) => format!("Comment(\"{}\")", show(d)),
            Token::Doctype(n, p, s, c) => format!(
                "DOCTYPE({}, {}, {}, {c})",
                show_opt(n),
                show_opt(p),
                show_opt(s)
            ),
            Token::Pi(t, d) => format!("Pi(\"{}\", \"{}\")", show(t), show(d)),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// One run of a case: its label and either the corpus-shaped tokens with the
/// ordered error codes, or the panic message.
struct Run {
    label: String,
    utf8: bool,
    outcome: Result<(Vec<Token>, Vec<&'static str>), String>,
}

/// Every run of one case: each initial state, UTF-16 always and UTF-8 when the
/// input is scalar-only.
fn runs(case: &Case) -> Result<Vec<Run>, String> {
    let mut runs = Vec::new();
    let scalar_input = String::from_utf16(&case.input).ok();
    for name in &case.states {
        let initial = state(name)?;
        let mut sources = vec![(
            "utf16",
            HtmlSource::utf16(DocumentString::from_utf16(case.input.clone())),
        )];
        if let Some(input) = &scalar_input {
            sources.push(("utf8", HtmlSource::utf8(input.as_str())));
        }
        for (encoding, source) in &sources {
            let last = case.last_start.as_deref();
            let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                collect(source, initial, last, initial == State::Cdata)
            }));
            let outcome = match run {
                Err(panic) => Err(panic
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| panic.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                    .unwrap_or_default()),
                Ok((seen, codes)) => Ok((
                    actual_tokens(seen),
                    codes.iter().map(|code| code.name()).collect(),
                )),
            };
            runs.push(Run {
                label: format!("[{name}, {encoding}]"),
                utf8: *encoding == "utf8",
                outcome,
            });
        }
    }
    Ok(runs)
}

/// How one run differs from the expected rendering and error codes, if it does.
fn differs(run: &Run, tokens: &str, errors: &[&str]) -> Option<String> {
    match &run.outcome {
        Err(panic) => Some(format!("{}\n    actual tokenizer panicked: {panic}", run.label)),
        Ok((actual, codes)) => (show_tokens(actual) != tokens || codes != errors).then(|| {
            format!(
                "{}\n    expected tokens: {tokens}\n    actual tokens:   {}\n    expected errors: {errors:?}\n    actual errors:   {codes:?}",
                run.label,
                show_tokens(actual),
            )
        }),
    }
}

/// The runs of one case that diverge from its corpus expectation.
fn divergences(case: &Case, runs: &[Run]) -> Vec<String> {
    let tokens = show_tokens(&case.output);
    let errors: Vec<&str> = case.errors.iter().map(String::as_str).collect();
    runs.iter()
        .filter_map(|run| differs(run, &tokens, &errors))
        .collect()
}

/// Every run of one case, returning the runs divergent from the corpus.
fn replay_case(case: &Case) -> Result<Vec<String>, String> {
    Ok(divergences(case, &runs(case)?))
}

fn replay(file: &str, raw: &str, count: usize) {
    let cases = cases(raw).unwrap_or_else(|e| panic!("{file}: {e}"));
    // Every retained case of the complete pinned file is replayed.
    assert_eq!(cases.len(), count, "{file}");
    let mut failures = Vec::new();
    for (excluded, ordinal, _) in EXCLUSIONS {
        if *excluded == file && (*ordinal == 0 || *ordinal > cases.len()) {
            failures.push(format!("{file}/{ordinal}: exclusion names no bundled case"));
        }
    }
    for case in &cases {
        let runs = runs(case).unwrap_or_else(|e| panic!("{file}/{}: {e}", case.ordinal));
        let divergences = divergences(case, &runs);
        let exclusion = EXCLUSIONS
            .iter()
            .find(|(f, o, _)| *f == file && *o == case.ordinal)
            .map(|(_, _, e)| *e);
        let context = || {
            format!(
                "{file}/{} {:?} input \"{}\"",
                case.ordinal,
                case.description,
                show(&case.input)
            )
        };
        match exclusion {
            None if divergences.is_empty() => {}
            None => failures.push(format!("{}\n  {}", context(), divergences.join("\n  "))),
            Some(Exclusion::Spec { reason, .. }) if divergences.is_empty() => failures.push(
                format!("{} now matches; stale spec exclusion: {reason}", context()),
            ),
            Some(Exclusion::Spec {
                reason,
                tokens,
                errors,
            }) => {
                if !runs.iter().any(|run| run.utf8) {
                    failures.push(format!("{}: spec exclusion has no UTF-8 run", context()));
                }
                let differences: Vec<String> = runs
                    .iter()
                    .filter_map(|run| differs(run, tokens, errors))
                    .collect();
                if !differences.is_empty() {
                    failures.push(format!(
                        "{} differs from its pinned output ({reason})\n  {}",
                        context(),
                        differences.join("\n  ")
                    ));
                }
            }
            Some(Exclusion::Defect(_)) => {
                for run in &runs {
                    if let Err(panic) = &run.outcome {
                        failures.push(format!(
                            "{} {}: actual tokenizer panicked: {panic}",
                            context(),
                            run.label
                        ));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} divergent {file} cases:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

macro_rules! bundled {
    ($($test:ident => $file:literal, $count:literal;)*) => {
        const BUNDLED: &[&str] = &[$($file),*];
        $(
            #[test]
            fn $test() {
                replay(
                    $file,
                    include_str!(concat!(
                        "../../../../tests/fixtures/html5lib-f994590f/tokenizer/",
                        $file
                    )),
                    $count,
                );
            }
        )*
    };
}

bundled! {
    content_model_flags_cases_match_tokens_and_ordered_error_codes => "contentModelFlags.test", 14;
    domjs_cases_match_tokens_and_ordered_error_codes => "domjs.test", 43;
    entities_cases_match_tokens_and_ordered_error_codes => "entities.test", 80;
    escape_flag_cases_match_tokens_and_ordered_error_codes => "escapeFlag.test", 5;
    named_entities_cases_match_tokens_and_ordered_error_codes => "namedEntities.test", 4210;
    numeric_entities_cases_match_tokens_and_ordered_error_codes => "numericEntities.test", 336;
    pending_spec_changes_cases_match_tokens_and_ordered_error_codes => "pendingSpecChanges.test", 1;
    test1_cases_match_tokens_and_ordered_error_codes => "test1.test", 69;
    test2_cases_match_tokens_and_ordered_error_codes => "test2.test", 45;
    test3_cases_match_tokens_and_ordered_error_codes => "test3.test", 1590;
    test4_cases_match_tokens_and_ordered_error_codes => "test4.test", 85;
    unicode_chars_cases_match_tokens_and_ordered_error_codes => "unicodeChars.test", 323;
    unicode_chars_problematic_cases_match_tokens_and_ordered_error_codes => "unicodeCharsProblematic.test", 5;
}

#[test]
fn exclusions_name_only_bundled_files_once_with_reasons() {
    for (i, (file, ordinal, exclusion)) in EXCLUSIONS.iter().enumerate() {
        assert!(BUNDLED.contains(file), "{file}");
        let (Exclusion::Spec { reason, .. } | Exclusion::Defect(reason)) = exclusion;
        assert!(!reason.is_empty(), "{file}/{ordinal} has no reason");
        assert!(
            !EXCLUSIONS[..i]
                .iter()
                .any(|(f, o, _)| f == file && o == ordinal),
            "{file}/{ordinal} repeated"
        );
    }
}

#[test]
fn reader_keeps_isolated_surrogate_escapes_and_double_unescapes_units() {
    let Json::Array(items) = Reader::parse(r#"["\uD800aé", "\\uDC00\\u0041", "é"]"#).unwrap()
    else {
        panic!("array")
    };
    assert_eq!(items[0], Json::String(vec![0xd800, 0x61, 0xe9]));
    let Json::String(double) = &items[1] else {
        panic!("string")
    };
    assert_eq!(unescape(double), [0xdc00, 0x41]);
    assert_eq!(items[2], Json::String(vec![0xe9]));
    assert!(Reader::parse("[1, ]").is_err());
}

#[test]
fn panicked_run_differs_from_every_expectation_and_renderings_are_ascii() {
    let run = Run {
        label: "[Data state, utf16]".into(),
        utf8: false,
        outcome: Err("boom".into()),
    };
    assert!(differs(&run, "", &[]).is_some());
    let comment = [Token::Comment(vec![0x3f, 0x0a, 0x22, 0xfffd, 0xd800])];
    assert_eq!(
        show_tokens(&comment),
        r#"Comment("?\u{a}\u{22}\u{fffd}\u{D800}")"#
    );
}

#[test]
fn changed_corpus_token_or_error_order_is_reported_as_divergence() {
    let raw = r#"{"tests": [{"description": "x", "input": "<a b=c b=d>&amp",
        "output": [["StartTag", "a", {"b": "c"}], ["Character", "&"]],
        "errors": [{"code": "duplicate-attribute", "line": 1, "col": 1},
                   {"code": "missing-semicolon-after-character-reference", "line": 1, "col": 1}]}]}"#;
    let mut case = cases(raw).unwrap().remove(0);
    assert!(replay_case(&case).unwrap().is_empty());
    case.errors.swap(0, 1);
    assert_eq!(replay_case(&case).unwrap().len(), 2);
    case.errors.swap(0, 1);
    case.output[0] = Token::StartTag(units("a"), vec![(units("b"), units("d"))], false);
    assert_eq!(replay_case(&case).unwrap().len(), 2);
}

//! Byte decoding with declared encoding hints, then parsing of the decoded snapshot.
//!
//! The case keeps the exact original bytes, the protocol label and environment
//! encoding the host supplies, and the decode limits. Bytes never pass through
//! lossy UTF-8 conversion on the way to the decoder; the expectations decode by
//! their own rules (Encoding Standard UTF-8 and UTF-16 decoders) and select the
//! encoding by CSS Syntax 3 section 3.2 as the crate reference states it.
use crate::{
    check::{self, Check, Violation},
    codec::{Reader, Writer},
    cursor::Cursor,
    descriptor, error, evaluated, rejected, violation,
};
use surgeist_css::{
    CssParser, CssParserContext, CssParserMode,
    component::CssComponentValueLimits,
    diagnostic::{CssErrorCode, CssRecoveryAction},
    source::CssDecodedStylesheetInput,
};
use surgeist_css_parser::input::{DecodeHints, DecodeLimits, Encoding, EncodingSelection};
use surgeist_fuzz::{
    CaseOutcome, CaseTarget, Generation, Result, TargetDescriptor,
    generation::{Budget, BudgetCharge},
};

#[cfg(test)]
mod tests;

pub(crate) const MAGIC: [u8; 4] = *b"CSBD";
const MAX_BYTES: usize = 4096;
const MAX_LABEL: usize = 64;
const MAX_CASE: usize = 8192;
const NONE: u8 = 0xff;

const SELECTION: &str = "surgeist-css::encoding-selection";
const TEXT: &str = "surgeist-css::decoded-text";
const LIMITS: &str = "surgeist-css::decode-limits";

/// Environment encodings a host may supply, by Encoding Standard label.
const ENVIRONMENT: [&str; 9] = [
    "utf-8",
    "utf-16le",
    "utf-16be",
    "windows-1252",
    "iso-8859-2",
    "shift_jis",
    "euc-kr",
    "iso-2022-jp",
    "hz-gb-2312",
];
/// Protocol labels, including spellings and names that are not labels at all.
const PROTOCOL: [&str; 12] = [
    "utf-8",
    " UTF-8\n",
    "utf-16le",
    "utf-16be",
    "latin1",
    "iso-8859-2",
    "shift_jis",
    "hz-gb-2312",
    "no-such-encoding",
    "",
    "utf-7",
    "x-user-defined",
];
/// Labels placed in a leading `@charset` declaration.
const DECLARED: [&str; 8] = [
    "utf-8",
    "UTF-8",
    "windows-1252",
    "utf-16le",
    "iso-8859-2",
    "hz-gb-2312",
    "shift_jis",
    "unknown-label",
];
const SNIPPETS: [&str; 12] = [
    "a{b:c}",
    ".k{color:red}",
    "/*c*/",
    "@media screen{a{}}",
    "b{content:\"x\\\"y\"}",
    "x{y:\\\n}",
    "u{v:url(a b)}",
    "\n",
    "\r\n",
    "\r",
    "\u{c}",
    "e{--w: 1px}",
];
/// Byte runs that stress each decoder: malformed UTF-8, high bytes, NUL.
const RAW: [&[u8]; 14] = [
    b"\xc0\x80",
    b"\xed\xa0\x80",
    b"\xf4\x90\x80\x80",
    b"\xe2\x82",
    b"\xe2\x82\xac",
    b"\xf0\x9f\x98\x80",
    b"\xe9",
    b"\xa0\xff",
    b"\x80\x9f",
    b"\x00",
    b"\xfe\xff",
    b"\xff\xfe",
    b"\xef\xbb\xbf",
    b"\x8e\xa1",
];

fn environment(index: u8) -> Encoding {
    Encoding::for_label(ENVIRONMENT[usize::from(index)]).expect("registered environment label")
}

/// A checked, bounded concrete case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ByteCase {
    bytes: Vec<u8>,
    protocol: Option<String>,
    environment: Option<u8>,
    original_limit: u32,
    decoded_limit: u32,
}

impl ByteCase {
    /// Builds a case; bounds and table indices are checked before use.
    pub fn new(
        bytes: Vec<u8>,
        protocol: Option<String>,
        environment: Option<u8>,
        original_limit: u32,
        decoded_limit: u32,
    ) -> Result<Self> {
        if bytes.len() > MAX_BYTES
            || protocol
                .as_ref()
                .is_some_and(|label| label.len() > MAX_LABEL)
            || environment.is_some_and(|index| usize::from(index) >= ENVIRONMENT.len())
        {
            return Err(error("byte case bounds"));
        }
        Ok(Self {
            bytes,
            protocol,
            environment,
            original_limit,
            decoded_limit,
        })
    }

    fn unlimited(bytes: Vec<u8>, protocol: Option<&str>, environment: Option<u8>) -> Result<Self> {
        Self::new(
            bytes,
            protocol.map(str::to_owned),
            environment,
            u32::MAX,
            u32::MAX,
        )
    }

    /// The exact original bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn hints(&self) -> DecodeHints<'_> {
        DecodeHints {
            protocol_label: self.protocol.as_deref(),
            environment_encoding: self.environment.map(environment),
        }
    }

    fn limits(&self) -> DecodeLimits {
        DecodeLimits::new(self.original_limit as usize, self.decoded_limit as usize)
    }
}

/// Independent expectations; tests replace one to plant a controlled failure.
#[derive(Clone, Copy)]
pub(crate) struct Oracle {
    pub(crate) selection: fn(&ByteCase) -> (String, EncodingSelection),
    pub(crate) text: fn(&ByteCase, &str) -> Option<(String, bool)>,
}

pub(crate) const INDEPENDENT: Oracle = Oracle {
    selection: expected_selection,
    text: expected_text,
};

/// Syntax 3 section 3.2: protocol label, then the exact `@charset "label";`
/// wholly within the first 1024 bytes, then the environment, then UTF-8; a byte
/// order mark overrides all of them.
fn expected_selection(case: &ByteCase) -> (String, EncodingSelection) {
    let bytes = &case.bytes;
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return ("UTF-8".into(), EncodingSelection::ByteOrderMark);
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
        return ("UTF-16LE".into(), EncodingSelection::ByteOrderMark);
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return ("UTF-16BE".into(), EncodingSelection::ByteOrderMark);
    }
    if let Some(name) = case.protocol.as_deref().and_then(standard_label) {
        return (name.into(), EncodingSelection::Protocol);
    }
    if let Some(name) = declared_name(bytes) {
        return (name, EncodingSelection::CharsetPrefix);
    }
    match case.environment {
        Some(index) => (
            standard_label(ENVIRONMENT[usize::from(index)])
                .expect("every environment label names an encoding")
                .into(),
            EncodingSelection::Environment,
        ),
        None => ("UTF-8".into(), EncodingSelection::DefaultUtf8),
    }
}

/// The Encoding Standard's label table, for the labels this package's
/// vocabularies use: label to encoding name. The crate reference reads a label
/// as an Encoding Standard label, "its case and surrounding ASCII whitespace do
/// not matter". A label outside the table (`utf-7`, an empty label, a made-up
/// name) names no encoding.
const STANDARD_LABELS: [(&str, &str); 11] = [
    ("utf-8", "UTF-8"),
    ("utf-16le", "UTF-16LE"),
    ("utf-16be", "UTF-16BE"),
    ("windows-1252", "windows-1252"),
    // `latin1` is a legacy label of windows-1252.
    ("latin1", "windows-1252"),
    ("iso-8859-2", "ISO-8859-2"),
    ("shift_jis", "Shift_JIS"),
    ("euc-kr", "EUC-KR"),
    ("iso-2022-jp", "ISO-2022-JP"),
    // A legacy label the standard maps to the replacement encoding.
    ("hz-gb-2312", "replacement"),
    ("x-user-defined", "x-user-defined"),
];

/// The encoding name of a label: strip leading and trailing ASCII whitespace,
/// lowercase ASCII, then look the label up.
fn standard_label(label: &str) -> Option<&'static str> {
    let label = label
        .trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\u{c}' | '\r'))
        .to_ascii_lowercase();
    STANDARD_LABELS
        .iter()
        .find(|(known, _)| *known == label)
        .map(|(_, name)| *name)
}

fn declared_name(bytes: &[u8]) -> Option<String> {
    let head = &bytes[..bytes.len().min(1024)];
    let rest = head.strip_prefix(b"@charset \"")?;
    let end = rest.iter().position(|byte| *byte == b'"')?;
    if rest.get(end + 1) != Some(&b';') || !rest[..end].is_ascii() {
        return None;
    }
    let name = standard_label(std::str::from_utf8(&rest[..end]).ok()?)?;
    Some(if name.starts_with("UTF-16") {
        "UTF-8".into()
    } else {
        name.into()
    })
}

/// Expected decoded text and replacement flag, for the encodings with an
/// independent standard-library decoder; `None` where there is none.
fn expected_text(case: &ByteCase, encoding: &str) -> Option<(String, bool)> {
    let bytes = case.bytes.as_slice();
    let (name, rest) = if let Some(rest) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        ("UTF-8", rest)
    } else if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        ("UTF-16LE", rest)
    } else if let Some(rest) = bytes.strip_prefix(&[0xfe, 0xff]) {
        ("UTF-16BE", rest)
    } else {
        (encoding, bytes)
    };
    match name {
        "UTF-8" => Some((
            String::from_utf8_lossy(rest).into_owned(),
            std::str::from_utf8(rest).is_err(),
        )),
        "UTF-16LE" | "UTF-16BE" => {
            let little = name == "UTF-16LE";
            let units: Vec<u16> = rest
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| {
                    if little {
                        u16::from_le_bytes(*pair)
                    } else {
                        u16::from_be_bytes(*pair)
                    }
                })
                .collect();
            let mut text = String::new();
            let mut replaced = false;
            for unit in char::decode_utf16(units.iter().copied()) {
                match unit {
                    Ok(character) => text.push(character),
                    Err(_) => {
                        text.push('\u{fffd}');
                        replaced = true;
                    }
                }
            }
            // At the end of the queue the Encoding Standard reports one error for
            // a pending lead byte and a pending lead surrogate together.
            let pending_surrogate = units
                .last()
                .is_some_and(|unit| (0xd800..0xdc00).contains(unit));
            if rest.len() % 2 == 1 && !pending_surrogate {
                text.push('\u{fffd}');
            }
            Some((text, replaced || rest.len() % 2 == 1))
        }
        // The single-byte Latin range above C1 is the identity on code points.
        "windows-1252" if rest.iter().all(|byte| !(0x80..=0x9f).contains(byte)) => {
            Some((rest.iter().map(|byte| char::from(*byte)).collect(), false))
        }
        _ => None,
    }
}

/// The target: one evaluator shared by workers, properties, coverage and replay.
pub struct ByteTarget {
    descriptor: TargetDescriptor,
    oracle: Oracle,
}

impl Default for ByteTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl ByteTarget {
    /// The number of source-qualified literal cases.
    pub const LITERALS: u8 = 20;

    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptor: descriptor("surgeist-css::byte-decoding"),
            oracle: INDEPENDENT,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_oracle(oracle: Oracle) -> Self {
        Self {
            oracle,
            ..Self::new()
        }
    }

    /// Independent literals; no expected value is sampled from the decoder.
    pub fn literal(&self, id: u8) -> Result<ByteCase> {
        literal(id)
    }

    fn observe(&self, case: &ByteCase) -> std::result::Result<Observed, Violation> {
        observe(&self.oracle, case)
    }
}

enum Observed {
    Decoded { text_checked: bool },
    Refused,
}

fn observe(oracle: &Oracle, case: &ByteCase) -> std::result::Result<Observed, Violation> {
    let reference =
        CssDecodedStylesheetInput::decode(&case.bytes, case.hints(), DecodeLimits::default())
            .map_err(|cause| {
                Violation::new(
                    LIMITS,
                    "unlimited-decode-refused",
                    format!(
                        "an unlimited decode of {} bytes failed: {cause}",
                        case.bytes.len()
                    ),
                )
            })?;

    let (name, selection) = (oracle.selection)(case);
    if reference.encoding().name() != name || reference.selection() != selection {
        return Err(Violation::new(
            SELECTION,
            "selection-differs",
            format!(
                "decoded as {} by {:?}, expected {name} by {selection:?}",
                reference.encoding().name(),
                reference.selection()
            ),
        ));
    }
    if reference.original_bytes() != case.bytes {
        return Err(Violation::new(
            TEXT,
            "original-bytes-changed",
            "the decoded input does not retain the original bytes",
        ));
    }
    let text = reference.source().as_str();
    let expected = (oracle.text)(case, &name);
    if let Some((expected_text, replaced)) = &expected {
        if text != expected_text {
            return Err(Violation::new(
                TEXT,
                "decoded-text-differs",
                format!("decoded {text:?}, expected {expected_text:?}"),
            ));
        }
        if reference.had_replacements() != *replaced {
            return Err(Violation::new(
                TEXT,
                "replacement-flag-differs",
                format!(
                    "had_replacements is {}, expected {replaced}",
                    reference.had_replacements()
                ),
            ));
        }
    }

    // Limits are atomic and independent: refuse the whole decode or equal the
    // unlimited decode.
    let limited = CssDecodedStylesheetInput::decode(&case.bytes, case.hints(), case.limits());
    if case.bytes.len() > case.original_limit as usize {
        // Neither the crate reference nor the rustdoc orders the two limit
        // errors when both are exceeded, so either refuses the decode then.
        let both = text.len() > case.decoded_limit as usize;
        match &limited {
            Err(cause)
                if cause.is_original_byte_limit()
                    || (both && cause.is_decoded_utf8_byte_limit()) =>
            {
                return Ok(Observed::Refused);
            }
            other => return Err(limit_violation("original", other)),
        }
    }
    if text.len() > case.decoded_limit as usize {
        match &limited {
            Err(cause) if cause.is_decoded_utf8_byte_limit() => return Ok(Observed::Refused),
            other => return Err(limit_violation("decoded", other)),
        }
    }
    match &limited {
        Ok(input) if input == &reference => {}
        other => return Err(limit_violation("none", other)),
    }

    parse_checks(&reference)?;
    Ok(Observed::Decoded {
        text_checked: expected.is_some(),
    })
}

fn limit_violation(
    refused: &str,
    got: &std::result::Result<CssDecodedStylesheetInput, surgeist_css::source::CssInputDecodeError>,
) -> Violation {
    let detail = match got {
        Ok(_) => format!("decode succeeded although the {refused} limit was exceeded"),
        Err(cause) => format!("decode failed ({cause}) where the {refused} limit decides"),
    };
    Violation::new(LIMITS, "limit-disposition-differs", detail)
}

/// The decoded snapshot's report obeys the coordinate and cleanliness contracts
/// and equals parsing the same decoded text.
fn parse_checks(input: &CssDecodedStylesheetInput) -> Check {
    let text = input.source().as_str();
    for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
        let parser = CssParser::new(
            CssParserContext::new(mode),
            CssComponentValueLimits::default(),
        );
        let report = parser.parse_sheet_bytes(input);
        check::diagnostics(text, report.diagnostics())?;
        check::cleanliness(&report)?;
        if report != parser.parse_sheet(text) {
            return Err(Violation::new(
                check::COORDINATES,
                "byte-front-differs-from-text-front",
                "parsing the decoded snapshot differs from parsing its text",
            ));
        }
        if input.selection() == EncodingSelection::CharsetPrefix
            && text.starts_with("@charset \"")
            && !report.diagnostics().iter().any(|diagnostic| {
                diagnostic.error().position().byte_offset().value() == 0
                    && diagnostic.error().code() == CssErrorCode::UnknownAtRule
                    && diagnostic.action() == CssRecoveryAction::DropAtRule
            })
        {
            return Err(Violation::new(
                check::CLEANLINESS,
                "charset-prefix-not-dropped",
                "the inferred charset declaration is not the dropped leading unknown at-rule",
            ));
        }
    }
    Ok(())
}

impl CaseTarget for ByteTarget {
    type Case = ByteCase;

    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }

    fn decode(&self, input: &[u8]) -> Result<ByteCase> {
        let mut reader = Reader::new(input, MAGIC, MAX_CASE)?;
        let original_limit = reader.u32()?;
        let decoded_limit = reader.u32()?;
        let environment = match reader.u8()? {
            NONE => None,
            index => Some(index),
        };
        let protocol = match reader.u8()? {
            0 => None,
            1 => Some(reader.text(MAX_LABEL)?),
            _ => return Err(error("protocol presence tag")),
        };
        let bytes = reader.bytes(MAX_BYTES)?;
        reader.finish()?;
        ByteCase::new(bytes, protocol, environment, original_limit, decoded_limit)
    }

    fn encode(&self, case: &ByteCase) -> Result<Vec<u8>> {
        let mut writer = Writer::new(MAGIC, MAX_CASE)?;
        writer.u32(case.original_limit)?;
        writer.u32(case.decoded_limit)?;
        writer.u8(case.environment.unwrap_or(NONE))?;
        match &case.protocol {
            None => writer.u8(0)?,
            Some(label) => {
                writer.u8(1)?;
                writer.text(label, MAX_LABEL)?;
            }
        }
        writer.bytes(&case.bytes, MAX_BYTES)?;
        Ok(writer.finish())
    }

    fn generate(&self, input: &[u8]) -> Result<Generation<ByteCase>> {
        generate(&self.descriptor, input).map(Generation::Generated)
    }

    fn evaluate(&self, case: &ByteCase) -> Result<CaseOutcome> {
        match self.observe(case) {
            Err(found) => violation(&found),
            Ok(Observed::Refused) => rejected(
                &[SELECTION, LIMITS],
                "decode refused at the declared byte limit",
            ),
            Ok(Observed::Decoded { text_checked }) => {
                let mut checked = vec![SELECTION, LIMITS, check::COORDINATES, check::CLEANLINESS];
                if text_checked {
                    checked.push(TEXT);
                }
                evaluated(&checked)
            }
        }
    }

    fn reduction_candidates(
        &self,
        case: &ByteCase,
        emit: &mut dyn FnMut(ByteCase) -> bool,
    ) -> Result<()> {
        // Candidates keep the hints and limits; they only shorten the payload or
        // drop a hint, and each is revalidated by the checked constructor.
        let len = case.bytes.len();
        let mut candidates: Vec<ByteCase> = Vec::new();
        for size in [0, len / 2, len.saturating_sub(1)] {
            if size < len {
                let mut shorter = case.clone();
                shorter.bytes.truncate(size);
                candidates.push(shorter);
            }
        }
        if len > 1 {
            let mut tail = case.clone();
            tail.bytes.drain(..len / 2);
            candidates.push(tail);
        }
        if case.protocol.is_some() {
            candidates.push(ByteCase {
                protocol: None,
                ..case.clone()
            });
        }
        if case.environment.is_some() {
            candidates.push(ByteCase {
                environment: None,
                ..case.clone()
            });
        }
        for candidate in candidates {
            if !emit(candidate) {
                break;
            }
        }
        Ok(())
    }
}

/// Entropy to a case. The leading bytes choose a prefix shape (none, each BOM,
/// a declared charset of every table label, near-miss declarations), the hints
/// and limits; the rest builds a payload of CSS snippets and raw byte runs.
fn generate(descriptor: &TargetDescriptor, input: &[u8]) -> Result<ByteCase> {
    let mut cursor = Cursor::new(input);
    let mut bytes: Vec<u8> = Vec::new();
    match cursor.below(14) {
        0 => bytes.extend_from_slice(&[0xef, 0xbb, 0xbf]),
        1 => bytes.extend_from_slice(&[0xff, 0xfe]),
        2 => bytes.extend_from_slice(&[0xfe, 0xff]),
        3..=7 => {
            let label = DECLARED[cursor.below(DECLARED.len())];
            bytes.extend_from_slice(format!("@charset \"{label}\";").as_bytes());
        }
        8 => bytes.extend_from_slice(b"@charset 'utf-8';"),
        9 => bytes.extend_from_slice(b"@charset  \"utf-8\";"),
        10 => bytes.extend_from_slice(b"@CHARSET \"utf-8\";"),
        11 => {
            // A declaration whose closing straddles the 1024-byte window.
            let padding = 1005 + cursor.below(4);
            let mut label = String::from("utf-8");
            label.push_str(&" ".repeat(padding));
            bytes.extend_from_slice(format!("@charset \"{label}\";").as_bytes());
        }
        _ => {}
    }
    for _ in 0..cursor.below(24) {
        match cursor.below(10) {
            0..=3 => bytes.extend_from_slice(SNIPPETS[cursor.below(SNIPPETS.len())].as_bytes()),
            4..=6 => bytes.extend_from_slice(RAW[cursor.below(RAW.len())]),
            7 => {
                // UTF-16 text in either order, possibly with a lone surrogate.
                let little = cursor.flag();
                let units: [u16; 3] = [
                    0x61 + u16::from(cursor.byte() % 26),
                    [0x7b, 0x20ac, 0xd800, 0xdc00, 0xd83d][cursor.below(5)],
                    0x7d,
                ];
                for unit in units {
                    bytes.extend_from_slice(&if little {
                        unit.to_le_bytes()
                    } else {
                        unit.to_be_bytes()
                    });
                }
            }
            8 => bytes.push(cursor.byte()),
            _ => bytes.extend_from_slice("\u{e9}\u{20ac}\u{1f600}\u{feff}".as_bytes()),
        }
        if bytes.len() > MAX_BYTES / 2 {
            break;
        }
    }
    bytes.truncate(MAX_BYTES);
    let protocol = match cursor.below(4) {
        0 => Some(PROTOCOL[cursor.below(PROTOCOL.len())].to_owned()),
        _ => None,
    };
    let environment = match cursor.below(3) {
        0 => Some(u8::try_from(cursor.below(ENVIRONMENT.len())).unwrap_or(0)),
        _ => None,
    };
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    let original_limit = match cursor.below(6) {
        0 => len,
        1 => len.saturating_sub(1),
        _ => u32::MAX,
    };
    let decoded_limit = match cursor.below(6) {
        0 => len,
        1 => len / 2,
        2 => 0,
        _ => u32::MAX,
    };
    let mut budget = Budget::new(descriptor.limits());
    budget.credit(BudgetCharge {
        bytes: bytes.len() as u64 + 64,
        text_units: bytes.len() as u64,
        ..BudgetCharge::default()
    })?;
    ByteCase::new(bytes, protocol, environment, original_limit, decoded_limit)
}

/// Source-qualified literal cases. Each states its own expectation in the
/// evaluator's independent oracle; none is sampled from decoder output.
fn literal(id: u8) -> Result<ByteCase> {
    let utf16 = |little: bool, text: &str| -> Vec<u8> {
        text.encode_utf16()
            .flat_map(|unit| {
                if little {
                    unit.to_le_bytes()
                } else {
                    unit.to_be_bytes()
                }
            })
            .collect()
    };
    let with = |prefix: &[u8], rest: &[u8]| [prefix, rest].concat();
    let unlimited = ByteCase::unlimited;
    match id {
        0 => unlimited(b".a{color:red}".to_vec(), None, None),
        1 => unlimited(with(&[0xef, 0xbb, 0xbf], b".a{color:red}"), None, None),
        2 => unlimited(with(&[0xff, 0xfe], &utf16(true, ".a{b:c}")), None, None),
        3 => unlimited(with(&[0xfe, 0xff], &utf16(false, ".a{b:c}")), None, None),
        4 => unlimited(
            b"@charset \"windows-1252\"; a{content:\"\xe9\xa0\xff\"}".to_vec(),
            None,
            None,
        ),
        5 => unlimited(
            b"@charset \"windows-1252\"; a{content:\"\xe9\"}".to_vec(),
            Some("utf-8"),
            None,
        ),
        6 => unlimited(b"a{content:\"\xe9\"}".to_vec(), None, Some(3)),
        7 => unlimited(b"@charset \"utf-16le\"; a{b:c}".to_vec(), None, Some(3)),
        8 => unlimited(
            b"a{content:\"\xc0\x80\xed\xa0\x80\xf4\x90\x80\x80\xe2\x82\"}".to_vec(),
            None,
            None,
        ),
        9 => unlimited(
            b"@charset  \"windows-1252\"; a{b:\xe9}".to_vec(),
            None,
            Some(3),
        ),
        10 | 11 => {
            // The declaration ends at byte 1024 (recognized) or 1025 (not).
            let spaces = " ".repeat(if id == 10 { 1007 } else { 1008 });
            let head = format!("@charset \"utf-8{spaces}\";");
            unlimited([head.as_bytes(), b"a{b:\xe9}"].concat(), None, Some(3))
        }
        12 => {
            let bytes = b".a{b:c}".to_vec();
            let len = u32::try_from(bytes.len()).map_err(|_| error("literal length"))?;
            ByteCase::new(bytes, None, None, len, u32::MAX)
        }
        13 => {
            let bytes = b".a{b:c}".to_vec();
            let len = u32::try_from(bytes.len()).map_err(|_| error("literal length"))?;
            ByteCase::new(bytes, None, None, len - 1, u32::MAX)
        }
        14 => {
            // Six original bytes decode to six UTF-8 bytes; five is refused.
            let bytes = with(&[0xff, 0xfe], &utf16(true, "\u{20ac}\u{20ac}"));
            ByteCase::new(bytes, None, None, u32::MAX, 5)
        }
        15 => unlimited(b"a{b:c}".to_vec(), Some("hz-gb-2312"), None),
        16 => unlimited(
            with(&[0xff, 0xfe], &[0x61, 0x00, 0x00, 0xd8, 0x62]),
            None,
            None,
        ),
        17 => unlimited(
            b"a{}\r\n.b{\x0c}\r.c{x:\x00}\n\xe2\x82\xac{}".to_vec(),
            None,
            None,
        ),
        18 => unlimited(b"/* open comment \\".to_vec(), None, None),
        19 => unlimited(b"a{b:c}".to_vec(), Some(" UTF-16BE\n"), None),
        _ => Err(error("unknown literal")),
    }
}

//! Revision 1: fixed header followed by exact already-decoded Unicode input.
use crate::{UnicodeTarget, error};
use surgeist_fuzz::{
    Result,
    generation::{Budget, BudgetCharge},
};
use surgeist_html_parser::HtmlParseLimits;

pub const MAX_SOURCE_UNITS: usize = 4096;
pub const CAPS: [u64; 19] = [
    100_000,
    100_000,
    10_000,
    1_000_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100,
    100_000,
    100_000_000,
    100_000_000,
    100_000_000,
    100_000_000,
];
const HEADER: usize = 4 + 4 + 19 * 8 + 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseKind {
    Document,
    Fragment,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lane {
    General,
    PreCancelled,
    SourceDenied,
    WorkDenied,
    Literal(u8),
    MissingScripting,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnicodeSource {
    Utf8(String),
    Utf16(Vec<u16>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnicodeCase {
    pub(crate) kind: ParseKind,
    pub(crate) source: UnicodeSource,
    pub(crate) lane: Lane,
    pub(crate) caps: [u64; 19],
}
impl UnicodeCase {
    pub fn new(kind: ParseKind, source: UnicodeSource, lane: Lane) -> Result<Self> {
        let mut caps = CAPS;
        match lane {
            Lane::SourceDenied => caps[0] = 0,
            Lane::WorkDenied => caps[18] = 0,
            _ => {}
        }
        let case = Self {
            kind,
            source,
            lane,
            caps,
        };
        case.validate()?;
        Ok(case)
    }
    /// Checked reconfiguration; no unlimited cap or implicit default.
    pub fn with_caps(mut self, caps: [u64; 19]) -> Result<Self> {
        self.caps = caps;
        self.validate()?;
        Ok(self)
    }
    pub fn source(&self) -> &UnicodeSource {
        &self.source
    }
    pub fn lane(&self) -> Lane {
        self.lane
    }
    pub fn kind(&self) -> ParseKind {
        self.kind
    }
    pub fn limits(&self) -> Result<HtmlParseLimits> {
        limits(self.caps)
    }
    fn validate(&self) -> Result<()> {
        let units = match &self.source {
            UnicodeSource::Utf8(s) => s.encode_utf16().count(),
            UnicodeSource::Utf16(s) => s.len(),
        };
        if units > MAX_SOURCE_UNITS || self.caps.iter().zip(CAPS).any(|(a, b)| *a > b) {
            return Err(error("source/cap envelope"));
        }
        self.limits()?;
        match self.lane {
            Lane::Literal(id) if !crate::seeds::matches(id, &self.source) || self.caps != CAPS => {
                return Err(error("literal oracle input mismatch"));
            }
            Lane::MissingScripting if self.kind != ParseKind::Fragment => {
                return Err(error("context lane requires fragment"));
            }
            Lane::SourceDenied
                if self.caps[0] != 0 || units == 0 || self.caps[1..] != CAPS[1..] =>
            {
                return Err(error(
                    "source denial requires nonempty source and zero source cap",
                ));
            }
            Lane::WorkDenied if self.caps[18] != 0 || self.caps[..18] != CAPS[..18] => {
                return Err(error("work denial requires zero work cap"));
            }
            _ => {}
        }
        Ok(())
    }
}
pub(crate) fn limits(c: [u64; 19]) -> Result<HtmlParseLimits> {
    HtmlParseLimits::builder()
        .source_units(c[0])
        .buffered_utf16_units(c[1])
        .token_attributes(c[2])
        .auxiliary_entries(c[3])
        .managed_nodes(c[4])
        .managed_edges(c[5])
        .managed_attributes(c[6])
        .managed_utf16_units(c[7])
        .source_map_entries(c[8])
        .source_map_payload_units(c[9])
        .diagnostics(c[10])
        .diagnostic_payload_units(c[11])
        .context_effects(c[12])
        .open_elements(c[13])
        .tree_depth(c[14])
        .managed_capacity_bytes(c[15])
        .peak_transient_bytes(c[16])
        .atomic_layout_bytes(c[17])
        .work_credits(c[18])
        .finish()
        .map_err(|_| error("invalid parser cap"))
}
impl UnicodeTarget {
    pub(crate) fn decode_case(&self, input: &[u8]) -> Result<UnicodeCase> {
        if input.len() < HEADER
            || input.len() as u64 > self.descriptor.limits().bytes()
            || &input[..4] != b"HUF1"
        {
            return Err(error("header/byte admission"));
        }
        let kind = match input[4] {
            0 => ParseKind::Document,
            1 => ParseKind::Fragment,
            _ => return Err(error("target tag")),
        };
        if kind != self.kind {
            return Err(error("wrong target"));
        }
        let lane = match (input[6], input[7]) {
            (0, 0) => Lane::General,
            (1, 0) => Lane::PreCancelled,
            (2, 0) => Lane::SourceDenied,
            (3, 0) => Lane::WorkDenied,
            (4, id) if id < 8 => Lane::Literal(id),
            (5, 0) => Lane::MissingScripting,
            _ => return Err(error("lane/reserved byte")),
        };
        let mut caps = [0; 19];
        for (i, cap) in caps.iter_mut().enumerate() {
            *cap = u64::from_le_bytes(
                input[8 + i * 8..16 + i * 8]
                    .try_into()
                    .expect("fixed header"),
            );
        }
        if caps.iter().zip(CAPS).any(|(a, b)| *a > b) {
            return Err(error("cap envelope"));
        }
        limits(caps)?;
        let count = u32::from_le_bytes(input[HEADER - 4..HEADER].try_into().expect("fixed header"))
            as usize;
        let payload = &input[HEADER..];
        let (units, storage) = match input[5] {
            0 if count == payload.len() => {
                let s = std::str::from_utf8(payload)
                    .map_err(|_| error("invalid UTF8 convenience source"))?;
                (s.encode_utf16().count(), payload.len())
            }
            1 if count <= MAX_SOURCE_UNITS && count.checked_mul(2) == Some(payload.len()) => {
                (count, payload.len())
            }
            _ => return Err(error("source tag/length/trailing bytes")),
        };
        if units > MAX_SOURCE_UNITS {
            return Err(error("source unit admission"));
        }
        match lane {
            Lane::Literal(id)
                if caps != CAPS || !crate::seeds::payload_matches(id, input[5], payload) =>
            {
                return Err(error("literal oracle input mismatch"));
            }
            Lane::MissingScripting if kind != ParseKind::Fragment => {
                return Err(error("context lane requires fragment"));
            }
            Lane::SourceDenied if caps[0] != 0 || units == 0 || caps[1..] != CAPS[1..] => {
                return Err(error("source denial qualification"));
            }
            Lane::WorkDenied if caps[18] != 0 || caps[..18] != CAPS[..18] => {
                return Err(error("work denial qualification"));
            }
            _ => {}
        }
        // Aggregate credit BEFORE materializing any source/capture payload. The
        // div recipe has two subjects and fixed names/namespace (<=128 units).
        let context = u64::from(kind == ParseKind::Fragment);
        Budget::new(self.descriptor.limits()).credit(BudgetCharge {
            bytes: (HEADER + storage + 256 * context as usize) as u64,
            nodes: 2 * context,
            depth: context,
            text_units: units as u64 + 128 * context,
            operations: 3,
            ..BudgetCharge::default()
        })?;
        let source = if input[5] == 0 {
            UnicodeSource::Utf8(
                std::str::from_utf8(payload)
                    .expect("validated UTF8")
                    .to_owned(),
            )
        } else {
            UnicodeSource::Utf16(
                payload
                    .chunks_exact(2)
                    .map(|p| u16::from_le_bytes([p[0], p[1]]))
                    .collect(),
            )
        };
        let case = UnicodeCase {
            kind,
            source,
            lane,
            caps,
        };
        case.validate()?;
        Ok(case)
    }
    pub(crate) fn encode_case(&self, case: &UnicodeCase) -> Result<Vec<u8>> {
        case.validate()?;
        if case.kind != self.kind {
            return Err(error("wrong target"));
        }
        let (encoding, count) = match &case.source {
            UnicodeSource::Utf8(s) => (0, s.len()),
            UnicodeSource::Utf16(s) => (1, s.len()),
        };
        let size = HEADER + count * (1 + encoding as usize);
        if size as u64 > self.descriptor.limits().bytes() {
            return Err(error("encoded byte limit"));
        }
        let mut out = Vec::with_capacity(size);
        out.extend_from_slice(b"HUF1");
        out.push(u8::from(case.kind == ParseKind::Fragment));
        out.push(encoding);
        let (lane, id) = match case.lane {
            Lane::General => (0, 0),
            Lane::PreCancelled => (1, 0),
            Lane::SourceDenied => (2, 0),
            Lane::WorkDenied => (3, 0),
            Lane::Literal(id) => (4, id),
            Lane::MissingScripting => (5, 0),
        };
        out.extend_from_slice(&[lane, id]);
        for cap in case.caps {
            out.extend_from_slice(&cap.to_le_bytes());
        }
        out.extend_from_slice(&(count as u32).to_le_bytes());
        match &case.source {
            UnicodeSource::Utf8(s) => out.extend_from_slice(s.as_bytes()),
            UnicodeSource::Utf16(s) => {
                for u in s {
                    out.extend_from_slice(&u.to_le_bytes());
                }
            }
        }
        Ok(out)
    }
}

//! HTML-owned concrete cases over public complete parse front doors.
#![forbid(unsafe_code)]
mod codec;
mod observe;
mod seeds;
pub use codec::{Lane, ParseKind, UnicodeCase, UnicodeSource};
use surgeist_fuzz::{registry::FuzzManifest, *};

pub(crate) fn error(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::InvalidCase, "HTML Unicode case", detail)
}
pub struct UnicodeTarget {
    pub(crate) descriptor: TargetDescriptor,
    pub(crate) kind: ParseKind,
}
impl UnicodeTarget {
    pub fn document() -> Self {
        Self::new(ParseKind::Document)
    }
    pub fn fragment() -> Self {
        Self::new(ParseKind::Fragment)
    }
    fn new(kind: ParseKind) -> Self {
        let manifest: FuzzManifest =
            toml::from_str(include_str!("../fuzz.toml")).expect("checked owner descriptor");
        let id = match kind {
            ParseKind::Document => "surgeist-html::unicode-document",
            ParseKind::Fragment => "surgeist-html::unicode-fragment",
        };
        Self {
            descriptor: manifest
                .targets
                .into_iter()
                .find(|d| d.id().as_str() == id)
                .expect("registered target"),
            kind,
        }
    }
    pub fn literal(&self, id: u8) -> Result<UnicodeCase> {
        UnicodeCase::new(self.kind, seeds::source(id)?, Lane::Literal(id))
    }
}
impl CaseTarget for UnicodeTarget {
    type Case = UnicodeCase;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<UnicodeCase> {
        self.decode_case(input)
    }
    fn encode(&self, case: &UnicodeCase) -> Result<Vec<u8>> {
        self.encode_case(case)
    }
    fn evaluate(&self, case: &UnicodeCase) -> Result<CaseOutcome> {
        // Constructors are private/checked; target qualification also holds for
        // a caller moving a valid case between the two concrete targets.
        if case.kind != self.kind {
            return Err(error("wrong evaluator target"));
        }
        observe::evaluate(self, case)
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<UnicodeCase>> {
        if input.len() as u64 > self.descriptor.limits().bytes() {
            return Err(error("entropy byte admission"));
        }
        // One quarter is independently specified literal successes. Remaining
        // entropy preserves all u16 values; odd trailing byte has high byte 0.
        let choice = input.first().copied().unwrap_or(0);
        if choice % 4 == 0 {
            return self.literal((choice / 4) % 9).map(Generation::Generated);
        }
        let raw = input.get(1..).unwrap_or_default();
        let units = if choice % 4 == 1 {
            raw.len().div_ceil(2)
        } else {
            raw.len()
        };
        if units > codec::MAX_SOURCE_UNITS {
            return Ok(Generation::Rejected(GenerationRejection::new(
                "entropy source exceeds selected ceiling",
            )?));
        }
        let context = u64::from(self.kind == ParseKind::Fragment);
        generation::Budget::new(self.descriptor.limits()).credit(generation::BudgetCharge {
            bytes: raw.len() as u64 * 2 + 512,
            nodes: 2 * context,
            depth: context,
            text_units: units as u64 + 128 * context,
            operations: 3,
            ..generation::BudgetCharge::default()
        })?;
        let source = if choice % 4 == 1 {
            UnicodeSource::Utf16(
                raw.chunks(2)
                    .map(|p| u16::from_le_bytes([p[0], p.get(1).copied().unwrap_or(0)]))
                    .collect(),
            )
        } else {
            UnicodeSource::Utf8(raw.iter().map(|b| char::from(*b)).collect())
        };
        UnicodeCase::new(self.kind, source, Lane::General).map(Generation::Generated)
    }
    fn reduction_candidates(
        &self,
        case: &UnicodeCase,
        emit: &mut dyn FnMut(UnicodeCase) -> bool,
    ) -> Result<()> {
        // Literal oracles require their exact original input. Other lanes keep
        // caps/context/operations; the framework checks the same failure key.
        if matches!(case.lane, Lane::Literal(_)) {
            return Ok(());
        }
        let len = match &case.source {
            UnicodeSource::Utf8(s) => s.len(),
            UnicodeSource::Utf16(s) => s.len(),
        };
        for size in [0, len / 2, len.saturating_sub(1)] {
            if size >= len {
                continue;
            }
            let source = match &case.source {
                UnicodeSource::Utf8(s) => {
                    let mut boundary = size;
                    while !s.is_char_boundary(boundary) {
                        boundary -= 1;
                    }
                    UnicodeSource::Utf8(s[..boundary].into())
                }
                UnicodeSource::Utf16(s) => UnicodeSource::Utf16(s[..size].to_vec()),
            };
            if let Ok(candidate) =
                UnicodeCase::new(case.kind, source, case.lane).and_then(|c| c.with_caps(case.caps))
                && !emit(candidate)
            {
                break;
            }
        }
        Ok(())
    }
}

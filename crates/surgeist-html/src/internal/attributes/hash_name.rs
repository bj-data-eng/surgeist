//! Exact local hash-name observations, HTML §2.3.9. Tree proof is separate.

use super::*;
use crate::HtmlSpaceToken;

const AUTHOR_SOURCE: &str = "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-hash-name-reference";
const PROCESSING_SOURCE: &str = "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#rules-for-parsing-a-hash-name-reference";

/// Local form only; LeadingHashName does not establish a same-tree name witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHashNameAuthorForm {
    /// The operand is absent, so there is no reference to classify.
    Missing,
    /// The value does not start with U+0023 (`#`), so it is not a valid
    /// hash-name reference whatever the tree contains.
    NoLeadingHash,
    /// Includes the empty name following a lone #.
    LeadingHashName,
}

/// Local processing observation before any specified-type tree lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHashNameProcessing {
    /// The operand is absent, so the rules for parsing a hash-name reference
    /// have nothing to parse.
    Missing,
    /// The value contains no U+0023 (`#`), so parsing returns null.
    NoHash,
    /// The first `#` is the last code unit, so parsing returns null.
    EmptyTail,
    /// A nonempty suffix follows the first `#`. It is the name that the tree
    /// lookup of the parsing rules would search for.
    Name,
}

/// Exact shared raw evidence, with borrowed suffixes and no tree conformance claim.
///
/// The author form and the processing observation are functions of the raw value and the position
/// of its first `#`, so the report stores only those and derives both; no stored classification
/// can disagree with the evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlHashNameReferenceReport {
    operand: HashNameOperand,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum HashNameOperand {
    Absent,
    Present {
        raw: DocumentString,
        first_hash: Option<usize>,
    },
}
impl HtmlHashNameReferenceReport {
    /// The exact raw value, retained by shared handle. `None` is an absent
    /// operand; an empty value is `Some`.
    pub fn raw(&self) -> Option<&DocumentString> {
        match &self.operand {
            HashNameOperand::Absent => None,
            HashNameOperand::Present { raw, .. } => Some(raw),
        }
    }
    /// Offset in the original UTF-16 code units.
    pub fn first_hash_offset(&self) -> Option<usize> {
        match &self.operand {
            HashNameOperand::Absent => None,
            HashNameOperand::Present { first_hash, .. } => *first_hash,
        }
    }
    /// Whether the value has the leading-`#` shape of a valid hash-name
    /// reference. The name after it is not checked against any tree.
    pub fn author_form(&self) -> HtmlHashNameAuthorForm {
        match &self.operand {
            HashNameOperand::Absent => HtmlHashNameAuthorForm::Missing,
            HashNameOperand::Present {
                first_hash: Some(0),
                ..
            } => HtmlHashNameAuthorForm::LeadingHashName,
            HashNameOperand::Present { .. } => HtmlHashNameAuthorForm::NoLeadingHash,
        }
    }
    /// How the rules for parsing a hash-name reference classify the value,
    /// before any lookup in a tree.
    pub fn processing(&self) -> HtmlHashNameProcessing {
        match &self.operand {
            HashNameOperand::Absent => HtmlHashNameProcessing::Missing,
            HashNameOperand::Present {
                first_hash: None, ..
            } => HtmlHashNameProcessing::NoHash,
            HashNameOperand::Present {
                raw,
                first_hash: Some(offset),
            } if offset + 1 == raw.len() => HtmlHashNameProcessing::EmptyTail,
            HashNameOperand::Present { .. } => HtmlHashNameProcessing::Name,
        }
    }
    /// Some(empty) is a leading # with an empty author-definition name.
    pub fn author_name_units(&self) -> Option<&[u16]> {
        match &self.operand {
            HashNameOperand::Present {
                raw,
                first_hash: Some(0),
            } => Some(&raw.code_units()[1..]),
            _ => None,
        }
    }
    /// The UTF-16 code units after the first `#`, exactly as the parsing rules
    /// take them. `Some` only for `HtmlHashNameProcessing::Name`.
    pub fn processing_name_units(&self) -> Option<&[u16]> {
        match &self.operand {
            HashNameOperand::Present {
                raw,
                first_hash: Some(offset),
            } if offset + 1 < raw.len() => Some(&raw.code_units()[offset + 1..]),
            _ => None,
        }
    }
    /// The clause that defines a valid hash-name reference (HTML877d #syntax-references).
    pub fn author_source_clause(&self) -> &'static str {
        AUTHOR_SOURCE
    }
    /// The clause that defines the rules for parsing a hash-name reference
    /// (HTML877d #syntax-references).
    pub fn processing_source_clause(&self) -> &'static str {
        PROCESSING_SOURCE
    }
    /// The local observation for an absent operand, without tree evidence.
    pub fn missing() -> Self {
        Self {
            operand: HashNameOperand::Absent,
        }
    }
}

/// Symbolic usemap spelling for a checked nonempty map-name token.
///
/// This does not prove existence, uniqueness, same-tree membership, map id/name
/// equality or serialized-character conformance. It is not a resolved map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlUsemapReference {
    name: HtmlSpaceToken,
}
impl HtmlUsemapReference {
    /// Wraps a map name that is already a nonempty token without whitespace.
    /// The name is kept exactly; the `#` is added only when emitted.
    pub fn for_map_name(name: HtmlSpaceToken) -> Self {
        Self { name }
    }
    /// The exact map name, without the leading `#`.
    pub fn map_name(&self) -> &HtmlSpaceToken {
        &self.name
    }
    /// Emits literal # followed by exact token units, with ordinary allocation.
    /// No normalization, URL parsing or tree validation is performed.
    pub fn into_attribute(self) -> Attribute {
        let units: Vec<u16> = std::iter::once(u16::from(b'#'))
            .chain(self.name.value().code_units().iter().copied())
            .collect();
        Attribute::new(
            KnownHtmlAttribute::Usemap.qualified_name(),
            DocumentString::from_utf16(units),
        )
    }
}

impl HtmlValueInterpreter {
    /// Classifies the first hash and borrows its exact suffix, HTML §2.3.9.
    ///
    /// Full original raw UTF-16 length is admitted first. Local author form and
    /// processing guards remain separate; no tree witness or overall author
    /// conformance is established. No trimming, decoding or case folding occurs.
    /// Shared-handle retention uses ordinary allocation; the per-value cap is
    /// not a physical-memory, semantic work or cancellation budget.
    pub fn hash_name_reference(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlHashNameReferenceReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw)?;
        let Some(raw) = raw else {
            return Ok(HtmlHashNameReferenceReport::missing());
        };
        let mut scan = HtmlHashNameScan::new(raw);
        loop {
            match scan.step() {
                HtmlHashNameScanStep::Continue(next) => scan = next,
                HtmlHashNameScanStep::Complete(report) => return Ok(report),
            }
        }
    }
}

/// One consumed state per action. Ready completion is separate from the last
/// unit read, so the caller can admit work before exact raw-handle retention.
///
/// The scan is a consuming cursor: it is not `Clone`, so one state cannot be
/// stepped twice. It borrows the raw value for `'raw`.
#[derive(Debug)]
pub struct HtmlHashNameScan<'raw> {
    raw: &'raw DocumentString,
    len: usize,
    phase: HashNameScanPhase,
}
#[derive(Debug)]
enum HashNameScanPhase {
    Searching(usize),
    Ready(Option<usize>),
}
/// The result of one `HtmlHashNameScan::step`.
#[derive(Debug)]
pub enum HtmlHashNameScanStep<'raw> {
    /// The scan has more steps. Step this state to continue.
    Continue(HtmlHashNameScan<'raw>),
    /// The scan is finished. The report holds the shared raw handle and the
    /// local classification.
    Complete(HtmlHashNameReferenceReport),
}
impl<'raw> HtmlHashNameScan<'raw> {
    // Action-accounting contract, not a CPU-instruction count. Begin observes
    // length. Search charges bounds/read/compare/advance; Ready charges author
    // classification/processing classification/raw retention/report write.
    /// Work units a driver charges once, when the scan begins.
    pub const BEGIN_WORK: usize = 1;
    /// Work units a driver charges for each call to `step`.
    pub const STEP_WORK: usize = 4;
    /// Begins a scan of a borrowed raw value. An empty value is ready to
    /// complete immediately; otherwise the scan starts at the first code unit.
    pub fn new(raw: &'raw DocumentString) -> Self {
        let len = raw.len();
        Self {
            raw,
            len,
            phase: if len == 0 {
                HashNameScanPhase::Ready(None)
            } else {
                HashNameScanPhase::Searching(0)
            },
        }
    }
    /// Consumes this state and performs one action. A searching step reads one
    /// code unit; the following step completes the report after the first `#`
    /// or the last unit.
    pub fn step(mut self) -> HtmlHashNameScanStep<'raw> {
        match self.phase {
            HashNameScanPhase::Searching(offset) => {
                // Searching guarantees offset < len. The next offset cannot
                // overflow: it is bounded by the existing slice length.
                let unit = self.raw.code_units()[offset];
                let next = offset + 1;
                self.phase = if unit == u16::from(b'#') {
                    HashNameScanPhase::Ready(Some(offset))
                } else if next == self.len {
                    HashNameScanPhase::Ready(None)
                } else {
                    HashNameScanPhase::Searching(next)
                };
                HtmlHashNameScanStep::Continue(self)
            }
            HashNameScanPhase::Ready(first_hash) => {
                HtmlHashNameScanStep::Complete(HtmlHashNameReferenceReport {
                    operand: HashNameOperand::Present {
                        raw: self.raw.clone(),
                        first_hash,
                    },
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumed_scan_separates_the_final_unit_from_ready_retention() {
        let raw = DocumentString::from("ab#c");
        let mut scan = HtmlHashNameScan::new(&raw);
        assert!(matches!(scan.phase, HashNameScanPhase::Searching(0)));
        for offset in [1, 2] {
            scan = match scan.step() {
                HtmlHashNameScanStep::Continue(next) => next,
                HtmlHashNameScanStep::Complete(_) => panic!("premature completion"),
            };
            assert!(matches!(scan.phase, HashNameScanPhase::Searching(n) if n == offset));
        }
        scan = match scan.step() {
            HtmlHashNameScanStep::Continue(next) => next,
            HtmlHashNameScanStep::Complete(_) => panic!("retained during unit read"),
        };
        assert!(matches!(scan.phase, HashNameScanPhase::Ready(Some(2))));
        let HtmlHashNameScanStep::Complete(report) = scan.step() else {
            panic!("Ready must complete")
        };
        assert_eq!(report.first_hash_offset(), Some(2));
        assert_eq!(report.processing_name_units(), Some(&[99][..]));
    }

    #[test]
    fn literal_scanner_action_schedules_stop_at_the_first_hash() {
        for (raw, steps, work) in [
            ("", 1, 5),
            ("abcd", 5, 21),
            ("ab#c", 4, 17),
            ("#", 2, 9),
            ("#long-tail", 2, 9),
        ] {
            let raw = DocumentString::from(raw);
            let mut scan = HtmlHashNameScan::new(&raw);
            let mut observed_steps = 0;
            loop {
                observed_steps += 1;
                match scan.step() {
                    HtmlHashNameScanStep::Continue(next) => scan = next,
                    HtmlHashNameScanStep::Complete(_) => break,
                }
            }
            assert_eq!(observed_steps, steps);
            assert_eq!(
                HtmlHashNameScan::BEGIN_WORK + observed_steps * HtmlHashNameScan::STEP_WORK,
                work
            );
        }
    }
}

//! Exact local hash-name observations, HTML §2.3.9. Tree proof is separate.

use super::*;
use crate::HtmlSpaceToken;

const AUTHOR_SOURCE: &str = "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-hash-name-reference";
const PROCESSING_SOURCE: &str = "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#rules-for-parsing-a-hash-name-reference";

/// Local form only; LeadingHashName does not establish a same-tree name witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHashNameAuthorForm {
    Missing,
    NoLeadingHash,
    /// Includes the empty name following a lone #.
    LeadingHashName,
}

/// Local processing observation before any specified-type tree lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlHashNameProcessing {
    Missing,
    NoHash,
    EmptyTail,
    Name,
}

/// Exact shared raw evidence, with borrowed suffixes and no tree conformance claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlHashNameReferenceReport {
    raw: Option<DocumentString>,
    first_hash: Option<usize>,
    author_form: HtmlHashNameAuthorForm,
    processing: HtmlHashNameProcessing,
}
impl HtmlHashNameReferenceReport {
    pub fn raw(&self) -> Option<&DocumentString> {
        self.raw.as_ref()
    }
    /// Offset in the original UTF-16 code units.
    pub fn first_hash_offset(&self) -> Option<usize> {
        self.first_hash
    }
    pub fn author_form(&self) -> HtmlHashNameAuthorForm {
        self.author_form
    }
    pub fn processing(&self) -> HtmlHashNameProcessing {
        self.processing
    }
    /// Some(empty) is a leading # with an empty author-definition name.
    pub fn author_name_units(&self) -> Option<&[u16]> {
        (self.author_form == HtmlHashNameAuthorForm::LeadingHashName).then(|| {
            &self
                .raw
                .as_ref()
                .expect("present leading hash")
                .code_units()[1..]
        })
    }
    pub fn processing_name_units(&self) -> Option<&[u16]> {
        (self.processing == HtmlHashNameProcessing::Name).then(|| {
            &self.raw.as_ref().expect("present hash").code_units()
                [self.first_hash.expect("recognized hash") + 1..]
        })
    }
    pub fn author_source_clause(&self) -> &'static str {
        AUTHOR_SOURCE
    }
    pub fn processing_source_clause(&self) -> &'static str {
        PROCESSING_SOURCE
    }
    /// The local observation for an absent operand, without tree evidence.
    pub fn missing() -> Self {
        Self {
            raw: None,
            first_hash: None,
            author_form: HtmlHashNameAuthorForm::Missing,
            processing: HtmlHashNameProcessing::Missing,
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
    pub fn for_map_name(name: HtmlSpaceToken) -> Self {
        Self { name }
    }
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
        let mut scan = HashNameScan::new(raw);
        loop {
            match scan.step() {
                HashNameScanStep::Continue(next) => scan = next,
                HashNameScanStep::Complete(report) => return Ok(report),
            }
        }
    }
}

/// One consumed state per action. Ready completion is separate from the last
/// unit read, so the caller can admit work before exact raw-handle retention.
pub struct HashNameScan<'raw> {
    raw: &'raw DocumentString,
    len: usize,
    phase: HashNameScanPhase,
}
enum HashNameScanPhase {
    Searching(usize),
    Ready(Option<usize>),
}
pub enum HashNameScanStep<'raw> {
    Continue(HashNameScan<'raw>),
    Complete(HtmlHashNameReferenceReport),
}
impl<'raw> HashNameScan<'raw> {
    // Action-accounting contract, not a CPU-instruction count. Begin observes
    // length. Search charges bounds/read/compare/advance; Ready charges author
    // classification/processing classification/raw retention/report write.
    pub const BEGIN_WORK: usize = 1;
    pub const STEP_WORK: usize = 4;
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
    pub fn step(mut self) -> HashNameScanStep<'raw> {
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
                HashNameScanStep::Continue(self)
            }
            HashNameScanPhase::Ready(first_hash) => {
                let author_form = if first_hash == Some(0) {
                    HtmlHashNameAuthorForm::LeadingHashName
                } else {
                    HtmlHashNameAuthorForm::NoLeadingHash
                };
                let processing = match first_hash {
                    None => HtmlHashNameProcessing::NoHash,
                    Some(offset) if offset + 1 == self.len => HtmlHashNameProcessing::EmptyTail,
                    Some(_) => HtmlHashNameProcessing::Name,
                };
                HashNameScanStep::Complete(HtmlHashNameReferenceReport {
                    raw: Some(self.raw.clone()),
                    first_hash,
                    author_form,
                    processing,
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
        let mut scan = HashNameScan::new(&raw);
        assert!(matches!(scan.phase, HashNameScanPhase::Searching(0)));
        for offset in [1, 2] {
            scan = match scan.step() {
                HashNameScanStep::Continue(next) => next,
                HashNameScanStep::Complete(_) => panic!("premature completion"),
            };
            assert!(matches!(scan.phase, HashNameScanPhase::Searching(n) if n == offset));
        }
        scan = match scan.step() {
            HashNameScanStep::Continue(next) => next,
            HashNameScanStep::Complete(_) => panic!("retained during unit read"),
        };
        assert!(matches!(scan.phase, HashNameScanPhase::Ready(Some(2))));
        let HashNameScanStep::Complete(report) = scan.step() else {
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
            let mut scan = HashNameScan::new(&raw);
            let mut observed_steps = 0;
            loop {
                observed_steps += 1;
                match scan.step() {
                    HashNameScanStep::Continue(next) => scan = next,
                    HashNameScanStep::Complete(_) => break,
                }
            }
            assert_eq!(observed_steps, steps);
            assert_eq!(
                HashNameScan::BEGIN_WORK + observed_steps * HashNameScan::STEP_WORK,
                work
            );
        }
    }
}

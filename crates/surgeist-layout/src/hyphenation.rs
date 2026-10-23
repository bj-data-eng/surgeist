//! Block-owned successive-hyphenated-line policy and committed progress.

/// Computed limit on successive hyphenated lines, supplied by the owning block.
///
/// A reached finite limit suppresses automatic hyphenation only. Admitted manual
/// endings, including empty indications, remain permitted. Style adapters own
/// inherited computed-value conversion; every `usize` count is valid.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum HyphenateLimitLines {
    #[default]
    NoLimit,
    Count(usize),
}

/// Remaining capacity, never a saturated claim about an exact sequence length.
/// Reconstructed for each block computation; trials cannot mutate this value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum LineHyphenationState {
    #[default]
    NoLimit,
    Finite {
        limit: usize,
        remaining: usize,
    },
}

impl LineHyphenationState {
    pub(crate) fn new(policy: HyphenateLimitLines) -> Self {
        match policy {
            HyphenateLimitLines::NoLimit => Self::NoLimit,
            HyphenateLimitLines::Count(limit) => Self::Finite {
                limit,
                remaining: limit,
            },
        }
    }

    /// Actual layout consumes remaining capacity; MinContent consults only the
    /// static zero policy, never this computation's live or virtual sequence.
    pub(crate) fn suppressed_limit(self, minimum: bool) -> Option<usize> {
        match self {
            Self::Finite { limit, remaining }
                if if minimum { limit == 0 } else { remaining == 0 } =>
            {
                Some(limit)
            }
            Self::NoLimit | Self::Finite { .. } => None,
        }
    }

    pub(crate) fn commit(&mut self, hyphenated: bool) {
        if let Self::Finite { limit, remaining } = self {
            if hyphenated {
                *remaining = remaining.saturating_sub(1);
            } else {
                *remaining = *limit;
            }
        }
    }
}

/// Computed restriction on the owning unfragmented block's element-terminal tail.
///
/// `Always` prohibits a selected hyphenation ending on the immediate Soft
/// predecessor of the completed block's last content line. Trailing empty lines
/// retain their geometry. Intrinsic queries do not construct a hypothetical tail.
/// Column, page and spread fragmentation are not represented by this input.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum HyphenateLimitLast {
    #[default]
    None,
    Always,
}

/// Facts derived by the actual selected Text/numeric line owner. The ledger
/// consumes only these roles; selection identity remains on the real fitter's
/// immutable line/request and pass basis, not a duplicate request allocation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CommittedInlineLine {
    pub end: surgeist_text::LineEnd,
    pub content: bool,
    pub hyphenated: bool,
}

/// The block traversal retains real run boundaries and child-block interruptions;
/// it never promotes a local Paragraph endpoint into a Soft continuation.
#[derive(Default)]
pub(crate) struct BlockTailLedger {
    entries: Vec<TailEntry>,
}
enum TailEntry {
    Run(Vec<CommittedInlineLine>),
    ChildBlock,
}
impl BlockTailLedger {
    pub fn run(&mut self, lines: Vec<CommittedInlineLine>) {
        self.entries.push(TailEntry::Run(lines));
    }
    pub fn child_block(&mut self) {
        self.entries.push(TailEntry::ChildBlock);
    }

    /// Missing cross-run continuation is consumed only by a potentially protected
    /// selected hyphenation. Ordinary, numeric and separated segments remain
    /// independently supported. Child-owned lines are never flattened here.
    pub fn needs_continuation(&self) -> bool {
        // Locate actual parent content through retained empty rows. Child block
        // lines have their own owner; they supply a custody gap, not a Forced
        // barrier or an invented parent content line.
        let last = self
            .entries
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, entry)| match entry {
                TailEntry::ChildBlock => None,
                TailEntry::Run(lines) => lines
                    .iter()
                    .rposition(|line| line.content)
                    .map(|last| (index, last)),
            });
        let Some((index, last)) = last else {
            return false;
        };
        let TailEntry::Run(lines) = &self.entries[index] else {
            unreachable!()
        };
        let later_child = self.entries[index + 1..]
            .iter()
            .any(|entry| matches!(entry, TailEntry::ChildBlock));
        if later_child && lines[last].hyphenated {
            return true;
        }
        if last > 0 {
            let previous = &lines[last - 1];
            return previous.end == surgeist_text::LineEnd::Soft && previous.hyphenated;
        }
        // A first content line in a later run (or a real child block) has no
        // authenticated parent Soft link. Preserve that gap if relevant; a real
        // Forced ending closes the preceding segment without offset inference.
        for entry in self.entries[..index].iter().rev() {
            match entry {
                TailEntry::ChildBlock => continue,
                TailEntry::Run(lines) => {
                    for line in lines.iter().rev() {
                        if line.end == surgeist_text::LineEnd::Forced {
                            return false;
                        }
                        if line.hyphenated {
                            return true;
                        }
                        if line.content {
                            return false;
                        }
                    }
                }
            }
        }
        false
    }
}

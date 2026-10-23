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

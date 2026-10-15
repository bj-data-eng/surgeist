//! Unit-fed normalization of HTML strings using Infra's normalize-newlines rule.

/// Allocation-free newline normalization over exact UTF-16 code units.
///
/// Infra's [normalize newlines](https://infra.spec.whatwg.org/#normalize-newlines)
/// operation replaces each CR LF pair with LF, then each remaining CR with LF.
/// All other units, including NUL and isolated surrogates, pass through unchanged.
///
/// Keep one normalizer across chunks of the same string. Start a new normalizer
/// for each independent string. Consumers own per-unit work admission,
/// cancellation and materializing returned units into their actual output.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlNewlineNormalizer {
    previous_was_cr: bool,
}

impl HtmlNewlineNormalizer {
    /// Start normalization without a preceding input unit.
    pub const fn new() -> Self {
        Self {
            previous_was_cr: false,
        }
    }

    /// Consume exactly one raw unit and return at most one output unit.
    ///
    /// CR immediately emits LF. An LF immediately following a raw CR returns
    /// `None`; it is consumed, and later input must still be supplied. No unit is
    /// pending at end of input, so no finish operation or lookahead is needed.
    pub fn push(&mut self, unit: u16) -> Option<u16> {
        let previous_was_cr = self.previous_was_cr;
        self.previous_was_cr = unit == 0x000d;
        match unit {
            0x000d => Some(0x000a),
            0x000a if previous_was_cr => None,
            _ => Some(unit),
        }
    }
}

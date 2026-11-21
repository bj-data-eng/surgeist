use surgeist_html::bounded::lexical::is_ascii_whitespace;

/// Strips and collapses ASCII whitespace across ordinary text segment boundaries.
#[derive(Debug, Default)]
pub(super) struct AsciiWhitespaceText {
    seen_non_space: bool,
    pending_space: bool,
}

impl AsciiWhitespaceText {
    /// Returns the emitted prefix of a fixed two-unit buffer.
    pub(super) fn push(&mut self, unit: u16) -> ([u16; 2], usize) {
        if is_ascii_whitespace(unit) {
            self.pending_space = self.seen_non_space;
            return ([0, 0], 0);
        }
        self.seen_non_space = true;
        if std::mem::take(&mut self.pending_space) {
            ([0x20, unit], 2)
        } else {
            ([unit, 0], 1)
        }
    }
}

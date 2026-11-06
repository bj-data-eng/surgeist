use crate::{LinearPosition, LinearSpan};

/// Source text.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct SourceText {
    source_text: Vec<u16>,
}

impl SourceText {
    /// Constructs a new, empty `SourceText` with at least the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            source_text: Vec::with_capacity(capacity),
        }
    }

    /// Get current `LinearPosition`.
    #[must_use]
    pub fn cur_linear_position(&self) -> LinearPosition {
        LinearPosition::new(self.source_text.len())
    }

    /// Get code points from `pos` to the current end.
    #[must_use]
    pub fn get_code_points_from_pos(&self, pos: LinearPosition) -> &[u16] {
        &self.source_text[pos.pos()..]
    }

    /// Get code points within `span`.
    #[must_use]
    pub fn get_code_points_from_span(&self, span: LinearSpan) -> &[u16] {
        &self.source_text[span.start().pos()..span.end().pos()]
    }

    /// Remove last code point.
    #[inline]
    pub fn remove_last_code_point(&mut self) {
        self.source_text.pop();
    }

    /// Collect a code point after admitting its full UTF-16 width.
    ///
    /// # Panics
    ///
    /// On invalid code point.
    #[inline]
    pub fn collect_code_point<E>(
        &mut self,
        cp: u32,
        admit: impl FnOnce(u8) -> Result<(), E>,
    ) -> Result<(), E> {
        if let Ok(cu) = cp.try_into() {
            admit(1)?;
            self.source_text.push(cu);
            return Ok(());
        }
        let cp = cp - 0x10000;
        let cu1 = (cp / 0x400 + 0xD800)
            .try_into()
            .expect("Invalid code point");
        let cu2 = (cp % 0x400 + 0xDC00)
            .try_into()
            .expect("Invalid code point");
        admit(2)?;
        self.source_text.extend_from_slice(&[cu1, cu2]);
        Ok(())
    }
}

const DEFAULT_CAPACITY: usize = 4 * 1024;

impl Default for SourceText {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refusing_an_astral_point_keeps_the_retained_prefix() {
        let mut source = SourceText::with_capacity(0);
        source
            .collect_code_point(0x61, |width| {
                assert_eq!(width, 1);
                Ok::<_, ()>(())
            })
            .unwrap();

        let refusal = source.collect_code_point(0x1F600, |width| {
            assert_eq!(width, 2);
            Err("source retention cap")
        });
        assert_eq!(refusal, Err("source retention cap"));
        assert_eq!(
            source.get_code_points_from_pos(LinearPosition::new(0)),
            [0x61]
        );
    }
}

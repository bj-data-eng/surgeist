use super::{ReadChar, ReadError, ReadUnit};
use crate::frontend::FrontendLimit;

/// Borrowed input whose bytes map directly to Latin1 code points.
#[derive(Debug)]
pub struct Latin1Input<'a> {
    input: &'a [u8],
}

impl<'a> Latin1Input<'a> {
    pub(super) const fn new(input: &'a [u8]) -> Self {
        Self { input }
    }
}

impl ReadChar for Latin1Input<'_> {
    fn next_char(
        &mut self,
        admit: &mut impl FnMut(ReadUnit) -> Result<(), FrontendLimit>,
    ) -> Result<Option<u32>, ReadError> {
        admit(ReadUnit::Latin1Byte)?;
        let Some((&byte, remaining)) = self.input.split_first() else {
            return Ok(None);
        };
        self.input = remaining;
        Ok(Some(byte.into()))
    }
}

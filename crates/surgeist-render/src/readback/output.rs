use crate::{
    ImageBuffer, PhysicalSize, Result,
    accounting::{Account, AccountedVec},
    image::AccountedImageBuffer,
};

/// Complete decoded bytes remain paired with CPU custody until final handoff.
pub(crate) struct ReadbackOutput(AccountedImageBuffer);

impl ReadbackOutput {
    pub(super) fn try_new(size: PhysicalSize, rgba: AccountedVec<u8>) -> Result<Self> {
        AccountedImageBuffer::try_new(size, rgba).map(Self)
    }

    pub(crate) fn empty(size: PhysicalSize, account: &Account) -> Result<Self> {
        Self::try_new(size, account.retained(0)?)
    }

    /// All validation and report extraction precedes this final checked move.
    pub(crate) fn transfer(self) -> Result<ImageBuffer> {
        self.0.transfer()
    }
}

#[cfg(test)]
mod tests;

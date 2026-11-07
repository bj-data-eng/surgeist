use crate::{
    BackendErrorCode, Error, ImageBuffer, PhysicalSize, Result,
    accounting::{Account, AccountedVec, BackedValue},
};

/// Complete decoded bytes remain paired with CPU custody until final handoff.
pub(crate) struct ReadbackOutput(BackedValue<ImageBuffer>);

impl ReadbackOutput {
    pub(super) fn try_new(size: PhysicalSize, rgba: AccountedVec<u8>) -> Result<Self> {
        let (values, hold) = rgba.into_parts();
        let storage = hold.storage();
        let guarded = BackedValue::new(values, [Some(hold), None]);
        storage.phase(|| {
            let (values, backing, import) = guarded.into_parts();
            assert!(
                import.is_none(),
                "readback output owns one direct allocation"
            );
            match ImageBuffer::try_new(size, values) {
                Ok(image) => Ok(Self(BackedValue::new(image, backing))),
                Err(source) => {
                    // The constructor destroyed values inside this actual CPU
                    // phase while their admission was still live.
                    drop(backing);
                    Err(Error::new(
                        BackendErrorCode::ReadbackFailed,
                        "decoded readback bytes did not form a valid RGBA8 image",
                    )
                    .with_source(source))
                }
            }
        })
    }

    pub(crate) fn empty(size: PhysicalSize, account: &Account) -> Result<Self> {
        Self::try_new(size, account.retained(0)?)
    }

    /// All validation and report extraction precedes this final checked move.
    pub(crate) fn transfer(self) -> Result<ImageBuffer> {
        self.0.transfer_readback()
    }
}

#[cfg(test)]
mod tests;

//! Compiled RGB-to-PCS lookup tables from ICC.1:2022 §§6.3.4 and 10.10–10.12.

use std::mem::size_of;

use super::{
    Account, AccountedVec, Extent, Hold, PcsColor, ProfileError, ProfileResult, RenderResource,
    Result, field, sample_invariant, tag_kind, u16_at, u32_at,
};

mod sampling;
mod staged;

use sampling::{ChannelTables, Grid, Precision};
use staged::Staged;

const CHANNELS: usize = 3;
const LEGACY_MATRIX: [u32; 9] = [0x10000, 0, 0, 0, 0x10000, 0, 0, 0, 0x10000];

/// Couples a validated transform and its tag range to the original backing.
pub(super) struct RgbLutProfile {
    pipeline: Pipeline,
    _hold: Hold,
}

enum Pipeline {
    Legacy(Box<Compiled<Legacy>>),
    Staged(Box<Compiled<Staged>>),
}

struct Compiled<T> {
    stages: T,
    bytes: AccountedVec<u8>,
    tag: Extent,
    encoding: PcsEncoding,
}

impl RgbLutProfile {
    pub(super) fn read(
        bytes: AccountedVec<u8>,
        tag: Extent,
        version: u8,
        pcs: &[u8; 4],
        account: &Account,
    ) -> ProfileResult<Self> {
        let payload = tag.payload(&bytes)?;
        let kind = tag_kind(payload, account)?;
        let encoding = match (pcs, &kind) {
            (b"XYZ ", _) => PcsEncoding::Xyz,
            (b"Lab ", b"mft2") => PcsEncoding::LegacyLab,
            (b"Lab ", _) => PcsEncoding::Lab,
            _ => return Err(ProfileError::Invalid),
        };
        let (pipeline, hold) = match &kind {
            b"mft1" | b"mft2" => {
                let precision = if &kind == b"mft1" {
                    Precision::Eight
                } else {
                    Precision::Sixteen
                };
                let legacy = Legacy::read(payload, precision, account)?;
                // ICC §10.11 defines no interoperable 8-bit XYZ PCS encoding.
                if matches!(precision, Precision::Eight) && pcs == b"XYZ " {
                    return Err(ProfileError::Unsupported);
                }
                let hold = account.acquire(
                    RenderResource::ScratchCpuBytes,
                    size_of::<Compiled<Legacy>>(),
                )?;
                let compiled = Compiled {
                    stages: legacy,
                    bytes,
                    tag,
                    encoding,
                };
                (Pipeline::Legacy(Box::new(compiled)), hold)
            }
            b"mAB " if version == 4 => {
                let hold = account.acquire(
                    RenderResource::ScratchCpuBytes,
                    size_of::<Compiled<Staged>>(),
                )?;
                let stages = Staged::read(payload, account)?;
                let compiled = Compiled {
                    stages,
                    bytes,
                    tag,
                    encoding,
                };
                (Pipeline::Staged(Box::new(compiled)), hold)
            }
            _ => return Err(ProfileError::Invalid),
        };

        Ok(Self {
            pipeline,
            _hold: hold,
        })
    }

    pub(super) fn pcs(&self, rgb: [u8; CHANNELS], account: &Account) -> Result<PcsColor> {
        let mut input = [0.0; CHANNELS];
        for (value, byte) in input.iter_mut().zip(rgb) {
            *value = account.coordinate(|| f64::from(byte) / 255.0)?;
        }
        match &self.pipeline {
            Pipeline::Legacy(legacy) => legacy.pcs(input, account, Legacy::evaluate),
            Pipeline::Staged(staged) => staged.pcs(input, account, Staged::evaluate),
        }
    }
}

impl<T> Compiled<T> {
    fn pcs(
        &self,
        input: [f64; CHANNELS],
        account: &Account,
        evaluate: impl FnOnce(&T, [f64; CHANNELS], &[u8], &Account) -> Result<[f64; CHANNELS]>,
    ) -> Result<PcsColor> {
        let payload = self
            .tag
            .payload(&self.bytes)
            .map_err(|_| sample_invariant())?;
        let output = evaluate(&self.stages, input, payload, account)?;
        self.encoding.decode(output, account)
    }
}

enum PcsEncoding {
    Xyz,
    Lab,
    LegacyLab,
}

impl PcsEncoding {
    fn decode(&self, mut values: [f64; CHANNELS], account: &Account) -> Result<PcsColor> {
        match self {
            Self::Xyz => {
                // The normalized lookup result represents a u1Fixed15 code.
                // ICC §6.3.4.2 and Annex F.3 prescribe this factor, also for mAB.
                for value in &mut values {
                    *value = account.coordinate(|| *value * (65535.0 / 32768.0))?;
                }
                Ok(PcsColor::XyzD50(values))
            }
            Self::Lab => {
                let [l, a, b] = values;
                Ok(PcsColor::Lab([
                    account.coordinate(|| 100.0 * l)?,
                    account.coordinate(|| 255.0_f64.mul_add(a, -128.0))?,
                    account.coordinate(|| 255.0_f64.mul_add(b, -128.0))?,
                ]))
            }
            Self::LegacyLab => {
                // lut16Type always uses its legacy Lab codes (ICC §10.10).
                // L*=100 is 0xFF00; a*=b*=0 is 0x8000. Bound lightness to
                // its valid range while keeping the extra legacy chroma range.
                let [l, a, b] = values;
                Ok(PcsColor::Lab([
                    account.coordinate(|| (l * (100.0 * 65535.0 / 65280.0)).min(100.0))?,
                    account.coordinate(|| (65535.0_f64 / 256.0).mul_add(a, -128.0))?,
                    account.coordinate(|| (65535.0_f64 / 256.0).mul_add(b, -128.0))?,
                ]))
            }
        }
    }
}

struct Legacy {
    input: ChannelTables,
    grid: Grid,
    output: ChannelTables,
}

impl Legacy {
    fn read(payload: &[u8], precision: Precision, account: &Account) -> ProfileResult<Self> {
        let [input, output, grid, reserved] = account.coordinate(|| field::<4>(payload, 8))??;
        if usize::from(input) != CHANNELS || usize::from(output) != CHANNELS || reserved != 0 {
            return Err(ProfileError::Invalid);
        }
        // The source is RGB. The legacy matrix is used only for XYZ input.
        for (index, expected) in LEGACY_MATRIX.into_iter().enumerate() {
            if account.coordinate(|| u32_at(payload, 12 + index * 4))?? != expected {
                return Err(ProfileError::Invalid);
            }
        }
        let (start, input_entries, output_entries) = match precision {
            Precision::Eight => (48, 256, 256),
            Precision::Sixteen => {
                let input_entries = account.coordinate(|| u16_at(payload, 48))??;
                let output_entries = account.coordinate(|| u16_at(payload, 50))??;
                if !(2..=4096).contains(&input_entries) || !(2..=4096).contains(&output_entries) {
                    return Err(ProfileError::Invalid);
                }
                (52, usize::from(input_entries), usize::from(output_entries))
            }
        };
        let length = payload.len();
        let input = ChannelTables::read(start, input_entries, precision, length)?;
        let grid = Grid::read(input.end(), [grid; CHANNELS], precision, length)?;
        let output = ChannelTables::read(grid.end(), output_entries, precision, length)?;
        if output.end() != length {
            return Err(ProfileError::Invalid);
        }
        Ok(Self {
            input,
            grid,
            output,
        })
    }

    fn evaluate(
        &self,
        rgb: [f64; CHANNELS],
        payload: &[u8],
        account: &Account,
    ) -> Result<[f64; CHANNELS]> {
        let input = self.input.evaluate(rgb, payload, account)?;
        let mapped = self.grid.evaluate(input, payload, account)?;
        self.output.evaluate(mapped, payload, account)
    }
}

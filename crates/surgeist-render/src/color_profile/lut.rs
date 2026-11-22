//! Integer ICC lookup tables in both directions (ICC.1:2022 §§6.3.4 and 10.10–10.13).

use std::mem::size_of;

use super::{
    Account, AccountedVec, Direction, Extent, Hold, PcsColor, ProfileError, ProfileResult,
    RenderResource, Result, field, fixed, sample_invariant, tag_kind, u16_at,
};

mod sampling;
mod staged;

use sampling::{ChannelTables, Grid, Precision};
use staged::Staged;

const PCS_CHANNELS: usize = 3;
pub(super) const MAX_CHANNELS: usize = 15;

/// Couples the PNG RGB transform to its original accounted profile backing.
pub(super) struct RgbLutProfile(Lut<OwnedTag>);

struct OwnedTag {
    bytes: AccountedVec<u8>,
    tag: Extent,
}

pub(super) trait TagBacking {
    fn payload(&self) -> ProfileResult<&[u8]>;
}

impl TagBacking for OwnedTag {
    fn payload(&self) -> ProfileResult<&[u8]> {
        self.tag.payload(&self.bytes)
    }
}

impl TagBacking for &[u8] {
    fn payload(&self) -> ProfileResult<&[u8]> {
        Ok(self)
    }
}

impl RgbLutProfile {
    pub(super) fn read(
        bytes: AccountedVec<u8>,
        tag: Extent,
        version: u8,
        pcs: &[u8; 4],
        account: &Account,
    ) -> ProfileResult<Self> {
        let lut = Lut::read(
            OwnedTag { bytes, tag },
            version,
            (b"RGB ", pcs),
            Direction::ToPcs,
            PCS_CHANNELS,
            PCS_CHANNELS,
            account,
        )?;
        Ok(Self(lut))
    }

    pub(super) fn pcs(&self, rgb: [u8; PCS_CHANNELS], account: &Account) -> Result<PcsColor> {
        let mut input = [0.0; PCS_CHANNELS];
        for (value, byte) in input.iter_mut().zip(rgb) {
            *value = account.coordinate(|| f64::from(byte) / 255.0)?;
        }
        let mut output = [0.0; PCS_CHANNELS];
        self.0.evaluate(&input, &mut output, account)?;
        Ok(match self.0.encoding {
            PcsEncoding::Xyz => PcsColor::XyzD50(output),
            PcsEncoding::Lab | PcsEncoding::LegacyLab => PcsColor::Lab(output),
        })
    }
}

/// A validated integer transform with extents into its original tag payload.
pub(super) struct Lut<B> {
    pipeline: Pipeline<B>,
    direction: Direction,
    input_channels: usize,
    output_channels: usize,
    encoding: PcsEncoding,
    _hold: Hold,
}

enum Pipeline<B> {
    Legacy(Box<Compiled<Legacy, B>>),
    Staged(Box<Compiled<Staged, B>>),
}

struct Compiled<S, B> {
    stages: S,
    backing: B,
}

impl<B: TagBacking> Lut<B> {
    pub(super) fn read(
        backing: B,
        version: u8,
        spaces: (&[u8; 4], &[u8; 4]),
        direction: Direction,
        input_channels: usize,
        output_channels: usize,
        account: &Account,
    ) -> ProfileResult<Self> {
        let (device_channels, pcs_channels) = match direction {
            Direction::ToPcs => (input_channels, output_channels),
            Direction::FromPcs => (output_channels, input_channels),
        };
        if !(1..=MAX_CHANNELS).contains(&device_channels) || pcs_channels != PCS_CHANNELS {
            return Err(ProfileError::Invalid);
        }

        let payload = backing.payload()?;
        let (device, pcs) = spaces;
        let kind = tag_kind(payload, account)?;
        let encoding = match (pcs, &kind) {
            (b"XYZ ", _) => PcsEncoding::Xyz,
            (b"Lab ", b"mft2") => PcsEncoding::LegacyLab,
            (b"Lab ", _) => PcsEncoding::Lab,
            _ => return Err(ProfileError::Invalid),
        };
        let (pipeline, hold) = match (&kind, direction) {
            (b"mft1" | b"mft2", _) => {
                let precision = if &kind == b"mft1" {
                    Precision::Eight
                } else {
                    Precision::Sixteen
                };
                let matrix_rule = match (version, direction, device, pcs) {
                    (_, Direction::FromPcs, _, b"XYZ ") | (2, Direction::ToPcs, b"XYZ ", _) => {
                        LegacyMatrixRule::XyzInput
                    }
                    _ => LegacyMatrixRule::Identity,
                };
                let legacy = Legacy::read(
                    payload,
                    precision,
                    matrix_rule,
                    input_channels,
                    output_channels,
                    account,
                )?;
                // ICC §10.11 leaves 8-bit PCSXYZ to implementation-specific conventions.
                if matches!(precision, Precision::Eight) && pcs == b"XYZ " {
                    return Err(ProfileError::Unsupported);
                }
                let hold = account.acquire(
                    RenderResource::ScratchCpuBytes,
                    size_of::<Compiled<Legacy, B>>(),
                )?;
                let compiled = Compiled {
                    stages: legacy,
                    backing,
                };
                (Pipeline::Legacy(Box::new(compiled)), hold)
            }
            (b"mAB ", Direction::ToPcs) | (b"mBA ", Direction::FromPcs) if version == 4 => {
                let hold = account.acquire(
                    RenderResource::ScratchCpuBytes,
                    size_of::<Compiled<Staged, B>>(),
                )?;
                let staged =
                    Staged::read(payload, direction, input_channels, output_channels, account)?;
                let compiled = Compiled {
                    stages: staged,
                    backing,
                };
                (Pipeline::Staged(Box::new(compiled)), hold)
            }
            _ => return Err(ProfileError::Invalid),
        };

        Ok(Self {
            pipeline,
            direction,
            input_channels,
            output_channels,
            encoding,
            _hold: hold,
        })
    }

    pub(super) fn evaluate(
        &self,
        input: &[f64],
        output: &mut [f64],
        account: &Account,
    ) -> Result<()> {
        if input.len() != self.input_channels
            || output.len() != self.output_channels
            || input.iter().any(|value| !value.is_finite())
        {
            return Err(sample_invariant());
        }

        let mut normalized = [0.0; MAX_CHANNELS];
        let input = match self.direction {
            Direction::ToPcs => {
                let normalized = &mut normalized[..input.len()];
                for (destination, &value) in normalized.iter_mut().zip(input) {
                    *destination = if (0.0..=1.0).contains(&value) {
                        value
                    } else {
                        account.coordinate(|| value.clamp(0.0, 1.0))?
                    };
                }
                normalized
            }
            Direction::FromPcs => {
                let pcs = input.first_chunk().ok_or_else(sample_invariant)?;
                let normalized = normalized.first_chunk_mut().ok_or_else(sample_invariant)?;
                *normalized = self.encoding.encode(*pcs, account)?;
                normalized.as_mut_slice()
            }
        };

        let mut values = [0.0; MAX_CHANNELS];
        let values = &mut values[..output.len()];
        match &self.pipeline {
            Pipeline::Legacy(legacy) => {
                let payload = legacy.backing.payload().map_err(|_| sample_invariant())?;
                legacy.stages.evaluate(input, values, payload, account)?;
            }
            Pipeline::Staged(staged) => {
                let payload = staged.backing.payload().map_err(|_| sample_invariant())?;
                staged.stages.evaluate(input, values, payload, account)?;
            }
        }

        match self.direction {
            Direction::ToPcs => {
                let pcs = values.first_chunk().ok_or_else(sample_invariant)?;
                let decoded = self.encoding.decode(*pcs, account)?;
                output.copy_from_slice(&decoded);
            }
            Direction::FromPcs => output.copy_from_slice(values),
        }
        Ok(())
    }
}

enum PcsEncoding {
    Xyz,
    Lab,
    LegacyLab,
}

impl PcsEncoding {
    fn encode(
        &self,
        mut values: [f64; PCS_CHANNELS],
        account: &Account,
    ) -> Result<[f64; PCS_CHANNELS]> {
        match self {
            Self::Xyz => {
                for value in &mut values {
                    *value =
                        account.coordinate(|| (*value * (32768.0 / 65535.0)).clamp(0.0, 1.0))?;
                }
                Ok(values)
            }
            Self::Lab => {
                let [l, a, b] = values;
                Ok([
                    account.coordinate(|| (l / 100.0).clamp(0.0, 1.0))?,
                    account.coordinate(|| ((a + 128.0) / 255.0).clamp(0.0, 1.0))?,
                    account.coordinate(|| ((b + 128.0) / 255.0).clamp(0.0, 1.0))?,
                ])
            }
            Self::LegacyLab => {
                let [l, a, b] = values;
                Ok([
                    account.coordinate(|| l.clamp(0.0, 100.0) * (65280.0 / (100.0 * 65535.0)))?,
                    account.coordinate(|| ((a + 128.0) * (256.0 / 65535.0)).clamp(0.0, 1.0))?,
                    account.coordinate(|| ((b + 128.0) * (256.0 / 65535.0)).clamp(0.0, 1.0))?,
                ])
            }
        }
    }

    fn decode(
        &self,
        mut values: [f64; PCS_CHANNELS],
        account: &Account,
    ) -> Result<[f64; PCS_CHANNELS]> {
        match self {
            Self::Xyz => {
                // The normalized lookup result represents a u1Fixed15 code.
                // ICC §6.3.4.2 and Annex F.3 prescribe this factor, also for mAB/mBA.
                for value in &mut values {
                    *value = account.coordinate(|| *value * (65535.0 / 32768.0))?;
                }
                Ok(values)
            }
            Self::Lab => {
                let [l, a, b] = values;
                Ok([
                    account.coordinate(|| 100.0 * l)?,
                    account.coordinate(|| 255.0_f64.mul_add(a, -128.0))?,
                    account.coordinate(|| 255.0_f64.mul_add(b, -128.0))?,
                ])
            }
            Self::LegacyLab => {
                // lut16Type always uses its legacy Lab codes (ICC §10.10).
                // L*=100 is 0xFF00; a*=b*=0 is 0x8000. Bound lightness to
                // its valid range while keeping the extra legacy chroma range.
                let [l, a, b] = values;
                Ok([
                    account.coordinate(|| (l * (100.0 * 65535.0 / 65280.0)).min(100.0))?,
                    account.coordinate(|| (65535.0_f64 / 256.0).mul_add(a, -128.0))?,
                    account.coordinate(|| (65535.0_f64 / 256.0).mul_add(b, -128.0))?,
                ])
            }
        }
    }
}

struct Legacy {
    matrix: Option<[[f64; PCS_CHANNELS]; PCS_CHANNELS]>,
    input: ChannelTables,
    grid: Grid,
    output: ChannelTables,
}

#[derive(Clone, Copy)]
enum LegacyMatrixRule {
    Identity,
    XyzInput,
}

impl Legacy {
    fn read(
        payload: &[u8],
        precision: Precision,
        matrix_rule: LegacyMatrixRule,
        input_channels: usize,
        output_channels: usize,
        account: &Account,
    ) -> ProfileResult<Self> {
        let [input, output, grid, reserved] = account.coordinate(|| field::<4>(payload, 8))??;
        if usize::from(input) != input_channels
            || usize::from(output) != output_channels
            || reserved != 0
        {
            return Err(ProfileError::Invalid);
        }
        let matrix = legacy_matrix(payload, matrix_rule, account)?;
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
        let input = ChannelTables::read(start, input_entries, input_channels, precision, length)?;
        let grid = Grid::read(
            input.end(),
            &[grid; MAX_CHANNELS][..input_channels],
            output_channels,
            precision,
            length,
        )?;
        let output = ChannelTables::read(
            grid.end(),
            output_entries,
            output_channels,
            precision,
            length,
        )?;
        if output.end() != length {
            return Err(ProfileError::Invalid);
        }
        Ok(Self {
            matrix,
            input,
            grid,
            output,
        })
    }

    fn evaluate(
        &self,
        input: &[f64],
        output: &mut [f64],
        payload: &[u8],
        account: &Account,
    ) -> Result<()> {
        let xyz;
        let input = if let Some(matrix) = &self.matrix {
            let coordinates =
                <&[f64; PCS_CHANNELS]>::try_from(input).map_err(|_| sample_invariant())?;
            xyz = apply_legacy_matrix(matrix, coordinates, account)?;
            xyz.as_slice()
        } else {
            input
        };
        let mut mapped_inputs = [0.0; MAX_CHANNELS];
        let mut mapped_grid = [0.0; MAX_CHANNELS];
        let mapped_inputs = &mut mapped_inputs[..input.len()];
        let mapped_grid = &mut mapped_grid[..output.len()];
        self.input
            .evaluate(input, mapped_inputs, payload, account)?;
        self.grid
            .evaluate(mapped_inputs, mapped_grid, payload, account)?;
        self.output.evaluate(mapped_grid, output, payload, account)
    }
}

fn legacy_matrix(
    payload: &[u8],
    rule: LegacyMatrixRule,
    account: &Account,
) -> ProfileResult<Option<[[f64; PCS_CHANNELS]; PCS_CHANNELS]>> {
    let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut matrix = [[0.0; PCS_CHANNELS]; PCS_CHANNELS];
    let mut differs = false;
    for (index, (coefficient, expected)) in matrix
        .iter_mut()
        .flatten()
        .zip(identity.into_iter().flatten())
        .enumerate()
    {
        *coefficient = account.coordinate(|| fixed(payload, 12 + index * 4))??;
        if *coefficient != expected {
            if matches!(rule, LegacyMatrixRule::Identity) {
                return Err(ProfileError::Invalid);
            }
            differs = true;
        }
    }
    Ok(differs.then_some(matrix))
}

fn apply_legacy_matrix(
    matrix: &[[f64; PCS_CHANNELS]; PCS_CHANNELS],
    input: &[f64; PCS_CHANNELS],
    account: &Account,
) -> Result<[f64; PCS_CHANNELS]> {
    let mut output = [0.0; PCS_CHANNELS];
    for (value, row) in output.iter_mut().zip(matrix) {
        for (&coefficient, &coordinate) in row.iter().zip(input) {
            *value = account.coordinate(|| coefficient.mul_add(coordinate, *value))?;
        }
        *value = account.coordinate(|| value.clamp(0.0, 1.0))?;
    }
    Ok(output)
}

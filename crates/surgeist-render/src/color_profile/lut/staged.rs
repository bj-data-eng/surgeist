//! ICC lutAToBType and lutBToAType stages (ICC.1:2022 §§10.12–10.13).

use crate::color_profile::{
    Account, Curve, CurveInput, Extent, ParametricCurve, ProfileError, ProfileResult, RenderWork,
    Result, field, fixed, sample_invariant, sampled_curve_size, tag_kind, u32_at,
};

use super::{
    Direction, MAX_CHANNELS,
    sampling::{Grid, Precision},
};

const PCS_CHANNELS: usize = 3;
const A_CURVES: usize = 2 * PCS_CHANNELS;
const CURVE_EXTENTS: usize = A_CURVES + MAX_CHANNELS;
const STAGE_EXTENTS: usize = CURVE_EXTENTS + 2;

pub(super) struct Staged {
    direction: Direction,
    device_channels: usize,
    b: [Curve; PCS_CHANNELS],
    matrix: Option<MatrixStage>,
    clut: Option<ClutStage>,
}

struct ClutStage {
    a: [Curve; MAX_CHANNELS],
    grid: Grid,
}

struct MatrixStage {
    m: [Curve; PCS_CHANNELS],
    rows: [[f64; PCS_CHANNELS]; PCS_CHANNELS],
    offsets: [f64; PCS_CHANNELS],
}

impl Staged {
    pub(super) fn read(
        payload: &[u8],
        direction: Direction,
        input_channels: usize,
        output_channels: usize,
        account: &Account,
    ) -> ProfileResult<Self> {
        let (signature, device_channels, pcs_channels) = match direction {
            Direction::ToPcs => (b"mAB \0\0\0\0", input_channels, output_channels),
            Direction::FromPcs => (b"mBA \0\0\0\0", output_channels, input_channels),
        };
        if !(1..=MAX_CHANNELS).contains(&device_channels) || pcs_channels != PCS_CHANNELS {
            return Err(ProfileError::Invalid);
        }

        let [b, matrix, m, clut, a] =
            stage_offsets(payload, signature, input_channels, output_channels, account)?;
        if b == 0
            || (matrix == 0) != (m == 0)
            || (clut == 0) != (a == 0)
            || (clut == 0 && input_channels != output_channels)
        {
            return Err(ProfileError::Invalid);
        }

        let mut extents = [None; STAGE_EXTENTS];
        let b = read_curves(payload, b, &mut extents[..PCS_CHANNELS], account)?;
        let matrix = if matrix == 0 {
            None
        } else {
            let curves = read_curves(payload, m, &mut extents[PCS_CHANNELS..A_CURVES], account)?;
            extents[CURVE_EXTENTS] = Some(element(payload, matrix, 48)?);
            Some(MatrixStage::read(curves, payload, matrix, account)?)
        };
        let clut = if clut == 0 {
            None
        } else {
            let a = read_curves(
                payload,
                a,
                &mut extents[A_CURVES..A_CURVES + device_channels],
                account,
            )?;
            let grid = clut_grid(payload, clut, input_channels, output_channels, account)?;
            extents[CURVE_EXTENTS + 1] = Some(element(payload, clut, grid.end() - clut)?);
            Some(ClutStage { a, grid })
        };
        validate_extents(&extents, account)?;
        Ok(Self {
            direction,
            device_channels,
            b,
            matrix,
            clut,
        })
    }

    pub(super) fn evaluate(
        &self,
        input: &[f64],
        output: &mut [f64],
        payload: &[u8],
        account: &Account,
    ) -> Result<()> {
        match self.direction {
            Direction::ToPcs => {
                let output: &mut [f64; PCS_CHANNELS] =
                    output.try_into().map_err(|_| sample_invariant())?;
                if input.len() != self.device_channels {
                    return Err(sample_invariant());
                }
                let mut pcs = if let Some(stage) = &self.clut {
                    let mut device = [0.0; MAX_CHANNELS];
                    let device = &mut device[..self.device_channels];
                    device.copy_from_slice(input);
                    evaluate_curves(&stage.a[..self.device_channels], device, payload, account)?;
                    let mut pcs = [0.0; PCS_CHANNELS];
                    stage.grid.evaluate(device, &mut pcs, payload, account)?;
                    pcs
                } else {
                    input.try_into().map_err(|_| sample_invariant())?
                };
                if let Some(stage) = &self.matrix {
                    evaluate_curves(&stage.m, &mut pcs, payload, account)?;
                    pcs = stage.apply(pcs, account)?;
                }
                evaluate_curves(&self.b, &mut pcs, payload, account)?;
                *output = pcs;
            }
            Direction::FromPcs => {
                let mut pcs: [f64; PCS_CHANNELS] =
                    input.try_into().map_err(|_| sample_invariant())?;
                if output.len() != self.device_channels {
                    return Err(sample_invariant());
                }
                evaluate_curves(&self.b, &mut pcs, payload, account)?;
                if let Some(stage) = &self.matrix {
                    pcs = stage.apply(pcs, account)?;
                    evaluate_curves(&stage.m, &mut pcs, payload, account)?;
                }
                if let Some(stage) = &self.clut {
                    let mut device = [0.0; MAX_CHANNELS];
                    let device = &mut device[..self.device_channels];
                    stage.grid.evaluate(&pcs, device, payload, account)?;
                    evaluate_curves(&stage.a[..self.device_channels], device, payload, account)?;
                    output.copy_from_slice(device);
                } else {
                    let output: &mut [f64; PCS_CHANNELS] =
                        output.try_into().map_err(|_| sample_invariant())?;
                    *output = pcs;
                }
            }
        }
        Ok(())
    }
}

fn stage_offsets(
    payload: &[u8],
    signature: &[u8; 8],
    input_channels: usize,
    output_channels: usize,
    account: &Account,
) -> ProfileResult<[usize; 5]> {
    const HEADER: usize = 32;
    let header = account.coordinate(|| field::<12>(payload, 0))??;
    if header[..8] != *signature
        || usize::from(header[8]) != input_channels
        || usize::from(header[9]) != output_channels
        || header[10..] != [0, 0]
    {
        return Err(ProfileError::Invalid);
    }
    let mut offsets = [0; 5];
    for (index, offset) in offsets.iter_mut().enumerate() {
        let raw = account.coordinate(|| u32_at(payload, 12 + index * 4))??;
        *offset = usize::try_from(raw).map_err(|_| ProfileError::Invalid)?;
        if *offset != 0 && (!(HEADER..payload.len()).contains(offset) || !offset.is_multiple_of(4))
        {
            return Err(ProfileError::Invalid);
        }
    }
    Ok(offsets)
}

fn read_curves<const CAPACITY: usize>(
    payload: &[u8],
    mut next: usize,
    extents: &mut [Option<Extent>],
    account: &Account,
) -> ProfileResult<[Curve; CAPACITY]> {
    let mut curves = [const { Curve::Identity }; CAPACITY];
    let active = curves
        .get_mut(..extents.len())
        .ok_or(ProfileError::Invalid)?;
    for (curve, extent) in active.iter_mut().zip(extents) {
        let tail = payload.get(next..).ok_or(ProfileError::Invalid)?;
        let encoded_len = match &tag_kind(tail, account)? {
            b"curv" => {
                let count = account.coordinate(|| u32_at(tail, 8))??;
                sampled_curve_size(count)?
            }
            b"para" => {
                let (_, count) = ParametricCurve::kind_and_count(tail, account)?;
                12 + count * 4
            }
            _ => return Err(ProfileError::Invalid),
        };
        let aligned = element(payload, next, encoded_len)?;
        let raw = Extent {
            start: next,
            end: next + encoded_len,
        };
        *curve = Curve::read(raw, payload, 4, account)?;
        *extent = Some(aligned);
        next = aligned.end;
    }
    Ok(curves)
}

fn evaluate_curves(
    curves: &[Curve],
    values: &mut [f64],
    payload: &[u8],
    account: &Account,
) -> Result<()> {
    for (curve, value) in curves.iter().zip(values) {
        *value = curve.evaluate(CurveInput::Unit(*value), payload, account)?;
    }
    Ok(())
}

impl MatrixStage {
    fn read(
        m: [Curve; PCS_CHANNELS],
        payload: &[u8],
        start: usize,
        account: &Account,
    ) -> ProfileResult<Self> {
        let mut rows = [[0.0; PCS_CHANNELS]; PCS_CHANNELS];
        let mut offsets = [0.0; PCS_CHANNELS];
        // The nine row-major coefficients precede the three offsets.
        for (index, value) in rows.iter_mut().flatten().chain(&mut offsets).enumerate() {
            *value = account.coordinate(|| fixed(payload, start + index * 4))??;
        }
        Ok(Self { m, rows, offsets })
    }

    fn apply(&self, input: [f64; PCS_CHANNELS], account: &Account) -> Result<[f64; PCS_CHANNELS]> {
        let mut output = self.offsets;
        for (value, row) in output.iter_mut().zip(&self.rows) {
            for (&coefficient, input) in row.iter().zip(input) {
                *value = account.coordinate(|| coefficient.mul_add(input, *value))?;
            }
            // B (A-to-B) or M (B-to-A) receives the clipped affine result.
            *value = account.coordinate(|| value.clamp(0.0, 1.0))?;
        }
        Ok(output)
    }
}

fn clut_grid(
    payload: &[u8],
    start: usize,
    input_channels: usize,
    output_channels: usize,
    account: &Account,
) -> ProfileResult<Grid> {
    let tail = payload.get(start..).ok_or(ProfileError::Invalid)?;
    let points = account.coordinate(|| field::<16>(tail, 0))??;
    let (dimensions, unused) = points.split_at(input_channels);
    let precision = match account.coordinate(|| field::<4>(tail, 16))?? {
        [1, 0, 0, 0] => Precision::Eight,
        [2, 0, 0, 0] => Precision::Sixteen,
        _ => return Err(ProfileError::Invalid),
    };
    if unused.iter().any(|&point| point != 0) {
        return Err(ProfileError::Invalid);
    }
    // Reading the complete 20-byte CLUT header above proved this addition.
    Grid::read(
        start + 20,
        dimensions,
        output_channels,
        precision,
        payload.len(),
    )
}

fn validate_extents(
    stages: &[Option<Extent>; STAGE_EXTENTS],
    account: &Account,
) -> ProfileResult<()> {
    // Exact aliases may share whole embedded curves. Reject partial or
    // differently typed overlap while admitting all four legal stage sets.
    for (index, extent) in stages.iter().enumerate() {
        let Some(extent) = extent else { continue };
        for (other_index, other) in stages[..index].iter().enumerate() {
            let Some(other) = other else { continue };
            account.work(RenderWork::IccProfileTagVisits, 2)?;
            if extent.start < other.end
                && other.start < extent.end
                && !(index < CURVE_EXTENTS && other_index < CURVE_EXTENTS && extent == other)
            {
                return Err(ProfileError::Invalid);
            }
        }
    }
    Ok(())
}

fn element(payload: &[u8], start: usize, size: usize) -> ProfileResult<Extent> {
    let end = start.checked_add(size).ok_or(ProfileError::Invalid)?;
    let padded = end
        .checked_next_multiple_of(4)
        .ok_or(ProfileError::Invalid)?;
    // Alignment after the declared tag size belongs to the outer tag table.
    let padding = payload
        .get(end..padded.min(payload.len()))
        .ok_or(ProfileError::Invalid)?;
    if padding.iter().any(|&byte| byte != 0) {
        return Err(ProfileError::Invalid);
    }
    Ok(Extent { start, end: padded })
}

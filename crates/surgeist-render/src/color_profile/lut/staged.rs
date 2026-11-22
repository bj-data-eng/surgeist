//! ICC lutAToBType stage admission and execution (ICC.1:2022 §10.12).

use crate::color_profile::{
    Account, Curve, CurveInput, Extent, ParametricCurve, ProfileError, ProfileResult, RenderWork,
    Result, field, fixed, sampled_curve_size, tag_kind, u32_at,
};

use super::{
    CHANNELS,
    sampling::{Grid, Precision},
};

const CURVE_EXTENTS: usize = 3 * CHANNELS;
const STAGE_EXTENTS: usize = CURVE_EXTENTS + 2;

pub(super) struct Staged {
    clut: Option<ClutStage>,
    matrix: Option<MatrixStage>,
    output: Curves,
}

struct ClutStage {
    input: Curves,
    grid: Grid,
}

struct MatrixStage {
    input: Curves,
    rows: [[f64; CHANNELS]; CHANNELS],
    offsets: [f64; CHANNELS],
}

impl Staged {
    pub(super) fn read(payload: &[u8], account: &Account) -> ProfileResult<Self> {
        let [b, matrix, m, clut, a] = stage_offsets(payload, account)?;
        if b == 0 || (matrix == 0) != (m == 0) || (clut == 0) != (a == 0) {
            return Err(ProfileError::Invalid);
        }

        let mut curve_extents = [[None; CHANNELS]; 3];
        let mut extents = [None; STAGE_EXTENTS];
        let output = Curves::read(payload, b, &mut curve_extents[0], account)?;
        let matrix = if matrix == 0 {
            None
        } else {
            let input = Curves::read(payload, m, &mut curve_extents[1], account)?;
            extents[CURVE_EXTENTS] = Some(element(payload, matrix, 48)?);
            Some(MatrixStage::read(input, payload, matrix, account)?)
        };
        let clut = if clut == 0 {
            None
        } else {
            let input = Curves::read(payload, a, &mut curve_extents[2], account)?;
            let grid = clut_grid(payload, clut, account)?;
            extents[CURVE_EXTENTS + 1] = Some(element(payload, clut, grid.end() - clut)?);
            Some(ClutStage { input, grid })
        };
        extents[..CURVE_EXTENTS].copy_from_slice(curve_extents.as_flattened());
        validate_extents(&extents, account)?;
        Ok(Self {
            clut,
            matrix,
            output,
        })
    }

    pub(super) fn evaluate(
        &self,
        mut values: [f64; CHANNELS],
        payload: &[u8],
        account: &Account,
    ) -> Result<[f64; CHANNELS]> {
        if let Some(stage) = &self.clut {
            let a = stage.input.evaluate(values, payload, account)?;
            values = stage.grid.evaluate(a, payload, account)?;
        }
        if let Some(stage) = &self.matrix {
            let m = stage.input.evaluate(values, payload, account)?;
            values = stage.apply(m, account)?;
        }
        self.output.evaluate(values, payload, account)
    }
}

fn stage_offsets(payload: &[u8], account: &Account) -> ProfileResult<[usize; 5]> {
    const HEADER: usize = 32;
    if account.coordinate(|| field::<4>(payload, 8))?? != [3, 3, 0, 0] {
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

struct Curves([Curve; CHANNELS]);

impl Curves {
    fn read(
        payload: &[u8],
        mut next: usize,
        extents: &mut [Option<Extent>; CHANNELS],
        account: &Account,
    ) -> ProfileResult<Self> {
        let mut curves = [Curve::Identity, Curve::Identity, Curve::Identity];
        for (curve, extent) in curves.iter_mut().zip(extents) {
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
        Ok(Self(curves))
    }

    fn evaluate(
        &self,
        mut values: [f64; CHANNELS],
        payload: &[u8],
        account: &Account,
    ) -> Result<[f64; CHANNELS]> {
        for (curve, value) in self.0.iter().zip(&mut values) {
            *value = curve.evaluate(CurveInput::Unit(*value), payload, account)?;
        }
        Ok(values)
    }
}

impl MatrixStage {
    fn read(input: Curves, payload: &[u8], start: usize, account: &Account) -> ProfileResult<Self> {
        let mut rows = [[0.0; CHANNELS]; CHANNELS];
        let mut offsets = [0.0; CHANNELS];
        // Table 45 encodes nine row-major coefficients, then three offsets.
        for (index, value) in rows.iter_mut().flatten().chain(&mut offsets).enumerate() {
            *value = account.coordinate(|| fixed(payload, start + index * 4))??;
        }
        Ok(Self {
            input,
            rows,
            offsets,
        })
    }

    fn apply(&self, input: [f64; CHANNELS], account: &Account) -> Result<[f64; CHANNELS]> {
        let mut output = self.offsets;
        for (value, row) in output.iter_mut().zip(&self.rows) {
            for (&coefficient, input) in row.iter().zip(input) {
                *value = account.coordinate(|| coefficient.mul_add(input, *value))?;
            }
            // The B curves receive the clipped affine result (§10.12.5).
            *value = account.coordinate(|| value.clamp(0.0, 1.0))?;
        }
        Ok(output)
    }
}

fn clut_grid(payload: &[u8], start: usize, account: &Account) -> ProfileResult<Grid> {
    let tail = payload.get(start..).ok_or(ProfileError::Invalid)?;
    let [r, g, b, unused @ ..] = account.coordinate(|| field::<16>(tail, 0))??;
    let precision = match account.coordinate(|| field::<4>(tail, 16))?? {
        [1, 0, 0, 0] => Precision::Eight,
        [2, 0, 0, 0] => Precision::Sixteen,
        _ => return Err(ProfileError::Invalid),
    };
    if unused != [0; 13] {
        return Err(ProfileError::Invalid);
    }
    // Reading the complete 20-byte CLUT header above proved this addition.
    Grid::read(start + 20, [r, g, b], precision, payload.len())
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

//! ICC normalized matrix/TRC and monochrome models from Annex F.

use std::mem::size_of;

mod inverse_matrix;
use inverse_matrix::InverseMatrix;

use super::native::IccPcsSpace;
use super::{
    Account, Curve, CurveInput, Direction, Extent, Header, Hold, ProfileError, ProfileResult,
    RenderResource, Result, Tag, find_tag, inverse::InverseCurve, pcs, read_xyz, sample_invariant,
};

pub(super) enum Model<'a> {
    Gray(Gray<'a>),
    Matrix(Matrix<'a>),
}

impl<'a> Model<'a> {
    pub(super) fn read(
        table: &[Tag],
        bytes: &'a [u8],
        header: &Header,
        space: IccPcsSpace,
        direction: Direction,
        account: &Account,
    ) -> ProfileResult<Self> {
        let version = header.version[0];
        if &header.data_space == b"GRAY"
            && matches!(&header.class, b"scnr" | b"mntr" | b"prtr")
            && (version == 4 || space == IccPcsSpace::XyzD50)
        {
            let tag = find_tag(table, b"kTRC", account)?.ok_or(ProfileError::Unsupported)?;
            let curve = Curve::read(tag, bytes, version, account)?;
            let pipeline = match direction {
                Direction::ToPcs => GrayPipeline::Forward(curve),
                Direction::FromPcs => {
                    GrayPipeline::Reverse(InverseCurve::prepare(&curve, bytes, account)?)
                }
            };
            return Ok(Self::Gray(Gray {
                space,
                bytes,
                pipeline,
            }));
        }
        let matrix_class = &header.class == b"scnr"
            || (&header.class == b"mntr" && (version == 4 || &header.data_space == b"RGB "));
        if space != IccPcsSpace::XyzD50
            || !matrix_class
            || super::device_channel_count(&header.data_space) != Some(3)
        {
            return Err(ProfileError::Unsupported);
        }
        let tags = MatrixTags::find(table, account)?.ok_or(ProfileError::Unsupported)?;
        let hold = account.acquire(RenderResource::ScratchCpuBytes, size_of::<MatrixPipeline>())?;
        let forward = MatrixForward::read(tags, bytes, version, account)?;
        let pipeline = match direction {
            Direction::ToPcs => MatrixPipeline::Forward(forward),
            Direction::FromPcs => MatrixPipeline::Reverse(forward.invert(bytes, account)?),
        };
        Ok(Self::Matrix(Matrix {
            bytes,
            pipeline: Box::new(pipeline),
            _hold: hold,
        }))
    }

    pub(super) fn evaluate(
        &self,
        input: &[f64],
        output: &mut [f64],
        account: &Account,
    ) -> Result<()> {
        match self {
            Self::Gray(gray) => gray.evaluate(input, output, account),
            Self::Matrix(matrix) => matrix.evaluate(input, output, account),
        }
    }
}

pub(super) struct Gray<'a> {
    space: IccPcsSpace,
    bytes: &'a [u8],
    pipeline: GrayPipeline,
}

enum GrayPipeline {
    Forward(Curve),
    Reverse(InverseCurve),
}

impl Gray<'_> {
    fn evaluate(&self, input: &[f64], output: &mut [f64], account: &Account) -> Result<()> {
        match &self.pipeline {
            GrayPipeline::Forward(curve) => {
                let &[value] = input else {
                    return Err(sample_invariant());
                };
                let destination =
                    <&mut [f64; 3]>::try_from(output).map_err(|_| sample_invariant())?;
                let input = CurveInput::Unit(account.coordinate(|| value.clamp(0.0, 1.0))?);
                let gray = curve.evaluate(input, self.bytes, account)?;
                let xyz = match self.space {
                    IccPcsSpace::XyzD50 => [
                        account.coordinate(|| gray * pcs::D50[0])?,
                        gray,
                        account.coordinate(|| gray * pcs::D50[2])?,
                    ],
                    IccPcsSpace::Lab => [account.coordinate(|| 100.0 * gray)?, 0.0, 0.0],
                };
                *destination = xyz;
            }
            GrayPipeline::Reverse(curve) => {
                let &[lightness, y, _] = input else {
                    return Err(sample_invariant());
                };
                let [destination] = output else {
                    return Err(sample_invariant());
                };
                let target = account.coordinate(|| match self.space {
                    IccPcsSpace::XyzD50 => y,
                    IccPcsSpace::Lab => lightness / 100.0,
                })?;
                *destination = curve.evaluate(target, self.bytes, account)?;
            }
        }
        Ok(())
    }
}

pub(super) struct Matrix<'a> {
    bytes: &'a [u8],
    pipeline: Box<MatrixPipeline>,
    _hold: Hold,
}

enum MatrixPipeline {
    Forward(MatrixForward),
    Reverse(MatrixInverse),
}

impl Matrix<'_> {
    fn evaluate(&self, input: &[f64], output: &mut [f64], account: &Account) -> Result<()> {
        let &[x, y, z] = input else {
            return Err(sample_invariant());
        };
        let destination = <&mut [f64; 3]>::try_from(output).map_err(|_| sample_invariant())?;
        *destination = match self.pipeline.as_ref() {
            MatrixPipeline::Forward(forward) => {
                let mut controls = [CurveInput::Unit(0.0); 3];
                for (control, value) in controls.iter_mut().zip([x, y, z]) {
                    *control = CurveInput::Unit(account.coordinate(|| value.clamp(0.0, 1.0))?);
                }
                forward.xyz_d50(controls, self.bytes, account)?
            }
            MatrixPipeline::Reverse(reverse) => reverse.device([x, y, z], self.bytes, account)?,
        };
        Ok(())
    }
}

pub(super) struct MatrixTags {
    columns: [Extent; 3],
    curves: [Extent; 3],
}

impl MatrixTags {
    pub(super) fn find(table: &[Tag], account: &Account) -> Result<Option<Self>> {
        let mut columns = [Extent { start: 0, end: 0 }; 3];
        let mut curves = columns;
        for (tag, name) in columns
            .iter_mut()
            .zip([b"rXYZ", b"gXYZ", b"bXYZ"])
            .chain(curves.iter_mut().zip([b"rTRC", b"gTRC", b"bTRC"]))
        {
            let Some(found) = find_tag(table, name, account)? else {
                return Ok(None);
            };
            *tag = found;
        }
        Ok(Some(Self { columns, curves }))
    }
}

pub(super) struct MatrixForward {
    columns: [[f64; 3]; 3],
    curves: [Curve; 3],
}

impl MatrixForward {
    pub(super) fn read(
        tags: MatrixTags,
        bytes: &[u8],
        version: u8,
        account: &Account,
    ) -> ProfileResult<Self> {
        let mut columns = [[0.0; 3]; 3];
        for (column, tag) in columns.iter_mut().zip(tags.columns) {
            *column = read_xyz(tag, bytes, account)?;
            if version == 2 {
                for &component in column.iter() {
                    if account.coordinate(|| component < 0.0)? {
                        return Err(ProfileError::Invalid);
                    }
                }
            }
        }
        let mut curves = [Curve::Identity, Curve::Identity, Curve::Identity];
        for (curve, tag) in curves.iter_mut().zip(tags.curves) {
            *curve = Curve::read(tag, bytes, version, account)?;
        }
        Ok(Self { columns, curves })
    }

    pub(super) fn uses_samples(&self) -> bool {
        self.curves
            .iter()
            .any(|curve| matches!(curve, Curve::Sampled { .. }))
    }

    pub(super) fn xyz_d50(
        &self,
        samples: [CurveInput; 3],
        bytes: &[u8],
        account: &Account,
    ) -> Result<[f64; 3]> {
        let mut xyz = [0.0; 3];
        for ((curve, column), sample) in self.curves.iter().zip(&self.columns).zip(samples) {
            let linear = curve.evaluate(sample, bytes, account)?;
            for (value, tint) in xyz.iter_mut().zip(column) {
                *value = account.coordinate(|| linear.mul_add(*tint, *value))?;
            }
        }
        Ok(xyz)
    }

    fn invert(&self, bytes: &[u8], account: &Account) -> ProfileResult<MatrixInverse> {
        let matrix =
            InverseMatrix::from_columns(self.columns, account)?.ok_or(ProfileError::Unsupported)?;
        let [r, g, b] = &self.curves;
        let curves = [
            InverseCurve::prepare(r, bytes, account)?,
            InverseCurve::prepare(g, bytes, account)?,
            InverseCurve::prepare(b, bytes, account)?,
        ];
        Ok(MatrixInverse { matrix, curves })
    }
}

struct MatrixInverse {
    matrix: InverseMatrix,
    curves: [InverseCurve; 3],
}

impl MatrixInverse {
    fn device(&self, xyz: [f64; 3], bytes: &[u8], account: &Account) -> Result<[f64; 3]> {
        let mut device = self.matrix.linear_controls(xyz, account)?;
        for (coordinate, curve) in device.iter_mut().zip(&self.curves) {
            *coordinate = curve.evaluate(*coordinate, bytes, account)?;
        }
        Ok(device)
    }
}

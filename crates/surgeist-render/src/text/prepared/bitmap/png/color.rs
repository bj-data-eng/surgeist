//! PNG source calibration and the native relative-colorimetric image policy.
//! PNG's fixed integer xy records bind this image's source; Color remains the
//! owner of predefined transfer/XYZ conversions and terminal sRGB quantization.
//!
//! Original implementation of PNG Third Edition (24 June 2025), sections 11.3.2
//! and 13.13–13.14: <https://www.w3.org/TR/2025/REC-png-3-20250624/>.
//! General-white Bradford follows ICC.1:2022 Annex E.3:
//! <https://www.color.org/specifications/ICC.1-2022-05.pdf>.
//! Missing-half sRGB defaults and relative-colorimetric terminal clipping are
//! native image policy; no individual-CSS-color gamut mapping is selected.

use super::*;
use crate::accounting::Hold;
use surgeist_color::{
    Color as IntrinsicColor, ColorSpace, Finite, PredefinedColorSpace as Space, Transformation,
};

type Matrix = [[f64; 3]; 3];
type IntegerMatrix = [[i128; 3]; 3];
const BRADFORD: IntegerMatrix = [[8951, 2664, -1614], [-7502, 17135, 367], [389, -685, 10296]];
const D65: [i128; 3] = [31270, 32900, 35830];

#[derive(Default)]
pub(super) struct Metadata {
    srgb: Option<u8>,
    gamma: Option<u32>,
    chromaticities: Option<[u32; 8]>,
}

impl Metadata {
    pub(super) fn read(
        &mut self,
        chunk: &Chunk<'_>,
        after_idat: bool,
        account: &Account,
    ) -> Result<()> {
        if after_idat {
            return Err(invalid());
        }
        match chunk.kind {
            b"sRGB" => {
                if self.srgb.is_some() || chunk.data.len() != 1 {
                    return Err(invalid());
                }
                self.srgb = Some(account.coordinate(|| chunk.data[0])?);
            }
            b"gAMA" => {
                if self.gamma.is_some() || chunk.data.len() != 4 {
                    return Err(invalid());
                }
                self.gamma = Some(field(chunk.data, account)?);
            }
            b"cHRM" => {
                if self.chromaticities.is_some() || chunk.data.len() != 32 {
                    return Err(invalid());
                }
                let mut values = [0; 8];
                for (value, bytes) in values.iter_mut().zip(chunk.data.as_chunks::<4>().0) {
                    *value = field(bytes, account)?;
                }
                self.chromaticities = Some(values);
            }
            _ => return Err(GlyphPreparationError::UnsupportedPng.into()),
        }
        Ok(())
    }
}

fn field(bytes: &[u8], account: &Account) -> Result<u32> {
    let value = account.coordinate(|| number(bytes))??;
    if account.coordinate(|| value > 0x7fff_ffff)? {
        return Err(invalid());
    }
    Ok(value)
}

#[derive(Clone, Copy)]
enum Transfer {
    Srgb,
    Power(f64),
}
enum Calibration {
    Srgb,
    Xyz { matrix: Matrix, white: Space },
}
struct ConversionWorkspace {
    transfer: Transfer,
    calibration: Calibration,
    linear: [f64; 3],
    xyz: [f64; 3],
}
struct Calibrated {
    workspace: ConversionWorkspace,
    // Numeric arrays are disposed before their admitted custody.
    _hold: Hold,
}
pub(super) struct Interpretation(Option<Calibrated>);

impl Interpretation {
    pub(super) fn compile(raw: Metadata, account: &Account) -> Result<Self> {
        if let Some(intent) = raw.srgb {
            // Both valid and semantically invalid selected sRGB identify the
            // complete sRGB fallback here. Lower-priority calibration is unused.
            if !account.coordinate(|| intent <= 3)? {
                return Ok(Self(None));
            }
            return Ok(Self(None));
        }
        let gamma = match raw.gamma {
            Some(value) if account.coordinate(|| value != 0)? => Some(value),
            _ => None,
        };
        if gamma.is_none() && raw.chromaticities.is_none() {
            return Ok(Self(None));
        }
        // Admit destination workspace before any compiled arrays can escape
        // calibration. Its overlap with the compile workspace is real custody.
        let hold = account.acquire(
            RenderResource::ScratchCpuBytes,
            size_of::<ConversionWorkspace>(),
        )?;
        let mut workspace = ConversionWorkspace {
            transfer: Transfer::Srgb,
            calibration: Calibration::Srgb,
            linear: [0.0; 3],
            xyz: [0.0; 3],
        };
        workspace.calibration = match raw.chromaticities {
            None => Calibration::Srgb,
            Some(values) => match calibrate(values, account)? {
                Some(calibration) => calibration,
                None => return Ok(Self(None)),
            },
        };
        workspace.transfer = match gamma {
            None => Transfer::Srgb,
            Some(value) => {
                let divisor = account.coordinate(|| f64::from(value))?;
                Transfer::Power(evaluate(account, || 100000.0 / divisor)?)
            }
        };
        Ok(Self(Some(Calibrated {
            workspace,
            _hold: hold,
        })))
    }

    pub(super) fn convert(&mut self, pixels: &mut [u8], account: &Account) -> Result<()> {
        let Some(calibrated) = &mut self.0 else {
            return Ok(());
        };
        account.phase(RenderPhase::PngColorConvert, || {
            let account = account.at(RenderPhase::PngColorConvert);
            let workspace = &mut calibrated.workspace;
            for pixel in pixels.as_chunks_mut::<4>().0 {
                account.work(RenderWork::ImageTexelVisits, 1)?;
                match workspace.transfer {
                    Transfer::Srgb => {
                        let red = account.coordinate(|| pixel[0])?;
                        let green = account.coordinate(|| pixel[1])?;
                        let blue = account.coordinate(|| pixel[2])?;
                        let source = IntrinsicColor::from_srgba8([red, green, blue, 255]);
                        workspace.linear =
                            channels(convert(source, Space::SrgbLinear, &account)?, &account)?;
                    }
                    Transfer::Power(exponent) => {
                        for (index, linear) in workspace.linear.iter_mut().enumerate() {
                            let byte = account.coordinate(|| pixel[index])?;
                            let value = account.coordinate(|| f64::from(byte))?;
                            let sample = evaluate(&account, || value / 255.0)?;
                            *linear = evaluate(&account, || sample.powf(exponent))?;
                        }
                    }
                }
                let xyz = match &workspace.calibration {
                    Calibration::Srgb => convert(
                        complete(Space::SrgbLinear, workspace.linear, &account)?,
                        Space::XyzD65,
                        &account,
                    )?,
                    Calibration::Xyz { matrix, white } => {
                        for (row, result) in matrix.iter().zip(&mut workspace.xyz) {
                            *result = dot(row, &workspace.linear, &account)?;
                        }
                        complete(*white, workspace.xyz, &account)?
                    }
                };
                channels(xyz, &account)?;
                let srgb = convert(xyz, Space::Srgb, &account)?;
                channels(srgb, &account)?;
                account.work(RenderWork::CoordinateVisits, 3)?;
                let quantized = srgb.transformed(Transformation::QuantizeSrgb);
                let bytes = quantized
                    .srgb_bytes()
                    .ok_or(GlyphPreparationError::UnsupportedPng)?;
                for index in 0..3 {
                    pixel[index] = account.coordinate(|| bytes[index])?;
                }
                // Alpha remains its original straight linear byte, even at zero.
            }
            Ok(())
        })
    }
}

fn numeric(value: f64) -> Error {
    GlyphPreparationError::Numeric {
        phase: crate::GlyphNumericPhase::BitmapColor,
        value,
    }
    .into()
}
fn finite(value: f64, account: &Account) -> Result<f64> {
    if !account.coordinate(|| value.is_finite())? {
        return Err(numeric(value));
    }
    Ok(value)
}
fn evaluate(account: &Account, operation: impl FnOnce() -> f64) -> Result<f64> {
    finite(account.coordinate(operation)?, account)
}
fn convert(color: IntrinsicColor, destination: Space, account: &Account) -> Result<IntrinsicColor> {
    account.work(RenderWork::CoordinateVisits, 3)?;
    Ok(color.converted(ColorSpace::Predefined(destination)))
}
fn channels(color: IntrinsicColor, account: &Account) -> Result<[f64; 3]> {
    let values = color.channels();
    let mut complete = [0.0; 3];
    for (value, result) in values.into_iter().zip(&mut complete) {
        let value = account
            .coordinate(|| value)?
            .ok_or(GlyphPreparationError::UnsupportedPng)?;
        *result = finite(value, account)?;
    }
    Ok(complete)
}
fn complete(space: Space, values: [f64; 3], account: &Account) -> Result<IntrinsicColor> {
    let mut channels = [None; 3];
    for (value, result) in values.into_iter().zip(&mut channels) {
        *result = Some(
            account
                .coordinate(|| Finite::new(value))?
                .ok_or_else(|| numeric(value))?,
        );
    }
    Ok(IntrinsicColor::new(
        ColorSpace::Predefined(space),
        channels,
        None,
    ))
}

struct Workspace {
    primaries: IntegerMatrix,
    cramer: IntegerMatrix,
    white: [i128; 3],
    numerators: [i128; 3],
    source: Matrix,
    cones: [i128; 3],
    destination_cones: [i128; 3],
    ratios: [f64; 3],
    basis: Matrix,
    inverse: Matrix,
    scaled_inverse: Matrix,
    adaptation: Matrix,
    combined: Matrix,
}

fn calibrate(values: [u32; 8], account: &Account) -> Result<Option<Calibration>> {
    for pair in values.as_chunks::<2>().0 {
        let x = account.coordinate(|| i128::from(pair[0]))?;
        let y = account.coordinate(|| i128::from(pair[1]))?;
        let sum = account.coordinate(|| x + y)?;
        if account.coordinate(|| sum > 100000)? {
            return Ok(None);
        }
    }
    if account.coordinate(|| values[1] == 0)? {
        return Ok(None);
    }
    let _hold = account.acquire(RenderResource::ScratchCpuBytes, size_of::<Workspace>())?;
    let mut workspace = Workspace {
        primaries: [[0; 3]; 3],
        cramer: [[0; 3]; 3],
        white: [0; 3],
        numerators: [0; 3],
        source: [[0.0; 3]; 3],
        cones: [0; 3],
        destination_cones: [0; 3],
        ratios: [0.0; 3],
        basis: [[0.0; 3]; 3],
        inverse: [[0.0; 3]; 3],
        scaled_inverse: [[0.0; 3]; 3],
        adaptation: [[0.0; 3]; 3],
        combined: [[0.0; 3]; 3],
    };
    workspace.white = homogeneous(values[0], values[1], account)?;
    for column in 0..3 {
        let primary = homogeneous(values[2 + column * 2], values[3 + column * 2], account)?;
        for (row, value) in primary.into_iter().enumerate() {
            workspace.primaries[row][column] = value;
        }
    }
    // Each admitted component is at most 100000: determinants fit 6*10^15;
    // subsequent component products fit 6*10^20, well inside signed i128.
    let primary_determinant = determinant(&workspace.primaries, account)?;
    if account.coordinate(|| primary_determinant == 0)? {
        return Ok(None);
    }
    for column in 0..3 {
        workspace.cramer = workspace.primaries;
        for row in 0..3 {
            workspace.cramer[row][column] = workspace.white[row];
        }
        let numerator = determinant(&workspace.cramer, account)?;
        workspace.numerators[column] = numerator;
        if account.coordinate(|| numerator != 0)? {
            let negative = account.coordinate(|| numerator < 0)?;
            let determinant_negative = account.coordinate(|| primary_determinant < 0)?;
            if account.coordinate(|| negative != determinant_negative)? {
                return Ok(None);
            }
        }
    }
    let denominator = account.coordinate(|| primary_determinant * workspace.white[1])?;
    let denominator = account.coordinate(|| denominator as f64)?;
    for row in 0..3 {
        for column in 0..3 {
            let numerator = account
                .coordinate(|| workspace.primaries[row][column] * workspace.numerators[column])?;
            let numerator = account.coordinate(|| numerator as f64)?;
            workspace.source[row][column] = evaluate(account, || numerator / denominator)?;
        }
    }
    if account.coordinate(|| values[0] == 31270)? && account.coordinate(|| values[1] == 32900)? {
        return Ok(Some(Calibration::Xyz {
            matrix: workspace.source,
            white: Space::XyzD65,
        }));
    }
    if account.coordinate(|| values[0] == 34570)? && account.coordinate(|| values[1] == 35850)? {
        // Color owns this exact standard adaptation; never apply it twice.
        return Ok(Some(Calibration::Xyz {
            matrix: workspace.source,
            white: Space::XyzD50,
        }));
    }
    for (row, basis_row) in BRADFORD.iter().enumerate() {
        workspace.cones[row] = integer_dot(basis_row, &workspace.white, account)?;
        if account.coordinate(|| workspace.cones[row] == 0)? {
            return Err(GlyphPreparationError::UnsupportedPng.into());
        }
        workspace.destination_cones[row] = integer_dot(basis_row, &D65, account)?;
        let source_denominator = account.coordinate(|| 10000 * workspace.white[1])?;
        let source_numerator = account.coordinate(|| workspace.cones[row] as f64)?;
        let source_denominator = account.coordinate(|| source_denominator as f64)?;
        let source = evaluate(account, || source_numerator / source_denominator)?;
        let destination_numerator =
            account.coordinate(|| workspace.destination_cones[row] as f64)?;
        let destination_denominator = account.coordinate(|| 10000 * D65[1])?;
        let destination_denominator = account.coordinate(|| destination_denominator as f64)?;
        let destination = evaluate(account, || destination_numerator / destination_denominator)?;
        workspace.ratios[row] = evaluate(account, || destination / source)?;
    }
    let basis_determinant = determinant(&BRADFORD, account)?;
    let basis_denominator = account.coordinate(|| basis_determinant as f64)?;
    for row in 0..3 {
        for column in 0..3 {
            let coefficient = account.coordinate(|| BRADFORD[row][column] as f64)?;
            workspace.basis[row][column] = evaluate(account, || coefficient / 10000.0)?;
            let a = account.coordinate(|| {
                BRADFORD[(column + 1) % 3][(row + 1) % 3]
                    * BRADFORD[(column + 2) % 3][(row + 2) % 3]
            })?;
            let b = account.coordinate(|| {
                BRADFORD[(column + 1) % 3][(row + 2) % 3]
                    * BRADFORD[(column + 2) % 3][(row + 1) % 3]
            })?;
            let minor = account.coordinate(|| a - b)?;
            let numerator = account.coordinate(|| minor * 10000)?;
            let numerator = account.coordinate(|| numerator as f64)?;
            workspace.inverse[row][column] = evaluate(account, || numerator / basis_denominator)?;
            workspace.scaled_inverse[row][column] = evaluate(account, || {
                workspace.inverse[row][column] * workspace.ratios[column]
            })?;
        }
    }
    multiply(
        &workspace.scaled_inverse,
        &workspace.basis,
        &mut workspace.adaptation,
        account,
    )?;
    multiply(
        &workspace.adaptation,
        &workspace.source,
        &mut workspace.combined,
        account,
    )?;
    Ok(Some(Calibration::Xyz {
        matrix: workspace.combined,
        white: Space::XyzD65,
    }))
}

fn homogeneous(x: u32, y: u32, account: &Account) -> Result<[i128; 3]> {
    let x = account.coordinate(|| i128::from(x))?;
    let y = account.coordinate(|| i128::from(y))?;
    let remaining = account.coordinate(|| 100000 - x)?;
    let z = account.coordinate(|| remaining - y)?;
    Ok([x, y, z])
}
fn integer_dot(a: &[i128; 3], b: &[i128; 3], account: &Account) -> Result<i128> {
    let mut sum = 0;
    for index in 0..3 {
        let product = account.coordinate(|| a[index] * b[index])?;
        sum = account.coordinate(|| sum + product)?;
    }
    Ok(sum)
}
fn determinant(matrix: &IntegerMatrix, account: &Account) -> Result<i128> {
    let mut sum = 0;
    for (a, b, c, positive) in [
        (0, 1, 2, true),
        (1, 2, 0, true),
        (2, 0, 1, true),
        (2, 1, 0, false),
        (1, 0, 2, false),
        (0, 2, 1, false),
    ] {
        let product = account.coordinate(|| matrix[0][a] * matrix[1][b])?;
        let product = account.coordinate(|| product * matrix[2][c])?;
        sum = if positive {
            account.coordinate(|| sum + product)?
        } else {
            account.coordinate(|| sum - product)?
        };
    }
    Ok(sum)
}
fn dot(a: &[f64; 3], b: &[f64; 3], account: &Account) -> Result<f64> {
    let mut sum = 0.0;
    for index in 0..3 {
        let product = evaluate(account, || a[index] * b[index])?;
        sum = evaluate(account, || sum + product)?;
    }
    Ok(sum)
}
fn multiply(a: &Matrix, b: &Matrix, out: &mut Matrix, account: &Account) -> Result<()> {
    for row in 0..3 {
        for column in 0..3 {
            let mut sum = 0.0;
            for index in 0..3 {
                let product = evaluate(account, || a[row][index] * b[index][column])?;
                sum = evaluate(account, || sum + product)?;
            }
            out[row][column] = sum;
        }
    }
    Ok(())
}

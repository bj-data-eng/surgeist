//! ICC v4 forward floating-point multi-process elements for RGB source profiles.
//! ICC.1:2022 §§4.3, 6.3.4, 8.10.2 and 10.16.

use std::mem::size_of;

use crate::error::BackendErrorCode;
use crate::text::{GlyphNumericPhase, GlyphPreparationError};

use super::{
    Account, AccountedVec, Error, Extent, Hold, PcsColor, ProfileError, ProfileResult,
    RenderResource, RenderWork, Result, field, tag_kind, u16_at, u32_at,
};

mod curves;
mod linear;
mod positions;

use curves::CurveSet;
use linear::{Grid, Matrix};
use positions::Positions;

const RGB_CHANNELS: usize = 3;

pub(super) enum Attempt {
    Ready(RgbMpeProfile),
    Fallback(AccountedVec<u8>),
}

/// A validated pipeline and the backing and scratch custody needed to execute it.
pub(super) struct RgbMpeProfile {
    data: Box<Prepared>,
    _hold: Hold,
}

struct Prepared {
    source: AccountedVec<u8>,
    tag: Extent,
    plan: Plan,
    front: AccountedVec<f64>,
    back: AccountedVec<f64>,
    pcs: Pcs,
}

enum Pcs {
    XyzD50,
    Lab,
}

pub(super) fn prepare(
    source: AccountedVec<u8>,
    tag: Extent,
    pcs: &[u8; 4],
    account: &Account,
) -> ProfileResult<Attempt> {
    let pcs = match pcs {
        b"XYZ " => Pcs::XyzD50,
        b"Lab " => Pcs::Lab,
        _ => return Err(ProfileError::Invalid),
    };
    let plan = match Plan::read(tag.payload(&source)?, account) {
        Ok(plan) => plan,
        // An unsupported element disables the whole DToB route (§10.16.1).
        Err(ProfileError::Unsupported) => return Ok(Attempt::Fallback(source)),
        Err(error) => return Err(error),
    };
    let hold = account.acquire(RenderResource::ScratchCpuBytes, size_of::<Prepared>())?;
    let mut front = account.temporary::<f64>(plan.width)?;
    front.resize(plan.width, 0.0)?;
    let mut back = account.temporary::<f64>(plan.width)?;
    back.resize(plan.width, 0.0)?;
    let data = Box::new(Prepared {
        source,
        tag,
        plan,
        front,
        back,
        pcs,
    });
    Ok(Attempt::Ready(RgbMpeProfile { data, _hold: hold }))
}

impl RgbMpeProfile {
    pub(super) fn pcs(&mut self, rgb: [u8; RGB_CHANNELS], account: &Account) -> Result<PcsColor> {
        let Prepared {
            source,
            tag,
            plan,
            front,
            back,
            pcs,
        } = self.data.as_mut();
        let payload = tag.payload(source).map_err(|_| invariant())?;
        let mut input: &mut [f64] = front;
        let mut output: &mut [f64] = back;
        let initial = input.get_mut(..RGB_CHANNELS).ok_or_else(invariant)?;
        for (value, sample) in initial.iter_mut().zip(rgb) {
            *value = account.coordinate(|| f64::from(sample) / 255.0)?;
        }

        for &index in plan.order.iter() {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let element = plan.elements.get(index).ok_or_else(invariant)?;
            if matches!(element.operation, Operation::PassThrough) {
                continue;
            }
            let from = input.get(..element.input).ok_or_else(invariant)?;
            let to = output.get_mut(..element.output).ok_or_else(invariant)?;
            let bytes = element.extent.payload(payload).map_err(|_| invariant())?;
            match &element.operation {
                Operation::Curves(curves) => curves.evaluate(bytes, from, to, account)?,
                Operation::Matrix(matrix) => matrix.evaluate(bytes, from, to, account)?,
                Operation::Grid(grid) => grid.evaluate(bytes, from, to, account)?,
                Operation::PassThrough | Operation::Unavailable => return Err(invariant()),
            }
            std::mem::swap(&mut input, &mut output);
        }

        let values = input.first_chunk::<RGB_CHANNELS>().ok_or_else(invariant)?;
        // DToB stores actual PCS values, with no integer PCS scaling.
        Ok(match pcs {
            Pcs::XyzD50 => PcsColor::XyzD50(*values),
            Pcs::Lab => PcsColor::Lab(*values),
        })
    }
}

struct Plan {
    order: AccountedVec<usize>,
    elements: AccountedVec<Element>,
    width: usize,
}

impl Plan {
    fn read(payload: &[u8], account: &Account) -> ProfileResult<Self> {
        if &tag_kind(payload, account)? != b"mpet"
            || account.coordinate(|| field::<4>(payload, 8))?? != [0, 3, 0, 3]
        {
            return Err(ProfileError::Invalid);
        }
        let count = account.coordinate(|| u32_at(payload, 12))??;
        let count = usize::try_from(count).map_err(|_| ProfileError::Invalid)?;
        let positions = Positions::read(payload, 16, count, 12, account)?;
        let mut elements = account.temporary(positions.unique_count())?;
        let mut order = account.temporary(count)?;
        order.resize(count, 0)?;

        let mut previous = None;
        let mut unsupported = false;
        for record in positions.physical() {
            account.work(RenderWork::IccProfileTagVisits, 1)?;
            if previous != Some(record.extent) {
                let element = Element::read(record.extent, payload, account)?;
                unsupported |= matches!(element.operation, Operation::Unavailable);
                elements.append_slot()?.write(element);
                previous = Some(record.extent);
            }
            let index = elements.len().checked_sub(1).ok_or(ProfileError::Invalid)?;
            *order.get_mut(record.index).ok_or(ProfileError::Invalid)? = index;
        }

        // The directory order is semantic. Physical order exists only for
        // bounded validation and to share a compiled stage across exact aliases.
        let mut required = RGB_CHANNELS;
        let mut width = required;
        for &index in order.iter() {
            account.work(RenderWork::IccProfileTagVisits, 1)?;
            let element = elements.get(index).ok_or(ProfileError::Invalid)?;
            if element.input != required {
                return Err(ProfileError::Invalid);
            }
            required = element.output;
            width = width.max(required);
        }
        if required != RGB_CHANNELS {
            return Err(ProfileError::Invalid);
        }
        if unsupported {
            return Err(ProfileError::Unsupported);
        }
        Ok(Self {
            order,
            elements,
            width,
        })
    }
}

struct Element {
    extent: Extent,
    input: usize,
    output: usize,
    operation: Operation,
}

enum Operation {
    Curves(CurveSet),
    Matrix(Matrix),
    Grid(Grid),
    PassThrough,
    Unavailable,
}

impl Element {
    fn read(extent: Extent, payload: &[u8], account: &Account) -> ProfileResult<Self> {
        let bytes = extent.payload(payload)?;
        let kind = tag_kind(bytes, account)?;
        let input = usize::from(account.coordinate(|| u16_at(bytes, 8))??);
        let output = usize::from(account.coordinate(|| u16_at(bytes, 10))??);
        if input == 0 || output == 0 {
            return Err(ProfileError::Invalid);
        }
        let operation = match &kind {
            b"cvst" if input == output => match CurveSet::read(bytes, input, account) {
                Ok(curves) => Operation::Curves(curves),
                Err(ProfileError::Unsupported) => Operation::Unavailable,
                Err(error) => return Err(error),
            },
            b"matf" => Operation::Matrix(Matrix::read(bytes, input, output, account)?),
            b"clut" => Operation::Grid(Grid::read(bytes, input, output, account)?),
            b"bACS" | b"eACS" if input == output && bytes.len() == 16 => Operation::PassThrough,
            b"cvst" | b"bACS" | b"eACS" => return Err(ProfileError::Invalid),
            _ => Operation::Unavailable,
        };
        Ok(Self {
            extent,
            input,
            output,
            operation,
        })
    }
}

fn finite_encoding(value: f32) -> ProfileResult<f64> {
    if !value.is_finite() || value.is_subnormal() {
        return Err(ProfileError::Invalid);
    }
    Ok(f64::from(value))
}

fn float_at(bytes: &[u8], offset: usize, account: &Account) -> ProfileResult<f64> {
    account.coordinate(|| finite_encoding(f32::from_bits(u32_at(bytes, offset)?)))?
}

fn float_values(bytes: &[u8], account: &Account) -> ProfileResult<()> {
    let (floats, remainder) = bytes.as_chunks::<4>();
    if !remainder.is_empty() {
        return Err(ProfileError::Invalid);
    }
    for bits in floats {
        account.coordinate(|| finite_encoding(f32::from_be_bytes(*bits)))??;
    }
    Ok(())
}

fn sample_at(bytes: &[u8], offset: usize, account: &Account) -> Result<f64> {
    match float_at(bytes, offset, account) {
        Ok(value) => Ok(value),
        Err(ProfileError::Render(error)) => Err(error),
        Err(_) => Err(invariant()),
    }
}

fn checked_value(value: f64) -> Result<f64> {
    if value.abs() <= f64::from(f32::MAX) {
        return Ok(value);
    }
    // Keep computed values wide between stages. Arithmetic that escapes the
    // representable float range is a numeric refusal rather than clipped ink.
    Err(GlyphPreparationError::Numeric {
        phase: GlyphNumericPhase::BitmapColor,
        value,
    }
    .into())
}

fn invariant() -> Error {
    Error::new(
        BackendErrorCode::RenderFailed,
        "validated ICC multi-process data is outside its backing",
    )
}

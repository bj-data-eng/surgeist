//! Segmented curve sets specified by ICC.1:2022 §10.16.2.2.

use std::mem::size_of;

use super::{Positions, checked_value, float_at, float_values, invariant, sample_at};
use crate::color_profile::{
    Account, AccountedVec, Extent, Hold, ProfileError, ProfileResult, RenderResource, RenderWork,
    Result, field, tag_kind, u32_at,
};

const HEADER: usize = 12;

pub(super) struct CurveSet {
    data: Box<CurveData>,
    _hold: Hold,
}

struct CurveData {
    order: AccountedVec<usize>,
    curves: AccountedVec<Curve>,
}

impl CurveSet {
    pub(super) fn read(payload: &[u8], channels: usize, account: &Account) -> ProfileResult<Self> {
        let positions = Positions::read(payload, HEADER, channels, HEADER, account)?;
        let mut order = account.temporary(channels)?;
        order.resize(channels, 0)?;
        let mut curves = account.temporary(positions.unique_count())?;
        let mut previous = None;
        let mut curve_index = 0;
        let mut unsupported = false;
        for record in positions.physical() {
            account.work(RenderWork::IccProfileTagVisits, 1)?;
            if previous != Some(record.extent) {
                previous = Some(record.extent);
                curve_index = curves.len();
                match Curve::read(record.extent, payload, account) {
                    Ok(curve) => curves.append_slot()?.write(curve),
                    Err(ProfileError::Unsupported) => unsupported = true,
                    Err(error) => return Err(error),
                }
            }
            *order.get_mut(record.index).ok_or(ProfileError::Invalid)? = curve_index;
        }
        if unsupported {
            return Err(ProfileError::Unsupported);
        }
        let hold = account.acquire(RenderResource::ScratchCpuBytes, size_of::<CurveData>())?;
        let data = Box::new(CurveData { order, curves });
        Ok(Self { data, _hold: hold })
    }

    pub(super) fn evaluate(
        &self,
        payload: &[u8],
        input: &[f64],
        output: &mut [f64],
        account: &Account,
    ) -> Result<()> {
        if input.len() != self.data.order.len() || output.len() != self.data.order.len() {
            return Err(invariant());
        }
        for ((&curve, &input), output) in self.data.order.iter().zip(input).zip(output) {
            *output = self
                .data
                .curves
                .get(curve)
                .ok_or_else(invariant)?
                .evaluate(payload, input, account)?;
        }
        Ok(())
    }
}

struct Curve {
    extent: Extent,
    segments: AccountedVec<Segment>,
}

impl Curve {
    fn read(extent: Extent, payload: &[u8], account: &Account) -> ProfileResult<Self> {
        let bytes = extent.payload(payload)?;
        if tag_kind(bytes, account)? != *b"curf" {
            return Err(ProfileError::Invalid);
        }
        let [high, low, 0, 0] = account.coordinate(|| field(bytes, 8))?? else {
            return Err(ProfileError::Invalid);
        };
        let count = usize::from(u16::from_be_bytes([high, low]));
        let breakpoints = count.checked_sub(1).ok_or(ProfileError::Invalid)?;
        let mut next = HEADER + breakpoints * 4;
        // Each segment has at least a signature, reserved bytes, and a count or subtype.
        if bytes.len() < next + count * HEADER {
            return Err(ProfileError::Invalid);
        }
        let mut previous = f64::NEG_INFINITY;
        for index in 0..breakpoints {
            let value = float_at(bytes, HEADER + index * 4, account)?;
            if value < previous {
                return Err(ProfileError::Invalid);
            }
            previous = value;
        }

        let mut segments = account.temporary(count)?;
        for index in 0..count {
            let formula_required = index == 0 || index + 1 == count;
            let (segment, end) = Segment::read(bytes, next, formula_required, account)?;
            if bytes.len() - end < (count - index - 1) * HEADER {
                return Err(ProfileError::Invalid);
            }
            segments.append_slot()?.write(segment);
            next = end;
        }
        if next != bytes.len() {
            return Err(ProfileError::Invalid);
        }
        Ok(Self { extent, segments })
    }

    fn evaluate(&self, payload: &[u8], input: f64, account: &Account) -> Result<f64> {
        let bytes = self.extent.payload(payload).map_err(|_| invariant())?;
        let input = checked_value(input)?;
        let mut first = 0_usize;
        let mut last = self.segments.len() - 1;
        // Find the first segment whose upper bound includes the input. Equal
        // breakpoints leave empty intervals that are never directly selected.
        while first < last {
            let middle = first.midpoint(last);
            if input <= sample_at(bytes, HEADER + middle * 4, account)? {
                last = middle;
            } else {
                first = middle + 1;
            }
        }

        match &self.segments[first] {
            Segment::Formula(formula) => formula.evaluate(input, account),
            Segment::Sampled(samples) => self.interpolate(samples, first, input, bytes, account),
        }
    }

    fn interpolate(
        &self,
        samples: &Samples,
        segment: usize,
        input: f64,
        bytes: &[u8],
        account: &Account,
    ) -> Result<f64> {
        let previous = segment - 1; // Both outer segments are formulas.
        let lower = sample_at(bytes, HEADER + previous * 4, account)?;
        let upper = sample_at(bytes, HEADER + segment * 4, account)?;
        let position =
            account.coordinate(|| (input - lower) / (upper - lower) * samples.count as f64)?;
        let index = position as usize;
        let fraction = position - index as f64;
        let left = if index == 0 {
            // The implicit sample comes from the immediately preceding segment,
            // even if equal breakpoints gave that segment an empty interval.
            self.segments[previous].upper_value(bytes, lower, account)?
        } else {
            samples.at(bytes, index - 1, account)?
        };
        if fraction == 0.0 {
            return Ok(left);
        }
        let right = samples.at(bytes, index, account)?;
        checked_value(account.coordinate(|| (right - left).mul_add(fraction, left))?)
    }
}

enum Segment {
    Formula(Formula),
    Sampled(Samples),
}

impl Segment {
    fn read(
        bytes: &[u8],
        start: usize,
        formula_required: bool,
        account: &Account,
    ) -> ProfileResult<(Self, usize)> {
        let tail = bytes.get(start..).ok_or(ProfileError::Invalid)?;
        match &tag_kind(tail, account)? {
            b"parf" => {
                let (formula, len) = Formula::read(tail, account)?;
                Ok((Self::Formula(formula), start + len))
            }
            b"samf" if !formula_required => {
                let count = account.coordinate(|| u32_at(tail, 8))??;
                let count = usize::try_from(count).map_err(|_| ProfileError::Invalid)?;
                if count == 0 {
                    return Err(ProfileError::Invalid);
                }
                let start = start + HEADER;
                let end = count
                    .checked_mul(4)
                    .and_then(|len| start.checked_add(len))
                    .ok_or(ProfileError::Invalid)?;
                let samples = bytes.get(start..end).ok_or(ProfileError::Invalid)?;
                float_values(samples, account)?;
                Ok((Self::Sampled(Samples { start, count }), end))
            }
            _ => Err(ProfileError::Invalid),
        }
    }

    fn upper_value(&self, bytes: &[u8], breakpoint: f64, account: &Account) -> Result<f64> {
        match self {
            Self::Formula(formula) => formula.evaluate(breakpoint, account),
            Self::Sampled(samples) => samples.at(bytes, samples.count - 1, account),
        }
    }
}

struct Samples {
    start: usize,
    count: usize,
}

impl Samples {
    fn at(&self, bytes: &[u8], index: usize, account: &Account) -> Result<f64> {
        if index >= self.count {
            return Err(invariant());
        }
        sample_at(bytes, self.start + index * 4, account)
    }
}

enum Formula {
    Power([f64; 4]),
    Logarithm([f64; 5]),
    Exponential([f64; 5]),
}

impl Formula {
    fn read(bytes: &[u8], account: &Account) -> ProfileResult<(Self, usize)> {
        match account.coordinate(|| field::<4>(bytes, 8))?? {
            [0, 0, 0, 0] => Ok((Self::Power(parameters(bytes, account)?), HEADER + 16)),
            [0, 1, 0, 0] => Ok((Self::Logarithm(parameters(bytes, account)?), HEADER + 20)),
            [0, 2, 0, 0] => Ok((Self::Exponential(parameters(bytes, account)?), HEADER + 20)),
            [_, _, 0, 0] => Err(ProfileError::Unsupported),
            _ => Err(ProfileError::Invalid),
        }
    }

    fn evaluate(&self, input: f64, account: &Account) -> Result<f64> {
        checked_value(account.coordinate(|| match *self {
            Self::Power([gamma, a, b, c]) => a.mul_add(input, b).powf(gamma) + c,
            Self::Logarithm([gamma, a, b, c, d]) => {
                a.mul_add(b.mul_add(input.powf(gamma), c).log10(), d)
            }
            Self::Exponential([a, b, c, d, e]) => a.mul_add(b.powf(c.mul_add(input, d)), e),
        })?)
    }
}

fn parameters<const N: usize>(bytes: &[u8], account: &Account) -> ProfileResult<[f64; N]> {
    let mut output = [0.0; N];
    for (index, parameter) in output.iter_mut().enumerate() {
        *parameter = float_at(bytes, HEADER + index * 4, account)?;
    }
    Ok(output)
}

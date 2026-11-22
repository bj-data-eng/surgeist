//! Packed float matrix and CLUT elements (ICC.1:2022 Tables 62–63).

use std::mem::size_of;

use super::{
    Account, ProfileError, ProfileResult, RenderResource, Result, checked_value, field,
    float_values, invariant, sample_at,
};

const ELEMENT_HEADER: usize = 12;
const GRID_AXES: usize = 16;
const GRID_HEADER: usize = ELEMENT_HEADER + GRID_AXES;
const FLOAT_BYTES: usize = size_of::<f32>();

struct Channels {
    input: u16,
    output: u16,
}

impl Channels {
    fn new(input: usize, output: usize) -> ProfileResult<Self> {
        if input == 0 || output == 0 {
            return Err(ProfileError::Invalid);
        }
        Ok(Self {
            input: u16::try_from(input).map_err(|_| ProfileError::Invalid)?,
            output: u16::try_from(output).map_err(|_| ProfileError::Invalid)?,
        })
    }

    fn check(&self, input: &[f64], output: &[f64]) -> Result<()> {
        if input.len() != usize::from(self.input) || output.len() != usize::from(self.output) {
            return Err(invariant());
        }
        Ok(())
    }
}

pub(super) struct Matrix(Channels);

impl Matrix {
    pub(super) fn read(
        payload: &[u8],
        input: usize,
        output: usize,
        account: &Account,
    ) -> ProfileResult<Self> {
        let channels = Channels::new(input, output)?;
        let length = (input + 1)
            .checked_mul(output)
            .and_then(|count| count.checked_mul(FLOAT_BYTES))
            .ok_or(ProfileError::Invalid)?;
        let samples = payload
            .get(ELEMENT_HEADER..)
            .filter(|samples| samples.len() == length)
            .ok_or(ProfileError::Invalid)?;
        float_values(samples, account)?;
        Ok(Self(channels))
    }

    pub(super) fn evaluate(
        &self,
        payload: &[u8],
        input: &[f64],
        output: &mut [f64],
        account: &Account,
    ) -> Result<()> {
        self.0.check(input, output)?;
        let samples = payload.get(ELEMENT_HEADER..).ok_or_else(invariant)?;
        // The validated extent bounds the packed rows and trailing offsets.
        let mut coefficient_offset = 0;
        let mut translation_offset = input.len() * output.len() * FLOAT_BYTES;
        for destination in output {
            let mut dot = 0.0;
            for &component in input {
                let coefficient = sample_at(samples, coefficient_offset, account)?;
                dot = account.coordinate(|| coefficient.mul_add(component, dot))?;
                coefficient_offset += FLOAT_BYTES;
            }
            let translation = sample_at(samples, translation_offset, account)?;
            *destination = account.coordinate(|| checked_value(dot + translation))??;
            translation_offset += FLOAT_BYTES;
        }
        Ok(())
    }
}

pub(super) struct Grid {
    channels: Channels,
    dimensions: [u8; GRID_AXES],
}

impl Grid {
    pub(super) fn read(
        payload: &[u8],
        input: usize,
        output: usize,
        account: &Account,
    ) -> ProfileResult<Self> {
        let channels = Channels::new(input, output)?;
        let dimensions = account.coordinate(|| field::<GRID_AXES>(payload, ELEMENT_HEADER))??;
        let (used, unused) = dimensions
            .split_at_checked(input)
            .ok_or(ProfileError::Invalid)?;
        let mut length = output * FLOAT_BYTES;
        for &dimension in used {
            length = account
                .coordinate(|| {
                    length
                        .checked_mul(usize::from(dimension))
                        .filter(|_| dimension >= 2)
                })?
                .ok_or(ProfileError::Invalid)?;
        }
        for &dimension in unused {
            if account.coordinate(|| dimension != 0)? {
                return Err(ProfileError::Invalid);
            }
        }
        let samples = payload
            .get(GRID_HEADER..)
            .filter(|samples| samples.len() == length)
            .ok_or(ProfileError::Invalid)?;
        float_values(samples, account)?;
        Ok(Self {
            channels,
            dimensions,
        })
    }

    pub(super) fn evaluate(
        &self,
        payload: &[u8],
        input: &[f64],
        output: &mut [f64],
        account: &Account,
    ) -> Result<()> {
        self.channels.check(input, output)?;
        let samples = payload.get(GRID_HEADER..).ok_or_else(invariant)?;
        // Reuse one list of fractional axes for all interleaved channels.
        let _hold = account.acquire(
            RenderResource::ScratchCpuBytes,
            size_of::<[Axis; GRID_AXES]>(),
        )?;
        let mut axes = [Axis {
            stride: 0,
            fraction: 0.0,
        }; GRID_AXES];
        let mut active = 0;
        let mut offset = 0;
        let mut stride = output.len() * FLOAT_BYTES;
        let dimensions = &self.dimensions[..input.len()];
        for (&size, &component) in dimensions.iter().zip(input).rev() {
            let size = usize::from(size);
            let (next_stride, fraction) = account.coordinate(|| {
                // The last input dimension varies fastest. The compiled extent
                // bounds every stride and every reachable sample offset.
                let position = component.clamp(0.0, 1.0) * (size - 1) as f64;
                let lower = position as usize;
                offset += lower * stride;
                (stride * size, position - lower as f64)
            })?;
            if fraction != 0.0 {
                axes[active] = Axis { stride, fraction };
                active += 1;
            }
            stride = next_stride;
        }

        let axes = &axes[..active];
        for (channel, destination) in output.iter_mut().enumerate() {
            let start = account.coordinate(|| offset + channel * FLOAT_BYTES)?;
            *destination = checked_value(interpolate(samples, axes, start, account)?)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Axis {
    stride: usize,
    fraction: f64,
}

fn interpolate(samples: &[u8], axes: &[Axis], start: usize, account: &Account) -> Result<f64> {
    let Some((axis, rest)) = axes.split_last() else {
        return sample_at(samples, start, account);
    };
    // At most 16 axes. Removing exact knots avoids visiting unused corners.
    // Blend the fastest-changing axis first, as for the legacy RGB grid.
    let lower = interpolate(samples, rest, start, account)?;
    let next = account.coordinate(|| start + axis.stride)?;
    let upper = interpolate(samples, rest, next, account)?;
    account.coordinate(|| (upper - lower).mul_add(axis.fraction, lower))
}

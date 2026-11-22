//! Uniform channel tables and multidimensional grids over packed ICC samples.

use super::{Account, MAX_CHANNELS, ProfileError, ProfileResult, Result, sample_invariant};

#[derive(Clone, Copy)]
pub(super) enum Precision {
    Eight,
    Sixteen,
}

impl Precision {
    const fn bytes(self) -> usize {
        match self {
            Self::Eight => 1,
            Self::Sixteen => 2,
        }
    }
}

struct Samples {
    start: usize,
    end: usize,
    precision: Precision,
}

impl Samples {
    fn read(
        start: usize,
        count: usize,
        precision: Precision,
        length: usize,
    ) -> ProfileResult<Self> {
        let end = count
            .checked_mul(precision.bytes())
            .and_then(|size| start.checked_add(size))
            .filter(|&end| end <= length)
            .ok_or(ProfileError::Invalid)?;
        Ok(Self {
            start,
            end,
            precision,
        })
    }

    fn backing<'a>(&self, payload: &'a [u8]) -> Result<&'a [u8]> {
        payload
            .get(self.start..self.end)
            .ok_or_else(sample_invariant)
    }

    fn sample(&self, bytes: &[u8], index: usize, account: &Account) -> Result<f64> {
        account
            .coordinate(|| match self.precision {
                Precision::Eight => bytes.get(index).map(|&value| f64::from(value) / 255.0),
                Precision::Sixteen => bytes
                    .as_chunks::<2>()
                    .0
                    .get(index)
                    .map(|&value| f64::from(u16::from_be_bytes(value)) / 65535.0),
            })?
            .ok_or_else(sample_invariant)
    }
}

/// ICC legacy input and output tables are packed one channel at a time.
pub(super) struct ChannelTables {
    samples: Samples,
    entries: usize,
    channels: usize,
}

impl ChannelTables {
    pub(super) fn read(
        start: usize,
        entries: usize,
        channels: usize,
        precision: Precision,
        length: usize,
    ) -> ProfileResult<Self> {
        if entries < 2 || !(1..=MAX_CHANNELS).contains(&channels) {
            return Err(ProfileError::Invalid);
        }
        let count = entries.checked_mul(channels).ok_or(ProfileError::Invalid)?;
        Ok(Self {
            samples: Samples::read(start, count, precision, length)?,
            entries,
            channels,
        })
    }

    pub(super) const fn end(&self) -> usize {
        self.samples.end
    }

    pub(super) fn evaluate(
        &self,
        input: &[f64],
        output: &mut [f64],
        payload: &[u8],
        account: &Account,
    ) -> Result<()> {
        if input.len() != self.channels || output.len() != self.channels {
            return Err(sample_invariant());
        }
        let bytes = self.samples.backing(payload)?;
        for (channel, (destination, &value)) in output.iter_mut().zip(input).enumerate() {
            let start = channel * self.entries;
            let axis = Axis::locate(value, self.entries, account)?;
            let lower = self.samples.sample(bytes, start + axis.lower, account)?;
            *destination = if axis.fraction == 0.0 {
                lower
            } else {
                let upper = self
                    .samples
                    .sample(bytes, start + axis.lower + 1, account)?;
                account.coordinate(|| mix(lower, upper, axis.fraction))?
            };
        }
        Ok(())
    }
}

/// The first input axis varies slowest, with output channels
/// interleaved at each corner (ICC.1:2022 §§10.10–10.12).
pub(super) struct Grid {
    samples: Samples,
    dimensions: [u8; MAX_CHANNELS],
    input_channels: usize,
    output_channels: usize,
}

impl Grid {
    pub(super) fn read(
        start: usize,
        dimensions: &[u8],
        output_channels: usize,
        precision: Precision,
        length: usize,
    ) -> ProfileResult<Self> {
        let input_channels = dimensions.len();
        if !(1..=MAX_CHANNELS).contains(&input_channels)
            || !(1..=MAX_CHANNELS).contains(&output_channels)
            || dimensions.iter().any(|&size| size < 2)
        {
            return Err(ProfileError::Invalid);
        }
        let count = dimensions
            .iter()
            .copied()
            .map(usize::from)
            .try_fold(output_channels, usize::checked_mul)
            .ok_or(ProfileError::Invalid)?;
        let mut sizes = [0; MAX_CHANNELS];
        sizes[..input_channels].copy_from_slice(dimensions);
        Ok(Self {
            samples: Samples::read(start, count, precision, length)?,
            dimensions: sizes,
            input_channels,
            output_channels,
        })
    }

    pub(super) const fn end(&self) -> usize {
        self.samples.end
    }

    pub(super) fn evaluate(
        &self,
        input: &[f64],
        output: &mut [f64],
        payload: &[u8],
        account: &Account,
    ) -> Result<()> {
        if input.len() != self.input_channels || output.len() != self.output_channels {
            return Err(sample_invariant());
        }
        let bytes = self.samples.backing(payload)?;
        let dimensions = &self.dimensions[..self.input_channels];
        let mut axes = [Axis {
            lower: 0,
            fraction: 0.0,
        }; MAX_CHANNELS];
        for ((axis, &value), &size) in axes.iter_mut().zip(input).zip(dimensions) {
            *axis = Axis::locate(value, usize::from(size), account)?;
        }
        let mut corner = [0; MAX_CHANNELS];
        let mut scratch = [[0.0; MAX_CHANNELS]; MAX_CHANNELS];
        GridInterpolator {
            samples: &self.samples,
            bytes,
            axes: &axes[..self.input_channels],
            dimensions,
            account,
        }
        .interpolate(0, &mut corner, output, &mut scratch)
    }
}

struct GridInterpolator<'a> {
    samples: &'a Samples,
    bytes: &'a [u8],
    axes: &'a [Axis],
    dimensions: &'a [u8],
    account: &'a Account,
}

impl GridInterpolator<'_> {
    fn interpolate(
        &self,
        depth: usize,
        corner: &mut [usize; MAX_CHANNELS],
        output: &mut [f64],
        scratch: &mut [[f64; MAX_CHANNELS]],
    ) -> Result<()> {
        let Some(axis) = self.axes.get(depth) else {
            // Any reachable row-major index and every interleaved channel
            // fall within the checked sample count established at read time.
            let first = self.account.coordinate(|| {
                let index = corner
                    .iter()
                    .zip(self.dimensions)
                    .fold(0, |index, (&coordinate, &size)| {
                        index * usize::from(size) + coordinate
                    });
                index * output.len()
            })?;
            for (channel, destination) in output.iter_mut().enumerate() {
                *destination = self
                    .samples
                    .sample(self.bytes, first + channel, self.account)?;
            }
            return Ok(());
        };

        corner[depth] = axis.lower;
        self.interpolate(depth + 1, corner, output, scratch)?;
        if axis.fraction != 0.0 {
            // A fractional axis keeps the lower result in output and uses one
            // scratch row for the upper result. Descendants reuse later rows.
            let (upper, remaining) = scratch.split_first_mut().ok_or_else(sample_invariant)?;
            let upper = &mut upper[..output.len()];
            corner[depth] = axis.lower + 1;
            self.interpolate(depth + 1, corner, upper, remaining)?;
            for (destination, &upper) in output.iter_mut().zip(upper.iter()) {
                *destination = self
                    .account
                    .coordinate(|| mix(*destination, upper, axis.fraction))?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Axis {
    lower: usize,
    fraction: f64,
}

impl Axis {
    fn locate(value: f64, length: usize, account: &Account) -> Result<Self> {
        account.coordinate(|| {
            if !value.is_finite() {
                return Err(sample_invariant());
            }
            // Every compiled axis has two or more entries. All input stages
            // produce finite normalized values. Bound any rounding at 0 or 1.
            let last = length - 1;
            let position = value.clamp(0.0, 1.0) * last as f64;
            let lower = (position as usize).min(last);
            Ok(Self {
                lower,
                fraction: if lower == last {
                    0.0
                } else {
                    position - lower as f64
                },
            })
        })?
    }
}

fn mix(lower: f64, upper: f64, fraction: f64) -> f64 {
    (upper - lower).mul_add(fraction, lower)
}

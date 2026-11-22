//! Uniform channel tables and RGB grids over validated packed ICC samples.

use super::{Account, CHANNELS, ProfileError, ProfileResult, Result, sample_invariant};

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
}

impl ChannelTables {
    pub(super) fn read(
        start: usize,
        entries: usize,
        precision: Precision,
        length: usize,
    ) -> ProfileResult<Self> {
        if entries < 2 {
            return Err(ProfileError::Invalid);
        }
        let count = entries.checked_mul(CHANNELS).ok_or(ProfileError::Invalid)?;
        Ok(Self {
            samples: Samples::read(start, count, precision, length)?,
            entries,
        })
    }

    pub(super) const fn end(&self) -> usize {
        self.samples.end
    }

    pub(super) fn evaluate(
        &self,
        input: [f64; CHANNELS],
        payload: &[u8],
        account: &Account,
    ) -> Result<[f64; CHANNELS]> {
        let bytes = self.samples.backing(payload)?;
        let mut output = [0.0; CHANNELS];
        for (channel, (destination, value)) in output.iter_mut().zip(input).enumerate() {
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
        Ok(output)
    }
}

/// Three input axes, with the first varying slowest and output channels
/// interleaved at each corner (ICC.1:2022 §§10.10–10.12).
pub(super) struct Grid {
    samples: Samples,
    dimensions: [usize; CHANNELS],
}

impl Grid {
    pub(super) fn read(
        start: usize,
        dimensions: [u8; CHANNELS],
        precision: Precision,
        length: usize,
    ) -> ProfileResult<Self> {
        if dimensions.iter().any(|&size| size < 2) {
            return Err(ProfileError::Invalid);
        }
        let dimensions = dimensions.map(usize::from);
        let count = dimensions
            .into_iter()
            .try_fold(CHANNELS, usize::checked_mul)
            .ok_or(ProfileError::Invalid)?;
        Ok(Self {
            samples: Samples::read(start, count, precision, length)?,
            dimensions,
        })
    }

    pub(super) const fn end(&self) -> usize {
        self.samples.end
    }

    pub(super) fn evaluate(
        &self,
        input: [f64; CHANNELS],
        payload: &[u8],
        account: &Account,
    ) -> Result<[f64; CHANNELS]> {
        let bytes = self.samples.backing(payload)?;
        let [r, g, b] = input;
        let [nr, ng, nb] = self.dimensions;
        let red = Axis::locate(r, nr, account)?;
        let green = Axis::locate(g, ng, account)?;
        let blue = Axis::locate(b, nb, account)?;
        red.interpolate(account, |r| {
            green.interpolate(account, |g| {
                blue.interpolate(account, |b| {
                    // The compiled grid extent bounds every corner and stride.
                    let first = account.coordinate(|| ((r * ng + g) * nb + b) * CHANNELS)?;
                    Ok([
                        self.samples.sample(bytes, first, account)?,
                        self.samples.sample(bytes, first + 1, account)?,
                        self.samples.sample(bytes, first + 2, account)?,
                    ])
                })
            })
        })
    }
}

struct Axis {
    lower: usize,
    fraction: f64,
}

impl Axis {
    fn locate(value: f64, length: usize, account: &Account) -> Result<Self> {
        account.coordinate(|| {
            // Every compiled axis has two or more entries. All input stages
            // produce finite normalized values. Bound any rounding at 0 or 1.
            let position = value.clamp(0.0, 1.0) * (length - 1) as f64;
            let lower = position as usize;
            Self {
                lower,
                fraction: position - lower as f64,
            }
        })
    }

    fn interpolate(
        &self,
        account: &Account,
        sample: impl Fn(usize) -> Result<[f64; CHANNELS]>,
    ) -> Result<[f64; CHANNELS]> {
        let mut output = sample(self.lower)?;
        if self.fraction != 0.0 {
            for (destination, upper) in output.iter_mut().zip(sample(self.lower + 1)?) {
                *destination = account.coordinate(|| mix(*destination, upper, self.fraction))?;
            }
        }
        Ok(output)
    }
}

fn mix(lower: f64, upper: f64, fraction: f64) -> f64 {
    (upper - lower).mul_add(fraction, lower)
}

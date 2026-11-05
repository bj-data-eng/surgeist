//! Find the smallest TrueType flag and coordinate representation of an outline.
use super::{Budget, Buffer, DecodeError, Point, add, multiply};

pub(super) const MAX_POINTS: usize = u16::MAX as usize + 1;
const MAX_RUN: usize = 256;
const PATTERNS: usize = 32;
const REPEAT: u8 = 0x08;
const OVERLAP: u8 = 0x40;

#[derive(Clone, Copy)]
enum Axis {
    Same,
    PositiveByte,
    NegativeByte,
    Word,
}

impl Axis {
    const ALL: [Self; 4] = [
        Self::Same,
        Self::PositiveByte,
        Self::NegativeByte,
        Self::Word,
    ];

    // X-axis bits. Y uses the same pattern shifted left one bit.
    fn flag(self) -> u8 {
        match self {
            Self::Same => 0x10,
            Self::PositiveByte => 0x12,
            Self::NegativeByte => 0x02,
            Self::Word => 0,
        }
    }

    fn from_flag(flag: u8) -> Self {
        match flag & 0x12 {
            0x10 => Self::Same,
            0x12 => Self::PositiveByte,
            0x02 => Self::NegativeByte,
            _ => Self::Word,
        }
    }

    fn width(self) -> u32 {
        match self {
            Self::Same => 0,
            Self::PositiveByte | Self::NegativeByte => 1,
            Self::Word => 2,
        }
    }

    fn fits(self, delta: i16) -> bool {
        match self {
            Self::Same => delta == 0,
            Self::PositiveByte => (0..=255).contains(&delta),
            Self::NegativeByte => (-255..=0).contains(&delta),
            Self::Word => true,
        }
    }

    fn write(self, delta: i16, output: &mut Buffer<'_, u8>) -> Result<(), DecodeError> {
        match self {
            Self::Same => Ok(()),
            Self::PositiveByte | Self::NegativeByte => output
                .push(u8::try_from(delta.unsigned_abs()).map_err(|_| DecodeError::SizeOverflow)?),
            Self::Word => output.extend(&delta.to_be_bytes()),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Step {
    bytes: u32,
    start: u16,
    flag: u8,
}

#[derive(Clone, Copy, Default)]
struct Candidate {
    score: u32,
    start: u16,
}

#[derive(Clone, Copy, Default)]
struct Window {
    head: usize,
    len: usize,
    previous_fits: bool,
}

impl Window {
    fn insert(&mut self, row: &mut [Candidate], oldest: usize, candidate: Candidate) -> Candidate {
        let capacity = row.len();
        while self.len != 0 && usize::from(row[self.head].start) < oldest {
            self.head = (self.head + 1) % capacity;
            self.len -= 1;
        }
        // A newer start with an equal or smaller score expires later. Each
        // start is inserted and removed at most once per pattern.
        while self.len != 0 && row[(self.head + self.len - 1) % capacity].score >= candidate.score {
            self.len -= 1;
        }
        row[(self.head + self.len) % capacity] = candidate;
        self.len += 1;
        row[self.head]
    }
}

#[derive(Clone, Copy)]
struct Run {
    flag: u8,
    repeats: u8,
}

pub(super) struct Encoder<'a> {
    trace: Buffer<'a, Step>,
    candidates: Buffer<'a, Candidate>,
    runs: Buffer<'a, Run>,
}

impl<'a> Encoder<'a> {
    pub fn new(budget: &'a Budget) -> Self {
        Self {
            trace: Buffer::new(budget),
            candidates: Buffer::new(budget),
            runs: Buffer::new(budget),
        }
    }

    pub fn plan(&mut self, points: &[Point]) -> Result<usize, DecodeError> {
        // A contour's final point index is uint16. Consequently every run
        // start fits u16, and a five-byte singleton per point bounds all
        // optimal prefix costs well below u32::MAX.
        if points.len() > MAX_POINTS {
            return Err(DecodeError::SizeOverflow);
        }
        let slots = points.len().saturating_sub(1).min(MAX_RUN - 1);
        self.trace.resize(add(points.len(), 1)?, Step::default())?;
        self.candidates
            .resize(multiply(PATTERNS, slots)?, Candidate::default())?;
        self.trace[0] = Step::default();
        let mut windows = [Window::default(); PATTERNS];
        for (index, point) in points.iter().enumerate() {
            let end = index + 1;
            // A word for each of the two checked deltas is always a valid
            // singleton. Other patterns can reduce its cost.
            let mut best = Step {
                bytes: self.trace[index].bytes + 5,
                start: index as u16,
                flag: u8::from(point.on_curve),
            };

            // Every encoding ends with a singleton or a repeated run with
            // one of the 2 * 4 * 4 flag patterns. For a repeated run [j, end)
            // with fixed coordinate width w, the cost is prefix[j] +
            // (end - j) * w + 2. The window minimizes prefix[j] - j*w over
            // starts with 2..=256 compatible points. Add MAX_POINTS*w to
            // every score in a window to keep that comparison unsigned.
            for (pattern, window) in windows.iter_mut().enumerate() {
                let x = Axis::ALL[pattern & 3];
                let y = Axis::ALL[(pattern >> 2) & 3];
                let on_curve = pattern & 16 != 0;
                if on_curve != point.on_curve || !x.fits(point.dx) || !y.fits(point.dy) {
                    *window = Window::default();
                    continue;
                }
                let flag = u8::from(on_curve) | x.flag() | (y.flag() << 1);
                let width = x.width() + y.width();
                let mut choice = Step {
                    bytes: self.trace[index].bytes + 1 + width,
                    start: index as u16,
                    flag,
                };
                if window.previous_fits {
                    // Earlier incompatibility clears the window. The only
                    // newly eligible start is two points before this end.
                    let start = end - 2;
                    let candidate = Candidate {
                        score: self.trace[start].bytes + (MAX_POINTS - start) as u32 * width,
                        start: start as u16,
                    };
                    let row = &mut self.candidates[pattern * slots..(pattern + 1) * slots];
                    let least = window.insert(row, end.saturating_sub(MAX_RUN), candidate);
                    let bytes = least.score - (MAX_POINTS - end) as u32 * width + 2;
                    if bytes < choice.bytes {
                        choice = Step {
                            bytes,
                            start: least.start,
                            flag,
                        };
                    }
                }
                window.previous_fits = true;
                if choice.bytes < best.bytes {
                    best = choice;
                }
            }
            self.trace[end] = best;
        }

        self.runs.clear();
        let mut end = points.len();
        let mut count = 0;
        while end != 0 {
            end = usize::from(self.trace[end].start);
            count += 1;
        }
        self.runs.reserve(count)?;
        end = points.len();
        while end != 0 {
            let step = self.trace[end];
            let start = usize::from(step.start);
            self.runs.push(Run {
                flag: step.flag,
                repeats: (end - start - 1) as u8,
            })?;
            end = start;
        }
        self.runs.reverse();
        Ok(self.trace[points.len()].bytes as usize)
    }

    pub fn write(
        &self,
        points: &[Point],
        overlap: bool,
        output: &mut Buffer<'_, u8>,
    ) -> Result<(), DecodeError> {
        // Preserve planned run boundaries so emission uses precisely the
        // byte count established before reserving the glyph buffer.
        for (index, run) in self.runs.iter().enumerate() {
            // WOFF2 §5.1 places overlap on the first stored flag byte. It is
            // a glyph property; it neither changes vector widths nor splits
            // the first repeat run into a separate logical point.
            let flag = run.flag | if overlap && index == 0 { OVERLAP } else { 0 };
            if run.repeats == 0 {
                output.push(flag)?;
            } else {
                output.extend(&[flag | REPEAT, run.repeats])?;
            }
        }
        for vertical in [false, true] {
            let flags = self
                .runs
                .iter()
                .flat_map(|run| std::iter::repeat_n(run.flag, usize::from(run.repeats) + 1));
            for (point, flag) in points.iter().zip(flags) {
                let delta = if vertical { point.dy } else { point.dx };
                Axis::from_flag(flag >> u8::from(vertical)).write(delta, output)?;
            }
        }
        Ok(())
    }
}

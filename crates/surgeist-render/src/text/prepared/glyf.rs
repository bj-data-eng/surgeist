//! Complete raw glyf admission before the provider's forgiving metadata/decoders.

use super::*;

fn invalid() -> Error {
    GlyphPreparationError::InvalidFont.into()
}
fn overflow() -> Error {
    GlyphPreparationError::CapacityOverflow.into()
}
fn unsupported() -> Error {
    GlyphPreparationError::UnsupportedOutline {
        capability: GlyphCapability::CompositeGlyph,
    }
    .into()
}
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or_else(overflow)
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or_else(overflow)
}
fn word(data: &[u8], at: usize) -> Result<u16> {
    let bytes = data.get(at..add(at, 2)?).ok_or_else(invalid)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}
fn signed(data: &[u8], at: usize) -> Result<i16> {
    Ok(word(data, at)? as i16)
}

pub(super) fn raw_glyph<'a>(font: &skrifa::FontRef<'a>, glyph: GlyphId) -> Result<&'a [u8]> {
    let count = u32::from(font.maxp().map_err(|_| invalid())?.num_glyphs());
    if glyph.to_u32() >= count {
        // Public roots have their existing GlyphRange check. An invalid child
        // reference is malformed composite source, not another caller request.
        return Err(invalid());
    }
    let head = font.head().map_err(|_| invalid())?;
    let loca = font
        .loca(Some(head.index_to_loc_format() == 1))
        .map_err(|_| invalid())?;
    let at = usize::try_from(glyph.to_u32()).map_err(|_| overflow())?;
    let start = usize::try_from(loca.get_raw(at).ok_or_else(invalid)?).map_err(|_| overflow())?;
    let end =
        usize::try_from(loca.get_raw(add(at, 1)?).ok_or_else(invalid)?).map_err(|_| overflow())?;
    font.table_data(Tag::new(b"glyf"))
        .ok_or_else(invalid)?
        .as_bytes()
        .get(start..end)
        .ok_or_else(invalid)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Bounds {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}
impl Bounds {
    fn header(data: &[u8]) -> Result<Self> {
        let result = Self {
            min_x: i32::from(signed(data, 2)?),
            min_y: i32::from(signed(data, 4)?),
            max_x: i32::from(signed(data, 6)?),
            max_y: i32::from(signed(data, 8)?),
        };
        if result.min_x > result.max_x || result.min_y > result.max_y {
            return Err(invalid());
        }
        Ok(result)
    }
    pub(super) fn point(x: i32, y: i32) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x,
            max_y: y,
        }
    }
    pub(super) fn include(&mut self, other: Self) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }
    fn translated(self, x: i32, y: i32) -> Result<Self> {
        Ok(Self {
            min_x: self.min_x.checked_add(x).ok_or_else(overflow)?,
            min_y: self.min_y.checked_add(y).ok_or_else(overflow)?,
            max_x: self.max_x.checked_add(x).ok_or_else(overflow)?,
            max_y: self.max_y.checked_add(y).ok_or_else(overflow)?,
        })
    }
    fn contains(self, other: Self) -> bool {
        self.min_x <= other.min_x
            && self.min_y <= other.min_y
            && self.max_x >= other.max_x
            && self.max_y >= other.max_y
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Summary {
    pub(super) points: usize,
    contours: usize,
    max_simple: usize,
    max_other: usize,
    component_stack: usize,
    depth: usize,
    pub(super) bounds: Option<Bounds>,
}
impl Summary {
    pub(super) fn simple(points: usize, contours: usize, bounds: Option<Bounds>) -> Result<Self> {
        Ok(Self {
            points,
            contours,
            max_simple: add(points, 4)?,
            max_other: add(points, 4)?,
            bounds,
            ..Self::default()
        })
    }
    fn metadata(self) -> Result<()> {
        let points = add(self.points, 4)?;
        if points > usize::from(u16::MAX) || self.depth > 32 {
            return Err(unsupported());
        }
        // Outline::required_buffer_size(None), static glyf: prove each otherwise
        // unchecked arithmetic operation before get()/draw_memory_size().
        let bytes = add(
            mul(points, size_of::<skrifa::raw::types::Point<i32>>())?,
            mul(self.max_other, size_of::<skrifa::raw::types::Point<i32>>())?,
        )?;
        let bytes = add(bytes, mul(self.contours, size_of::<u16>())?)?;
        let bytes = add(
            bytes,
            mul(points, size_of::<skrifa::raw::tables::glyf::PointFlags>())?,
        )?;
        let _ = add(bytes, std::mem::align_of::<i32>())?;
        Ok(())
    }
}

/// Validate exact source extents, not generated accessors' empty defaults.
/// Residual bytes after coordinates are uninterpreted, not part of decoding.
pub(super) fn simple_structure(data: &[u8], glyph: GlyphId) -> Result<(usize, usize)> {
    let contours = usize::try_from(signed(data, 0)?).map_err(|_| invalid())?;
    Bounds::header(data)?;
    let ends_end = add(10, mul(contours, 2)?)?;
    let mut prior = None;
    for at in (10..ends_end).step_by(2) {
        let end = word(data, at)?;
        if prior.is_some_and(|last| end <= last) {
            return Err(invalid());
        }
        prior = Some(end);
    }
    let count = prior.map_or(0, |end| usize::from(end) + 1);
    let instructions = usize::from(word(data, ends_end)?);
    let flags_start = add(add(ends_end, 2)?, instructions)?;
    data.get(..flags_start).ok_or_else(invalid)?;
    let mut at = flags_start;
    let mut logical = 0;
    let mut x_bytes = 0;
    let mut y_bytes = 0;
    while logical < count {
        let flag = *data.get(at).ok_or_else(invalid)?;
        at = add(at, 1)?;
        let repeated = if flag & 8 != 0 {
            let repeat = usize::from(*data.get(at).ok_or_else(invalid)?);
            at = add(at, 1)?;
            add(repeat, 1)?
        } else {
            1
        };
        logical = add(logical, repeated)?;
        if logical > count {
            return Err(invalid());
        }
        let x = if flag & 2 != 0 {
            1
        } else if flag & 16 != 0 {
            0
        } else {
            2
        };
        let y = if flag & 4 != 0 {
            1
        } else if flag & 32 != 0 {
            0
        } else {
            2
        };
        x_bytes = add(x_bytes, mul(x, repeated)?)?;
        y_bytes = add(y_bytes, mul(y, repeated)?)?;
    }
    let end = add(add(at, x_bytes)?, y_bytes)?;
    data.get(..end).ok_or_else(invalid)?;
    // read_points_fast prefetches at most count bytes. Full prefix <= count
    // proves every encoded flag was prefetched, its clamp is inactive, and
    // coordinate decoding begins exactly after the complete flag prefix.
    if at - flags_start > count {
        return Err(GlyphPreparationError::Provider {
            glyph: glyph.to_u32(),
        }
        .into());
    }
    Ok((count, contours))
}

#[derive(Clone, Copy)]
enum State {
    Active,
    Qualified(Summary),
}
struct Memo {
    glyph: GlyphId,
    state: State,
}
struct Frame<'a> {
    glyph: GlyphId,
    data: &'a [u8],
    at: usize,
    terminal: bool,
    instructions: bool,
    first_overlap: bool,
    components: usize,
    pending_offset: (i32, i32),
    summary: Summary,
}
impl<'a> Frame<'a> {
    fn new(glyph: GlyphId, data: &'a [u8]) -> Result<Self> {
        if signed(data, 0)? >= 0 {
            return Err(invalid());
        }
        Bounds::header(data)?;
        Ok(Self {
            glyph,
            data,
            at: 10,
            terminal: false,
            instructions: false,
            first_overlap: false,
            components: 0,
            pending_offset: (0, 0),
            summary: Summary::default(),
        })
    }
    fn next(&mut self) -> Result<GlyphId> {
        let flags = word(self.data, self.at)?;
        let glyph = GlyphId::new(u32::from(word(self.data, add(self.at, 2)?)?));
        self.at = add(self.at, 4)?;
        let (x, y) = if flags & 1 != 0 {
            let result = (
                i32::from(signed(self.data, self.at)?),
                i32::from(signed(self.data, add(self.at, 2)?)?),
            );
            self.at = add(self.at, 4)?;
            result
        } else {
            let bytes = self
                .data
                .get(self.at..add(self.at, 2)?)
                .ok_or_else(invalid)?;
            self.at = add(self.at, 2)?;
            (i32::from(bytes[0] as i8), i32::from(bytes[1] as i8))
        };
        let transforms = flags & (8 | 64 | 128);
        if transforms.count_ones() > 1 || flags & 0x1800 == 0x1800 {
            return Err(invalid());
        }
        let identity = match transforms {
            8 => {
                let v = signed(self.data, self.at)?;
                self.at = add(self.at, 2)?;
                v == 16384
            }
            64 => {
                let a = signed(self.data, self.at)?;
                let b = signed(self.data, add(self.at, 2)?)?;
                self.at = add(self.at, 4)?;
                a == 16384 && b == 16384
            }
            128 => {
                let values = [
                    signed(self.data, self.at)?,
                    signed(self.data, add(self.at, 2)?)?,
                    signed(self.data, add(self.at, 4)?)?,
                    signed(self.data, add(self.at, 6)?)?,
                ];
                self.at = add(self.at, 8)?;
                values == [16384, 0, 0, 16384]
            }
            _ => true,
        };
        let more = flags & 32 != 0;
        // All defined flags except obsolete/reserved bit 0x10 are explicit.
        if flags & !0x1fef != 0
            || flags & 2 == 0
            || flags & 512 != 0
            || !identity
            || (flags & 256 != 0 && more)
            || (flags & 1024 != 0 && self.components != 0 && !self.first_overlap)
        {
            return Err(unsupported());
        }
        if self.components == 0 {
            self.first_overlap = flags & 1024 != 0;
        }
        self.terminal = !more;
        self.instructions = flags & 256 != 0;
        self.components = add(self.components, 1)?;
        self.pending_offset = (x, y);
        Ok(glyph)
    }
    fn include(&mut self, child: Summary) -> Result<()> {
        self.summary.points = add(self.summary.points, child.points)?;
        self.summary.contours = add(self.summary.contours, child.contours)?;
        self.summary.max_simple = self.summary.max_simple.max(child.max_simple);
        self.summary.max_other = self.summary.max_other.max(child.max_other);
        self.summary.component_stack = self.summary.component_stack.max(child.component_stack);
        // Draw calls Scaler::load for every child, including loca-equal None.
        // Only the metadata walk skips None; its zero point/storage summary
        // must not erase the actual draw recursion level.
        self.summary.depth = self.summary.depth.max(add(child.depth, 1)?);
        if let Some(bounds) = child.bounds {
            let bounds = bounds.translated(self.pending_offset.0, self.pending_offset.1)?;
            if let Some(existing) = &mut self.summary.bounds {
                existing.include(bounds);
            } else {
                self.summary.bounds = Some(bounds);
            }
        }
        self.summary.metadata()
    }
    fn finish(mut self) -> Result<Summary> {
        if self.instructions {
            let len = usize::from(word(self.data, self.at)?);
            self.at = add(add(self.at, 2)?, len)?;
            self.data.get(..self.at).ok_or_else(invalid)?;
            self.summary.max_other = self.summary.max_other.max(add(self.summary.points, 4)?);
        }
        if self
            .data
            .get(self.at..)
            .ok_or_else(invalid)?
            .iter()
            .any(|&byte| byte != 0)
        {
            return Err(unsupported());
        }
        if let Some(bounds) = self.summary.bounds
            && !Bounds::header(self.data)?.contains(bounds)
        {
            return Err(invalid());
        }
        self.summary.component_stack = add(self.summary.component_stack, add(self.components, 4)?)?;
        self.summary.metadata()?;
        Ok(self.summary)
    }
}

fn vector_bytes<T>(values: &Vec<T>) -> Result<usize> {
    mul(values.capacity(), size_of::<T>())
}
fn grow<T>(values: &mut Vec<T>, other: usize, base: usize, limit: usize) -> Result<()> {
    if values.len() < values.capacity() {
        return Ok(());
    }
    let requested = add(values.len(), 1)?;
    let old = vector_bytes(values)?;
    check_limit(
        GlyphResource::ScratchBytes,
        add(
            add(base, other)?,
            add(old, mul(requested, size_of::<T>())?)?,
        )?,
        limit,
    )?;
    values.try_reserve_exact(1).map_err(|_| {
        Error::from(GlyphPreparationError::Allocation {
            resource: GlyphResource::ScratchBytes,
        })
    })?;
    check_limit(
        GlyphResource::ScratchBytes,
        add(add(base, other)?, add(old, vector_bytes(values)?)?)?,
        limit,
    )
}

fn root_metrics(font: &skrifa::FontRef<'_>, glyph: GlyphId) -> Result<()> {
    let count = usize::from(font.maxp().map_err(|_| invalid())?.num_glyphs());
    let records = usize::from(font.hhea().map_err(|_| invalid())?.number_of_h_metrics());
    if records == 0 || records > count {
        return Err(invalid());
    }
    let glyph = usize::try_from(glyph.to_u32()).map_err(|_| overflow())?;
    let at = if glyph < records {
        add(mul(glyph, 4)?, 2)?
    } else {
        add(mul(records, 4)?, mul(glyph - records, 2)?)?
    };
    let hmtx = font.table_data(Tag::new(b"hmtx")).ok_or_else(invalid)?;
    hmtx.as_bytes()
        .get(..mul(records, 4)?)
        .ok_or_else(invalid)?;
    word(hmtx.as_bytes(), at)?;
    // The final complete long metric supplies advance width for trailing LSBs.
    word(hmtx.as_bytes(), mul(records - 1, 4)?)?;
    Ok(())
}

pub(super) fn qualify(
    font: &skrifa::FontRef<'_>,
    glyph: GlyphId,
    coordinates: &[NormalizedCoord],
    limits: GlyphRunLimits,
    coordinate_bytes: usize,
) -> Result<Summary> {
    let raw = raw_glyph(font, glyph)?;
    if raw.is_empty() || signed(raw, 0)? >= 0 {
        return super::qualify_simple_glyf(font, glyph, coordinates, limits, coordinate_bytes);
    }
    if !coordinates.is_empty()
        || [b"fvar", b"gvar", b"HVAR", b"VARC"]
            .iter()
            .any(|tag| font.table_data(Tag::new(tag)).is_some())
    {
        return Err(unsupported());
    }
    root_metrics(font, glyph)?;
    let mut memos = Vec::<Memo>::new();
    let mut frames = Vec::<Frame<'_>>::new();
    grow(&mut memos, 0, coordinate_bytes, limits.scratch_bytes)?;
    memos.push(Memo {
        glyph,
        state: State::Active,
    });
    grow(
        &mut frames,
        vector_bytes(&memos)?,
        coordinate_bytes,
        limits.scratch_bytes,
    )?;
    frames.push(Frame::new(glyph, raw)?);
    loop {
        if frames.last().expect("root frame").terminal {
            let frame = frames.pop().expect("current frame");
            let completed_glyph = frame.glyph;
            let summary = frame.finish()?;
            memos
                .iter_mut()
                .find(|m| m.glyph == completed_glyph)
                .expect("active memo")
                .state = State::Qualified(summary);
            if let Some(parent) = frames.last_mut() {
                parent.include(summary)?;
            } else {
                return Ok(summary);
            }
            continue;
        }
        let child = frames.last_mut().expect("current frame").next()?;
        if let Some(memo) = memos.iter().find(|m| m.glyph == child) {
            let State::Qualified(summary) = memo.state else {
                return Err(invalid());
            };
            if add(frames.len(), summary.depth)? > 32 {
                return Err(unsupported());
            }
            frames.last_mut().expect("parent frame").include(summary)?;
            continue;
        }
        let raw = raw_glyph(font, child)?;
        if frames.len() > 32 {
            return Err(unsupported());
        }
        if raw.is_empty() || signed(raw, 0)? >= 0 {
            let live = add(
                coordinate_bytes,
                add(vector_bytes(&memos)?, vector_bytes(&frames)?)?,
            )?;
            let summary = super::qualify_simple_glyf(font, child, coordinates, limits, live)?;
            grow(
                &mut memos,
                vector_bytes(&frames)?,
                coordinate_bytes,
                limits.scratch_bytes,
            )?;
            memos.push(Memo {
                glyph: child,
                state: State::Qualified(summary),
            });
            frames.last_mut().expect("parent frame").include(summary)?;
        } else {
            grow(
                &mut memos,
                vector_bytes(&frames)?,
                coordinate_bytes,
                limits.scratch_bytes,
            )?;
            memos.push(Memo {
                glyph: child,
                state: State::Active,
            });
            grow(
                &mut frames,
                vector_bytes(&memos)?,
                coordinate_bytes,
                limits.scratch_bytes,
            )?;
            frames.push(Frame::new(child, raw)?);
        }
    }
}

#[cfg(test)]
mod tests;

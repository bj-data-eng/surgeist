//! Reverse glyf/loca transform; triplet equations adapted from Wuff 0.2.9.
mod encoding;

use super::{
    Budget, Buffer, DecodeError, InvalidKind, Reader, add, as_u32, invalid, multiply, table_limit,
};

pub(super) struct Tables<'a> {
    pub glyf: Buffer<'a, u8>,
    pub loca: Buffer<'a, u8>,
}
#[derive(Clone, Copy)]
struct Point {
    dx: i16,
    dy: i16,
    on_curve: bool,
}

pub(super) struct Header {
    options: u16,
    pub glyphs: u16,
    pub index_format: u16,
}

impl Header {
    pub fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        if reader.u16()? != 0 {
            return Err(invalid(InvalidKind::WoffGlyph, 0));
        }
        let options = reader.u16()?;
        let glyphs = reader.u16()?;
        let index_format = reader.u16()?;
        if index_format > 1 {
            return Err(invalid(InvalidKind::WoffGlyph, 6));
        }
        Ok(Self {
            options,
            glyphs,
            index_format,
        })
    }
}

struct Decoder<'a> {
    contours: Reader<'a>,
    points: Reader<'a>,
    flags: Reader<'a>,
    coordinates: Reader<'a>,
    composites: Reader<'a>,
    boxes: Reader<'a>,
    instructions: Reader<'a>,
    bbox_bitmap: &'a [u8],
    overlap_bitmap: Option<&'a [u8]>,
    glyphs: usize,
    index_format: u16,
}
impl<'a> Decoder<'a> {
    fn new(data: &'a [u8]) -> Result<Self, DecodeError> {
        let mut reader = Reader::new(data, InvalidKind::WoffGlyph);
        let header = Header::read(&mut reader)?;
        let glyphs = usize::from(header.glyphs);
        let mut sizes = [0usize; 7];
        for size in &mut sizes {
            *size = reader.u32()? as usize;
        }
        let mut streams = [&[][..]; 7];
        for (stream, size) in streams.iter_mut().zip(sizes) {
            *stream = reader.take(size)?;
        }
        let bitmap_length = multiply(glyphs.div_ceil(32), 4)?;
        let mut boxes = Reader::new(streams[5], InvalidKind::WoffGlyph);
        let bbox_bitmap = boxes.take(bitmap_length)?;
        let overlap_bitmap = if header.options & 1 != 0 {
            Some(reader.take(glyphs.div_ceil(8))?)
        } else {
            None
        };
        reader.finish()?;
        Ok(Self {
            contours: Reader::new(streams[0], InvalidKind::WoffGlyph),
            points: Reader::new(streams[1], InvalidKind::WoffGlyph),
            flags: Reader::new(streams[2], InvalidKind::WoffGlyph),
            coordinates: Reader::new(streams[3], InvalidKind::WoffGlyph),
            composites: Reader::new(streams[4], InvalidKind::WoffGlyph),
            boxes,
            instructions: Reader::new(streams[6], InvalidKind::WoffGlyph),
            bbox_bitmap,
            overlap_bitmap,
            glyphs,
            index_format: header.index_format,
        })
    }
    fn visit(
        mut self,
        budget: &Budget,
        mut emit: impl FnMut(&[u8]) -> Result<(), DecodeError>,
    ) -> Result<(), DecodeError> {
        let mut glyph = Buffer::new(budget);
        let mut points = Buffer::new(budget);
        let mut ends = Buffer::new(budget);
        let mut encoder = encoding::Encoder::new(budget);
        for index in 0..self.glyphs {
            glyph.clear();
            points.clear();
            ends.clear();
            let contour_position = self.contours.position;
            let contours = self.contours.i16()?;
            let bbox_position = index / 8;
            let explicit = self.bbox_bitmap[bbox_position] & (0x80 >> (index % 8)) != 0;
            match contours {
                0 => {
                    if explicit {
                        return Err(invalid(InvalidKind::WoffGlyph, bbox_position));
                    }
                }
                -1 => {
                    if !explicit {
                        return Err(invalid(InvalidKind::WoffGlyph, bbox_position));
                    }
                    self.composite(&mut glyph, budget)?;
                }
                1..=i16::MAX => {
                    let count = contours as usize;
                    ends.reserve(count)?;
                    let mut total = 0usize;
                    for _ in 0..count {
                        let position = self.points.position;
                        let count = usize::from(self.points.uint255()?);
                        if count == 0 {
                            return Err(invalid(InvalidKind::WoffGlyph, position));
                        }
                        total = add(total, count)?;
                        if total > encoding::MAX_POINTS {
                            return Err(invalid(InvalidKind::WoffGlyph, position));
                        }
                        ends.push((total - 1) as u16)?;
                    }
                    let flags = self.flags.take(total)?;
                    points.reserve(total)?;
                    let (mut x, mut y) = (0i16, 0i16);
                    let mut inferred = None::<[i16; 4]>;
                    for flag in flags {
                        let position = self.coordinates.position;
                        let (dx, dy) = triplet(*flag & 127, &mut self.coordinates)?;
                        if !explicit {
                            // Every point must fit the inferred signed bounds.
                            // Report the triplet that first makes that impossible.
                            x = x
                                .checked_add(dx)
                                .ok_or_else(|| invalid(InvalidKind::WoffGlyph, position))?;
                            y = y
                                .checked_add(dy)
                                .ok_or_else(|| invalid(InvalidKind::WoffGlyph, position))?;
                            let bounds = inferred.get_or_insert([x, y, x, y]);
                            bounds[0] = bounds[0].min(x);
                            bounds[1] = bounds[1].min(y);
                            bounds[2] = bounds[2].max(x);
                            bounds[3] = bounds[3].max(y);
                        }
                        points.push(Point {
                            dx,
                            dy,
                            on_curve: flag & 128 == 0,
                        })?;
                    }
                    let instructions = usize::from(self.coordinates.uint255()?);
                    let overlap = self
                        .overlap_bitmap
                        .is_some_and(|bitmap| bitmap[index / 8] & (0x80 >> (index % 8)) != 0);
                    let point_bytes = encoder.plan(&points)?;
                    let length = add(
                        add(12, multiply(count, 2)?)?,
                        add(point_bytes, instructions)?,
                    )?;
                    table_limit(length, budget)?;
                    glyph.reserve(length)?;
                    glyph.extend(&contours.to_be_bytes())?;
                    if let Some(bounds) = inferred {
                        for value in bounds {
                            glyph.extend(&value.to_be_bytes())?;
                        }
                    } else {
                        glyph.extend(self.boxes.take(8)?)?;
                    }
                    for end in &*ends {
                        glyph.extend(&end.to_be_bytes())?;
                    }
                    glyph.extend(&(instructions as u16).to_be_bytes())?;
                    glyph.extend(self.instructions.take(instructions)?)?;
                    encoder.write(&points, overlap, &mut glyph)?;
                }
                _ => return Err(invalid(InvalidKind::WoffGlyph, contour_position)),
            }
            emit(&glyph)?;
        }
        self.contours.finish()?;
        self.points.finish()?;
        self.flags.finish()?;
        self.coordinates.finish()?;
        self.composites.finish()?;
        self.boxes.finish()?;
        self.instructions.finish()?;
        Ok(())
    }
    fn composite(
        &mut self,
        glyph: &mut Buffer<'_, u8>,
        budget: &Budget,
    ) -> Result<(), DecodeError> {
        let start = self.composites.position;
        let mut have_instructions = false;
        loop {
            let flags = self.composites.u16()?;
            have_instructions |= flags & 0x100 != 0;
            let arguments = if flags & 1 != 0 { 4 } else { 2 };
            let matrix = if flags & 8 != 0 {
                2
            } else if flags & 64 != 0 {
                4
            } else if flags & 128 != 0 {
                8
            } else {
                0
            };
            self.composites.take(2 + arguments + matrix)?;
            if flags & 32 == 0 {
                break;
            }
        }
        let composite = &self.composites.data[start..self.composites.position];
        let instructions = if have_instructions {
            usize::from(self.coordinates.uint255()?)
        } else {
            0
        };
        let length = add(
            add(10, composite.len())?,
            if have_instructions {
                add(2, instructions)?
            } else {
                0
            },
        )?;
        table_limit(length, budget)?;
        glyph.reserve(length)?;
        glyph.extend(&(-1i16).to_be_bytes())?;
        glyph.extend(self.boxes.take(8)?)?;
        glyph.extend(composite)?;
        if have_instructions {
            glyph.extend(&(instructions as u16).to_be_bytes())?;
            glyph.extend(self.instructions.take(instructions)?)?;
        }
        Ok(())
    }
}

pub(super) fn reconstruct<'a>(
    data: &[u8],
    loca_original: usize,
    budget: &'a Budget,
) -> Result<Tables<'a>, DecodeError> {
    let decoder = Decoder::new(data)?;
    let glyphs = decoder.glyphs;
    let index_format = decoder.index_format;
    let loca_length = multiply(glyphs + 1, if index_format == 0 { 2 } else { 4 })?;
    // REC-WOFF2-20240808 §5.3 explicitly requires matching loca.origLength.
    // It is a validation comparison after deriving the length from numGlyphs
    // and indexFormat, never authority for an allocation request.
    if loca_original != loca_length {
        return Err(invalid(InvalidKind::WoffGlyph, 6));
    }
    table_limit(loca_length, budget)?;
    // First pass establishes the exact reconstructed length. It also validates
    // every stream before final table storage is allocated.
    let mut length = 0usize;
    decoder.visit(budget, |glyph| {
        length = word_aligned(add(length, glyph.len())?)?;
        table_limit(length, budget)?;
        as_u32(length)?;
        if index_format == 0 && length > 131070 {
            return Err(invalid(InvalidKind::WoffGlyph, 6));
        }
        Ok(())
    })?;
    let mut glyf = Buffer::new(budget);
    let mut loca = Buffer::new(budget);
    glyf.reserve(length)?;
    loca.reserve(loca_length)?;
    Decoder::new(data)?.visit(budget, |glyph| {
        append_offset(&mut loca, glyf.len(), index_format)?;
        glyf.extend(glyph)?;
        glyf.resize(word_aligned(glyf.len())?, 0)
    })?;
    append_offset(&mut loca, glyf.len(), index_format)?;
    Ok(Tables { glyf, loca })
}

fn word_aligned(length: usize) -> Result<usize, DecodeError> {
    // Short loca stores word offsets. Four-byte padding belongs to the outer
    // SFNT table, not each glyph: excess per-glyph padding can exhaust the
    // short-loca domain even when the outlines have a valid representation.
    Ok(add(length, 1)? & !1)
}

fn append_offset(loca: &mut Buffer<'_, u8>, offset: usize, format: u16) -> Result<(), DecodeError> {
    if format == 0 {
        let value = u16::try_from(offset / 2).map_err(|_| invalid(InvalidKind::WoffGlyph, 6))?;
        loca.extend(&value.to_be_bytes())
    } else {
        loca.extend(&as_u32(offset)?.to_be_bytes())
    }
}
fn signed(flag: u8, value: i32) -> i32 {
    if flag & 1 != 0 { value } else { -value }
}
fn triplet(flag: u8, reader: &mut Reader<'_>) -> Result<(i16, i16), DecodeError> {
    let position = reader.position;
    let count = if flag < 84 {
        1
    } else if flag < 120 {
        2
    } else if flag < 124 {
        3
    } else {
        4
    };
    let bytes = reader.take(count)?;
    let f = i32::from(flag);
    let b = |index: usize| i32::from(bytes[index]);
    let (dx, dy) = if flag < 10 {
        (0, signed(flag, ((f & 14) << 7) + b(0)))
    } else if flag < 20 {
        (signed(flag, (((f - 10) & 14) << 7) + b(0)), 0)
    } else if flag < 84 {
        (
            signed(flag, 1 + ((f - 20) & 0x30) + (b(0) >> 4)),
            signed(flag >> 1, 1 + (((f - 20) & 12) << 2) + (b(0) & 15)),
        )
    } else if flag < 120 {
        (
            signed(flag, 1 + (((f - 84) / 12) << 8) + b(0)),
            signed(flag >> 1, 1 + ((((f - 84) % 12) >> 2) << 8) + b(1)),
        )
    } else if flag < 124 {
        (
            signed(flag, (b(0) << 4) + (b(1) >> 4)),
            signed(flag >> 1, ((b(1) & 15) << 8) + b(2)),
        )
    } else {
        (
            signed(flag, (b(0) << 8) + b(1)),
            signed(flag >> 1, (b(2) << 8) + b(3)),
        )
    };
    let word = |delta| i16::try_from(delta).map_err(|_| invalid(InvalidKind::WoffGlyph, position));
    Ok((word(dx)?, word(dy)?))
}

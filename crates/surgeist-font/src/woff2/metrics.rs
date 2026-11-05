//! Reverse hmtx transform, including null-transformed and shared glyf context.
use super::{
    Budget, Buffer, DecodeError, Face, InvalidKind, Reader, Table, add, find, invalid, multiply,
    table_limit,
};

pub(super) fn reconstruct<'a>(
    face: Face,
    indices: &[usize],
    tables: &[Table],
    stream: &[u8],
    output: &[u8],
    budget: &'a Budget,
) -> Result<Buffer<'a, u8>, DecodeError> {
    let table = |tag| -> Result<&[u8], DecodeError> {
        let index =
            find(face, indices, tables, tag).ok_or_else(|| invalid(InvalidKind::WoffMetrics, 0))?;
        tables[index]
            .output
            .ok_or_else(|| invalid(InvalidKind::WoffMetrics, tables[index].start))?
            .bytes(output)
    };
    let hmtx =
        find(face, indices, tables, b"hmtx").ok_or_else(|| invalid(InvalidKind::WoffMetrics, 0))?;
    let mut reader = Reader::new(tables[hmtx].bytes(stream)?, InvalidKind::WoffMetrics);
    let flags = reader.u8()?;
    if flags == 0 || flags & 0xfc != 0 {
        return Err(invalid(InvalidKind::WoffMetrics, 0));
    }
    let mut hhea = Reader::new(table(b"hhea")?, InvalidKind::WoffMetrics);
    hhea.take(34)?;
    let metrics = usize::from(hhea.u16()?);
    let mut maxp = Reader::new(table(b"maxp")?, InvalidKind::WoffMetrics);
    maxp.take(4)?;
    let glyphs = usize::from(maxp.u16()?);
    if metrics == 0 || metrics > glyphs {
        return Err(invalid(InvalidKind::WoffMetrics, 34));
    }
    let glyph_table = table(b"glyf")?;
    let loca = table(b"loca")?;
    let mut head = Reader::new(table(b"head")?, InvalidKind::WoffMetrics);
    head.take(50)?;
    let format = head.u16()?;
    if format > 1 {
        return Err(invalid(InvalidKind::WoffMetrics, 50));
    }
    // For transformed outlines the transform header, maxp, and head describe
    // one common glyph population and index format; verify that correspondence.
    let glyf_index =
        find(face, indices, tables, b"glyf").ok_or_else(|| invalid(InvalidKind::WoffMetrics, 0))?;
    if tables[glyf_index].transformed {
        let mut transformed =
            Reader::new(tables[glyf_index].bytes(stream)?, InvalidKind::WoffMetrics);
        transformed.take(4)?;
        if usize::from(transformed.u16()?) != glyphs || transformed.u16()? != format {
            return Err(invalid(InvalidKind::WoffMetrics, 4));
        }
    }
    let output_length = add(multiply(metrics, 2)?, multiply(glyphs, 2)?)?;
    table_limit(output_length, budget)?;
    let mut result = Buffer::new(budget);
    result.resize(output_length, 0)?;
    for index in 0..metrics {
        result[index * 4..index * 4 + 2].copy_from_slice(reader.take(2)?);
    }
    let mut offsets = Reader::new(loca, InvalidKind::WoffMetrics);
    let mut previous = read_offset(&mut offsets, format)?;
    if previous > glyph_table.len() {
        return Err(invalid(InvalidKind::WoffMetrics, previous));
    }
    for index in 0..glyphs {
        let next = read_offset(&mut offsets, format)?;
        if next < previous || next > glyph_table.len() {
            return Err(invalid(InvalidKind::WoffMetrics, next));
        }
        let infer = flags & if index < metrics { 1 } else { 2 } != 0;
        let bearing = if infer {
            if next == previous {
                0
            } else {
                let mut glyph = Reader::new(&glyph_table[previous..next], InvalidKind::WoffMetrics);
                if glyph.i16()? == 0 { 0 } else { glyph.i16()? }
            }
        } else {
            reader.i16()?
        };
        let offset = if index < metrics {
            index * 4 + 2
        } else {
            metrics * 4 + (index - metrics) * 2
        };
        result[offset..offset + 2].copy_from_slice(&bearing.to_be_bytes());
        previous = next;
    }
    offsets.finish()?;
    reader.finish()?;
    Ok(result)
}
fn read_offset(reader: &mut Reader<'_>, format: u16) -> Result<usize, DecodeError> {
    if format == 0 {
        Ok(usize::from(reader.u16()?) * 2)
    } else {
        Ok(reader.u32()? as usize)
    }
}

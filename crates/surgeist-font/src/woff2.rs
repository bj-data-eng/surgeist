//! WOFF2 reconstruction, adapted from Wuff 0.2.9's MIT-licensed decoder.
//! The wire formats are governed by W3C REC-WOFF2-20240808.
mod buffer;
mod glyph;
mod metrics;

use super::{
    DecodeError, DecodeLimits, FontSource, InvalidKind, LimitKind, add, check, invalid, multiply,
};
use buffer::{Budget, Buffer};

const TAGS: [[u8; 4]; 63] = [
    *b"cmap", *b"head", *b"hhea", *b"hmtx", *b"maxp", *b"name", *b"OS/2", *b"post", *b"cvt ",
    *b"fpgm", *b"glyf", *b"loca", *b"prep", *b"CFF ", *b"VORG", *b"EBDT", *b"EBLC", *b"gasp",
    *b"hdmx", *b"kern", *b"LTSH", *b"PCLT", *b"VDMX", *b"vhea", *b"vmtx", *b"BASE", *b"GDEF",
    *b"GPOS", *b"GSUB", *b"EBSC", *b"JSTF", *b"MATH", *b"CBDT", *b"CBLC", *b"COLR", *b"CPAL",
    *b"SVG ", *b"sbix", *b"acnt", *b"avar", *b"bdat", *b"bloc", *b"bsln", *b"cvar", *b"fdsc",
    *b"feat", *b"fmtx", *b"fvar", *b"gvar", *b"hsty", *b"just", *b"lcar", *b"mort", *b"morx",
    *b"opbd", *b"prop", *b"trak", *b"Zapf", *b"Silf", *b"Glat", *b"Gloc", *b"Feat", *b"Sill",
];

#[derive(Clone, Copy)]
struct Table {
    tag: [u8; 4],
    transformed: bool,
    original_length: usize,
    start: usize,
    length: usize,
    output: Option<Region>,
    partner: Option<usize>,
}
#[derive(Clone, Copy)]
struct Region {
    start: usize,
    length: usize,
}
impl Region {
    fn bytes(self, data: &[u8]) -> Result<&[u8], DecodeError> {
        data.get(self.start..add(self.start, self.length)?)
            .ok_or_else(|| invalid(InvalidKind::Truncated, self.start))
    }
}
impl Table {
    fn bytes(self, data: &[u8]) -> Result<&[u8], DecodeError> {
        Region {
            start: self.start,
            length: self.length,
        }
        .bytes(data)
    }
}
#[derive(Clone, Copy)]
struct Face {
    flavor: [u8; 4],
    first: usize,
    count: usize,
    header: usize,
}

#[derive(Clone, Copy)]
struct Reader<'a> {
    data: &'a [u8],
    position: usize,
    kind: InvalidKind,
}
impl<'a> Reader<'a> {
    fn new(data: &'a [u8], kind: InvalidKind) -> Self {
        Self {
            data,
            position: 0,
            kind,
        }
    }
    fn take(&mut self, length: usize) -> Result<&'a [u8], DecodeError> {
        let end = add(self.position, length)?;
        let value = self
            .data
            .get(self.position..end)
            .ok_or_else(|| invalid(self.kind, self.position))?;
        self.position = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_be_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }
    fn i16(&mut self) -> Result<i16, DecodeError> {
        Ok(self.u16()? as i16)
    }
    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_be_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }
    fn tag(&mut self) -> Result<[u8; 4], DecodeError> {
        Ok(self.take(4)?.try_into().expect("four bytes"))
    }
    fn uint255(&mut self) -> Result<u16, DecodeError> {
        match self.u8()? {
            253 => self.u16(),
            254 => Ok(506 + u16::from(self.u8()?)),
            255 => Ok(253 + u16::from(self.u8()?)),
            n => Ok(u16::from(n)),
        }
    }
    fn base128(&mut self) -> Result<usize, DecodeError> {
        let start = self.position;
        let mut value = 0u32;
        for index in 0..5 {
            let byte = self.u8()?;
            if (index == 0 && byte == 0x80) || value & 0xfe00_0000 != 0 {
                return Err(invalid(self.kind, start));
            }
            value = (value << 7) | u32::from(byte & 0x7f);
            if byte & 0x80 == 0 {
                return usize::try_from(value).map_err(|_| DecodeError::SizeOverflow);
            }
        }
        Err(invalid(self.kind, start))
    }
    fn finish(self) -> Result<(), DecodeError> {
        if self.position == self.data.len() {
            Ok(())
        } else {
            Err(invalid(self.kind, self.position))
        }
    }
}

fn padded(value: usize) -> Result<usize, DecodeError> {
    Ok(add(value, 3)? & !3)
}
fn as_u32(value: usize) -> Result<u32, DecodeError> {
    u32::try_from(value).map_err(|_| DecodeError::SizeOverflow)
}
fn table_limit(length: usize, budget: &Budget) -> Result<(), DecodeError> {
    check(LimitKind::TableBytes, length, budget.limits.table_bytes())
}
fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0u32, |sum, bytes| {
        let mut word = [0; 4];
        word[..bytes.len()].copy_from_slice(bytes);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}
fn write_u16(data: &mut [u8], offset: usize, value: u16) {
    data[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}
fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}
fn supported_flavor(flavor: [u8; 4]) -> Result<(), DecodeError> {
    match &flavor {
        b"\0\x01\0\0" | b"OTTO" | b"true" | b"typ1" => Ok(()),
        _ => Err(DecodeError::UnsupportedFlavor(flavor)),
    }
}

pub(super) fn decode(source: &FontSource, limits: DecodeLimits) -> Result<Vec<u8>, DecodeError> {
    let budget = Budget::new(limits, source.retained_bytes()?)?;
    let data = source.bytes();
    let mut reader = Reader::new(data, InvalidKind::WoffHeader);
    if reader.tag()? != *b"wOF2" {
        return Err(invalid(InvalidKind::Signature, 0));
    }
    let flavor = reader.tag()?;
    let collection = flavor == *b"ttcf";
    if reader.u32()? as usize != data.len() {
        return Err(invalid(InvalidKind::WoffHeader, 8));
    }
    let count = usize::from(reader.u16()?);
    if count == 0 {
        return Err(invalid(InvalidKind::WoffDirectory, 12));
    }
    check(LimitKind::Tables, count, limits.tables())?;
    reader.u16()?; // reserved: decoders MUST ignore its value.
    reader.u32()?; // totalSfntSize is reference information, never authority.
    let compressed_length = reader.u32()? as usize;
    reader.take(4)?; // major/minor version.
    let metadata = Region {
        start: reader.u32()? as usize,
        length: reader.u32()? as usize,
    };
    let metadata_original = reader.u32()?;
    let private = Region {
        start: reader.u32()? as usize,
        length: reader.u32()? as usize,
    };
    if !collection {
        supported_flavor(flavor)?;
    }
    reader.kind = InvalidKind::WoffDirectory;
    let mut tables = Buffer::new(&budget);
    tables.reserve(count)?;
    let mut stream_length = 0usize;
    for _ in 0..count {
        let flags = reader.u8()?;
        let tag = if flags & 63 == 63 {
            reader.tag()?
        } else {
            TAGS[usize::from(flags & 63)]
        };
        let version = flags >> 6;
        let transformed = match (&tag, version) {
            (b"glyf" | b"loca", 0) | (b"hmtx", 1) => true,
            (b"glyf" | b"loca", 3) => false,
            (_, 0) if tag != *b"glyf" && tag != *b"loca" => false,
            _ => return Err(DecodeError::UnsupportedTransform { tag, version }),
        };
        let original_length = reader.base128()?;
        let length = if transformed {
            reader.base128()?
        } else {
            original_length
        };
        if tag == *b"loca" && transformed && length != 0 {
            return Err(invalid(InvalidKind::WoffTransform, reader.position));
        }
        // Transformed lengths bound the Brotli payload, while reconstructed
        // lengths are checked independently when the reverse transform writes.
        if !transformed {
            table_limit(length, &budget)?;
        }
        tables.push(Table {
            tag,
            transformed,
            original_length,
            start: stream_length,
            length,
            output: None,
            partner: None,
        })?;
        stream_length = add(stream_length, length)?;
    }
    let (version, face_count) = if collection {
        let version = reader.u32()?;
        if version != 0x0001_0000 && version != 0x0002_0000 {
            return Err(invalid(InvalidKind::CollectionVersion, reader.position - 4));
        }
        (version, usize::from(reader.uint255()?))
    } else {
        (0, 1)
    };
    if face_count == 0 {
        return Err(invalid(InvalidKind::EmptyCollection, reader.position));
    }
    check(LimitKind::Faces, face_count, limits.faces())?;
    // Discover the complete flattened directory request before allocating it;
    // collection face counts must not drive successive reallocations.
    let mut scan = reader;
    let mut total_indices = 0usize;
    if collection {
        for _ in 0..face_count {
            let count = usize::from(scan.uint255()?);
            if count == 0 {
                return Err(invalid(InvalidKind::WoffDirectory, scan.position));
            }
            total_indices = add(total_indices, count)?;
            check(LimitKind::Tables, total_indices, limits.tables())?;
            supported_flavor(scan.tag()?)?;
            for _ in 0..count {
                scan.uint255()?;
            }
        }
    } else {
        total_indices = count;
    }
    let mut faces = Buffer::new(&budget);
    faces.reserve(face_count)?;
    let mut indices = Buffer::new(&budget);
    indices.reserve(total_indices)?;
    let mut aggregate = 0usize;
    for _ in 0..face_count {
        let (face_count, face_flavor) = if collection {
            (usize::from(reader.uint255()?), reader.tag()?)
        } else {
            (count, flavor)
        };
        if face_count == 0 {
            return Err(invalid(InvalidKind::WoffDirectory, reader.position));
        }
        aggregate = add(aggregate, face_count)?;
        check(LimitKind::Tables, aggregate, limits.tables())?;
        let first = indices.len();
        indices.reserve(aggregate)?;
        for index in 0..face_count {
            let index = if collection {
                usize::from(reader.uint255()?)
            } else {
                index
            };
            if index >= tables.len() {
                return Err(invalid(InvalidKind::WoffDirectory, reader.position));
            }
            indices.push(index)?;
        }
        indices[first..].sort_unstable_by_key(|index| tables[*index].tag);
        if indices[first..]
            .windows(2)
            .any(|pair| tables[pair[0]].tag == tables[pair[1]].tag)
        {
            return Err(invalid(InvalidKind::TableOrder, reader.position));
        }
        faces.push(Face {
            flavor: face_flavor,
            first,
            count: face_count,
            header: 0,
        })?;
    }
    validate_pairs(&faces, &indices, &mut tables, collection)?;
    let compressed = Region {
        start: reader.position,
        length: compressed_length,
    };
    validate_blocks(data, compressed, metadata, metadata_original, private)?;
    let (live, workspace) = budget.usage();
    let stream = super::brotli::decompress(
        compressed.bytes(data)?,
        stream_length,
        limits,
        live,
        workspace,
    )?;
    let stream = Buffer::from_vec(stream, &budget)?;
    reconstruct(&budget, &stream, version, &mut faces, &indices, &mut tables)
}

fn validate_blocks(
    data: &[u8],
    compressed: Region,
    metadata: Region,
    metadata_original: u32,
    private: Region,
) -> Result<(), DecodeError> {
    compressed.bytes(data)?;
    let mut end = add(compressed.start, compressed.length)?;
    for (region, is_metadata) in [(metadata, true), (private, false)] {
        if region.start == 0 {
            if region.length != 0 || (is_metadata && metadata_original != 0) {
                return Err(invalid(InvalidKind::WoffHeader, 28));
            }
            continue;
        }
        if region.start != padded(end)? || region.length == 0 {
            return Err(invalid(InvalidKind::WoffHeader, region.start));
        }
        if data
            .get(end..region.start)
            .is_none_or(|padding| padding.iter().any(|byte| *byte != 0))
        {
            return Err(invalid(InvalidKind::WoffHeader, end));
        }
        region.bytes(data)?;
        end = add(region.start, region.length)?;
    }
    // The private block ends exactly at EOF. Other final blocks may have
    // optional null padding to the next four-byte boundary.
    if private.start != 0 {
        if end != data.len() {
            return Err(invalid(InvalidKind::WoffHeader, end));
        }
    } else if data.len() < end
        || data.len() > padded(end)?
        || data[end..].iter().any(|byte| *byte != 0)
    {
        return Err(invalid(InvalidKind::WoffHeader, end));
    }
    Ok(())
}

fn find(face: Face, indices: &[usize], tables: &[Table], tag: &[u8; 4]) -> Option<usize> {
    indices[face.first..face.first + face.count]
        .iter()
        .copied()
        .find(|index| tables[*index].tag == *tag)
}
fn validate_pairs(
    faces: &[Face],
    indices: &[usize],
    tables: &mut [Table],
    collection: bool,
) -> Result<(), DecodeError> {
    for face in faces {
        match (
            find(*face, indices, tables, b"glyf"),
            find(*face, indices, tables, b"loca"),
        ) {
            (Some(glyf), Some(loca)) => {
                if glyf >= loca
                    || (collection && loca != glyf + 1)
                    || tables[glyf].transformed != tables[loca].transformed
                {
                    return Err(invalid(InvalidKind::WoffDirectory, tables[loca].start));
                }
                // One partner per unique table prevents partial or crossed
                // sharing without a quadratic scan over collection faces.
                if tables[glyf].partner.is_some_and(|partner| partner != loca)
                    || tables[loca].partner.is_some_and(|partner| partner != glyf)
                {
                    return Err(invalid(InvalidKind::WoffDirectory, tables[loca].start));
                }
                tables[glyf].partner = Some(loca);
                tables[loca].partner = Some(glyf);
            }
            (None, None) => {}
            _ => return Err(invalid(InvalidKind::WoffDirectory, 0)),
        }
    }
    Ok(())
}

fn append_table(
    output: &mut Buffer<'_, u8>,
    bytes: &[u8],
    budget: &Budget,
) -> Result<Region, DecodeError> {
    table_limit(bytes.len(), budget)?;
    let start = output.len();
    let end = padded(add(start, bytes.len())?)?;
    check(LimitKind::DecodedBytes, end, budget.limits.decoded_bytes())?;
    as_u32(end)?;
    output.reserve(end)?;
    output.extend(bytes)?;
    output.resize(end, 0)?;
    Ok(Region {
        start,
        length: bytes.len(),
    })
}

fn reconstruct(
    budget: &Budget,
    stream: &[u8],
    version: u32,
    faces: &mut [Face],
    indices: &[usize],
    tables: &mut [Table],
) -> Result<Vec<u8>, DecodeError> {
    let mut header_size = if version == 0 {
        0
    } else {
        add(
            12,
            add(
                multiply(faces.len(), 4)?,
                if version == 0x0002_0000 { 12 } else { 0 },
            )?,
        )?
    };
    for face in faces.iter_mut() {
        face.header = header_size;
        header_size = add(header_size, add(12, multiply(face.count, 16)?)?)?;
    }
    check(
        LimitKind::DecodedBytes,
        header_size,
        budget.limits.decoded_bytes(),
    )?;
    let mut output = Buffer::output(budget);
    output.resize(header_size, 0)?;
    // Opaque tables are copied unchanged; only head.checkSumAdjustment is reset.
    for table in tables.iter_mut().filter(|table| !table.transformed) {
        let region = append_table(&mut output, table.bytes(stream)?, budget)?;
        if table.tag == *b"head" {
            if region.length < 12 {
                return Err(invalid(InvalidKind::WoffTransform, table.start));
            }
            write_u32(&mut output, region.start + 8, 0);
        }
        table.output = Some(region);
    }
    // Reconstruct pairs before any hmtx, irrespective of directory order and
    // sharing. Per-face metrics later read the actual reconstructed pair.
    for index in 0..tables.len() {
        if tables[index].tag != *b"glyf" || !tables[index].transformed {
            continue;
        }
        let face = faces
            .iter()
            .find(|face| find(**face, indices, tables, b"glyf") == Some(index))
            .ok_or_else(|| invalid(InvalidKind::WoffDirectory, tables[index].start))?;
        let loca = find(*face, indices, tables, b"loca")
            .ok_or_else(|| invalid(InvalidKind::WoffDirectory, 0))?;
        let decoded = glyph::reconstruct(
            tables[index].bytes(stream)?,
            tables[loca].original_length,
            budget,
        )?;
        tables[index].output = Some(append_table(&mut output, &decoded.glyf, budget)?);
        tables[loca].output = Some(append_table(&mut output, &decoded.loca, budget)?);
    }
    for face in faces.iter().copied() {
        let Some(index) = find(face, indices, tables, b"hmtx") else {
            continue;
        };
        if !tables[index].transformed {
            continue;
        }
        let metrics = metrics::reconstruct(face, indices, tables, stream, &output, budget)?;
        if let Some(region) = tables[index].output {
            if region.bytes(&output)? != &*metrics {
                return Err(invalid(InvalidKind::WoffMetrics, tables[index].start));
            }
        } else {
            tables[index].output = Some(append_table(&mut output, &metrics, budget)?);
        }
    }
    if tables.iter().any(|table| table.output.is_none()) {
        return Err(invalid(InvalidKind::WoffDirectory, 0));
    }
    if version != 0 {
        output[..4].copy_from_slice(b"ttcf");
        write_u32(&mut output, 4, version);
        write_u32(&mut output, 8, as_u32(faces.len())?);
        for (index, face) in faces.iter().enumerate() {
            write_u32(&mut output, 12 + index * 4, as_u32(face.header)?);
        }
    }
    for face in faces.iter().copied() {
        let base = face.header;
        output[base..base + 4].copy_from_slice(&face.flavor);
        let count = u16::try_from(face.count).map_err(|_| DecodeError::SizeOverflow)?;
        let exponent = 15 - count.leading_zeros() as u16;
        let search = (1u16 << exponent)
            .checked_mul(16)
            .ok_or(DecodeError::SizeOverflow)?;
        write_u16(&mut output, base + 4, count);
        write_u16(&mut output, base + 6, search);
        write_u16(&mut output, base + 8, exponent);
        let range = u16::try_from(face.count * 16).map_err(|_| DecodeError::SizeOverflow)?;
        write_u16(&mut output, base + 10, range - search);
        for (position, index) in indices[face.first..face.first + face.count]
            .iter()
            .copied()
            .enumerate()
        {
            let table = tables[index];
            let region = table
                .output
                .ok_or_else(|| invalid(InvalidKind::WoffDirectory, 0))?;
            let sum = checksum(region.bytes(&output)?);
            let record = base + 12 + position * 16;
            output[record..record + 4].copy_from_slice(&table.tag);
            write_u32(&mut output, record + 4, sum);
            write_u32(&mut output, record + 8, as_u32(region.start)?);
            write_u32(&mut output, record + 12, as_u32(region.length)?);
        }
    }
    if version == 0
        && let Some(head) = find(faces[0], indices, tables, b"head")
    {
        let region = tables[head]
            .output
            .ok_or_else(|| invalid(InvalidKind::WoffDirectory, 0))?;
        let adjustment = 0xb1b0_afbau32.wrapping_sub(checksum(&output));
        write_u32(&mut output, region.start + 8, adjustment);
    }
    Ok(output.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> DecodeLimits {
        DecodeLimits::new(
            1_000_000, 1_000_000, 1_000_000, 100, 10, 1_000_000, 3_000_000,
        )
    }
    fn transformed_glyph() -> Vec<u8> {
        // Two glyphs: a simple one-point contour at (5,6), then an empty
        // glyph. REC-WOFF2 5.2 flag 23 + byte 0x45 encodes (+5,+6).
        let mut data = vec![0, 0, 0, 0, 0, 2, 0, 0];
        for length in [4u32, 1, 1, 2, 0, 4, 0] {
            data.extend_from_slice(&length.to_be_bytes());
        }
        data.extend_from_slice(&[0, 1, 0, 0, 1, 23, 0x45, 0, 0, 0, 0, 0]);
        data
    }
    fn table(
        stream: &mut Vec<u8>,
        tag: [u8; 4],
        transformed: bool,
        original_length: usize,
        bytes: &[u8],
    ) -> Table {
        let start = stream.len();
        stream.extend_from_slice(bytes);
        Table {
            tag,
            transformed,
            original_length,
            start,
            length: bytes.len(),
            output: None,
            partner: None,
        }
    }
    fn record(bytes: &[u8], face: usize, tag: &[u8; 4]) -> Region {
        let count = u16::from_be_bytes(bytes[face + 4..face + 6].try_into().unwrap()) as usize;
        for index in 0..count {
            let offset = face + 12 + index * 16;
            if bytes[offset..offset + 4] == *tag {
                return Region {
                    start: u32::from_be_bytes(bytes[offset + 8..offset + 12].try_into().unwrap())
                        as usize,
                    length: u32::from_be_bytes(bytes[offset + 12..offset + 16].try_into().unwrap())
                        as usize,
                };
            }
        }
        panic!("expected table absent")
    }
    fn fixture(transformed: bool) -> (Vec<u8>, Vec<Table>, Vec<Face>, Vec<usize>) {
        let mut stream = Vec::new();
        let mut head = [0u8; 54];
        head[8..12].copy_from_slice(&123u32.to_be_bytes());
        let mut hhea = [0u8; 36];
        hhea[34..].copy_from_slice(&1u16.to_be_bytes());
        let maxp = [0, 1, 0, 0, 0, 2];
        let mut tables = Vec::new();
        if transformed {
            // Original glyf length is deliberately unrelated to the result.
            tables.push(table(
                &mut stream,
                *b"glyf",
                true,
                usize::MAX,
                &transformed_glyph(),
            ));
            tables.push(table(&mut stream, *b"loca", true, 6, &[]));
        } else {
            // SFNT record for a one-point simple glyph, padded to 20 bytes.
            let glyf = [
                0, 1, 0, 5, 0, 6, 0, 5, 0, 6, 0, 0, 0, 0, 0x37, 5, 6, 0, 0, 0,
            ];
            tables.push(table(&mut stream, *b"glyf", false, 20, &glyf));
            tables.push(table(
                &mut stream,
                *b"loca",
                false,
                6,
                &[0, 0, 0, 10, 0, 10],
            ));
        }
        tables.push(table(&mut stream, *b"head", false, 54, &head));
        tables.push(table(&mut stream, *b"hhea", false, 36, &hhea));
        // Both bearing arrays omitted; two independently transformed metrics
        // tables share the same outlines but have different advance widths.
        tables.push(table(&mut stream, *b"hmtx", true, usize::MAX, &[3, 1, 244]));
        tables.push(table(&mut stream, *b"maxp", false, 6, &maxp));
        tables.push(table(&mut stream, *b"hmtx", true, 0, &[3, 2, 88]));
        let faces = vec![
            Face {
                flavor: [0, 1, 0, 0],
                first: 0,
                count: 6,
                header: 0,
            },
            Face {
                flavor: [0, 1, 0, 0],
                first: 6,
                count: 6,
                header: 0,
            },
        ];
        let indices = vec![0, 2, 3, 4, 1, 5, 0, 2, 3, 6, 1, 5];
        (stream, tables, faces, indices)
    }

    #[test]
    fn base128_rejects_noncanonical_and_overflow_values() {
        for bytes in [
            &[0x80, 0][..],
            &[0xff, 0xff, 0xff, 0xff, 0x7f],
            &[0x81, 0x80, 0x80, 0x80, 0x80, 0],
        ] {
            assert!(
                Reader::new(bytes, InvalidKind::WoffDirectory)
                    .base128()
                    .is_err()
            );
        }
        assert_eq!(
            Reader::new(&[0x8f, 0xff, 0xff, 0xff, 0x7f], InvalidKind::WoffDirectory)
                .base128()
                .unwrap(),
            u32::MAX as usize
        );
    }
    #[test]
    fn uint255_accepts_all_specified_encodings_of_506() {
        for bytes in [&[255, 253][..], &[254, 0], &[253, 1, 250]] {
            assert_eq!(
                Reader::new(bytes, InvalidKind::WoffDirectory)
                    .uint255()
                    .unwrap(),
                506
            );
        }
    }
    #[test]
    fn transformed_simple_outline_and_empty_glyph_have_derived_loca_and_bbox() {
        let budget = Budget::new(limits(), 0).unwrap();
        let result = glyph::reconstruct(&transformed_glyph(), 6, &budget).unwrap();
        assert_eq!(
            &*result.glyf,
            &[0, 1, 0, 5, 0, 6, 0, 5, 0, 6, 0, 0, 0, 0, 0x37, 5, 6, 0]
        );
        // 14 header/end-point/instruction bytes + one flag + x/y bytes = 17;
        // word alignment adds one byte, and short loca stores 18 / 2 = 9.
        assert_eq!(&*result.loca, &[0, 0, 0, 9, 0, 9]);
    }
    #[test]
    fn extra_transform_substream_bytes_are_rejected() {
        let budget = Budget::new(limits(), 0).unwrap();
        let mut bytes = transformed_glyph();
        bytes[35] = 1;
        bytes.push(0); // instruction stream has an unused byte.
        assert!(glyph::reconstruct(&bytes, 6, &budget).is_err());
    }
    #[test]
    fn transformed_loca_reference_length_must_equal_calculated_length() {
        let budget = Budget::new(limits(), 0).unwrap();
        assert!(glyph::reconstruct(&transformed_glyph(), 8, &budget).is_err());
    }
    #[test]
    fn overlap_bitmap_sets_the_first_stored_simple_flag() {
        let budget = Budget::new(limits(), 0).unwrap();
        let mut bytes = transformed_glyph();
        bytes[3] = 1;
        bytes.push(0x80);
        let result = glyph::reconstruct(&bytes, 6, &budget).unwrap();
        assert_eq!(result.glyf[14], 0x77);
    }
    #[test]
    fn composite_components_bbox_and_instructions_are_reconstructed() {
        let budget = Budget::new(limits(), 0).unwrap();
        let mut bytes = vec![0, 0, 0, 0, 0, 1, 0, 1];
        for length in [2u32, 0, 0, 1, 6, 12, 2] {
            bytes.extend_from_slice(&length.to_be_bytes());
        }
        // Composite: glyph 0 with two byte arguments, WE_HAVE_INSTRUCTIONS;
        // explicit bbox (-1,-2,3,4), then two instruction bytes.
        bytes.extend_from_slice(&[
            0xff, 0xff, 2, 1, 0, 0, 0, 5, 6, 0x80, 0, 0, 0, 0xff, 0xff, 0xff, 0xfe, 0, 3, 0, 4,
            0xaa, 0xbb,
        ]);
        let result = glyph::reconstruct(&bytes, 8, &budget).unwrap();
        assert_eq!(
            &*result.glyf,
            &[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0, 3, 0, 4, 1, 0, 0, 0, 5, 6, 0, 2, 0xaa, 0xbb
            ]
        );
        assert_eq!(&*result.loca, &[0, 0, 0, 0, 0, 0, 0, 20]);
    }
    #[test]
    fn shared_collection_outlines_reconstruct_independent_metrics_and_face_order() {
        let budget = Budget::new(limits(), 0).unwrap();
        let (stream, mut tables, mut faces, indices) = fixture(true);
        validate_pairs(&faces, &indices, &mut tables, true).unwrap();
        let result = reconstruct(
            &budget,
            &stream,
            0x0002_0000,
            &mut faces,
            &indices,
            &mut tables,
        )
        .unwrap();
        assert_eq!(&result[..12], b"ttcf\x00\x02\x00\x00\x00\x00\x00\x02");
        assert_eq!(&result[20..32], &[0; 12]); // version 2 DSIG fields.
        assert_eq!(
            record(&result, faces[0].header, b"glyf").start,
            record(&result, faces[1].header, b"glyf").start
        );
        assert_eq!(
            record(&result, faces[0].header, b"loca").start,
            record(&result, faces[1].header, b"loca").start
        );
        assert_eq!(
            record(&result, faces[0].header, b"hmtx")
                .bytes(&result)
                .unwrap(),
            &[1, 244, 0, 5, 0, 0]
        );
        assert_eq!(
            record(&result, faces[1].header, b"hmtx")
                .bytes(&result)
                .unwrap(),
            &[2, 88, 0, 5, 0, 0]
        );
    }
    #[test]
    fn null_transformed_outlines_supply_inferred_metrics() {
        let budget = Budget::new(limits(), 0).unwrap();
        let (stream, mut tables, mut faces, indices) = fixture(false);
        let result = reconstruct(
            &budget,
            &stream,
            0x0001_0000,
            &mut faces,
            &indices,
            &mut tables,
        )
        .unwrap();
        assert_eq!(
            record(&result, faces[1].header, b"hmtx")
                .bytes(&result)
                .unwrap(),
            &[2, 88, 0, 5, 0, 0]
        );
    }
    #[test]
    fn shared_metrics_keep_the_original_shared_table_association() {
        let budget = Budget::new(limits(), 0).unwrap();
        let (stream, mut tables, mut faces, mut indices) = fixture(true);
        tables.pop();
        indices[9] = 4;
        let result = reconstruct(
            &budget,
            &stream,
            0x0001_0000,
            &mut faces,
            &indices,
            &mut tables,
        )
        .unwrap();
        assert_eq!(
            record(&result, faces[0].header, b"hmtx").start,
            record(&result, faces[1].header, b"hmtx").start
        );
        assert_eq!(
            record(&result, faces[1].header, b"hmtx")
                .bytes(&result)
                .unwrap(),
            &[1, 244, 0, 5, 0, 0]
        );
    }
    #[test]
    fn invalid_metrics_flags_are_rejected() {
        for flags in [0, 4, 0xff] {
            let budget = Budget::new(limits(), 0).unwrap();
            let (mut stream, mut tables, mut faces, indices) = fixture(true);
            stream[tables[4].start] = flags;
            assert!(
                matches!(reconstruct(&budget, &stream, 0x0001_0000, &mut faces, &indices, &mut tables), Err(DecodeError::InvalidContainer(error)) if error.kind() == InvalidKind::WoffMetrics)
            );
        }
    }
    #[test]
    fn standalone_checksum_is_recomputed_and_opaque_tables_are_preserved() {
        let budget = Budget::new(limits(), 0).unwrap();
        let mut stream = Vec::new();
        let mut head = [0u8; 54];
        head[8..12].copy_from_slice(&123u32.to_be_bytes());
        let opaque = b"variation/color/shaping payload";
        let mut tables = [
            table(&mut stream, *b"head", false, 54, &head),
            table(&mut stream, *b"gvar", false, opaque.len(), opaque),
        ];
        let mut faces = [Face {
            flavor: *b"OTTO",
            first: 0,
            count: 2,
            header: 0,
        }];
        let result = reconstruct(&budget, &stream, 0, &mut faces, &[1, 0], &mut tables).unwrap();
        assert_eq!(checksum(&result), 0xb1b0_afba);
        assert_eq!(record(&result, 0, b"gvar").bytes(&result).unwrap(), opaque);
        let head = record(&result, 0, b"head");
        let mut normalized_head = head.bytes(&result).unwrap().to_vec();
        normalized_head[8..12].fill(0);
        assert_eq!(
            u32::from_be_bytes(result[32..36].try_into().unwrap()),
            checksum(&normalized_head)
        );
    }
    #[test]
    fn partial_outline_pair_sharing_is_rejected() {
        let (_, mut tables, faces, mut indices) = fixture(true);
        let mut second_loca = tables[1];
        second_loca.partner = None;
        tables.push(second_loca);
        indices[10] = 7;
        assert!(validate_pairs(&faces, &indices, &mut tables, true).is_err());
    }
    #[test]
    fn output_and_workspace_quotas_reject_before_buffer_growth() {
        let policy = DecodeLimits::new(100, 100, 100, 10, 10, 4, 100);
        let budget = Budget::new(policy, 0).unwrap();
        let mut buffer = Buffer::new(&budget);
        buffer.extend(&[1u8; 4]).unwrap();
        assert!(matches!(
            buffer.push(5),
            Err(DecodeError::LimitExceeded {
                kind: LimitKind::WorkspaceBytes,
                ..
            })
        ));
        assert_eq!(&*buffer, &[1; 4]);
        assert_eq!(budget.usage(), (4, 4));
    }
    #[test]
    fn metadata_and_private_ranges_require_order_and_null_alignment_padding() {
        let compressed = Region {
            start: 0,
            length: 1,
        };
        let metadata = Region {
            start: 4,
            length: 1,
        };
        let private = Region {
            start: 8,
            length: 1,
        };
        let mut bytes = [1, 0, 0, 0, 2, 0, 0, 0, 3];
        validate_blocks(&bytes, compressed, metadata, 1, private).unwrap();
        bytes[2] = 1;
        assert!(validate_blocks(&bytes, compressed, metadata, 1, private).is_err());
    }
}

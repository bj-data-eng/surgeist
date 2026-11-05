use std::mem::size_of;

use crate::{DecodeError, DecodeLimits, InvalidKind, LimitKind, add, check, invalid, multiply};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Kind {
    Header,
    Directory,
    Table,
    Signature,
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Region {
    start: usize,
    end: usize,
    kind: Kind,
    tag: [u8; 4],
    face: usize,
}

struct Container {
    faces: usize,
    header_end: usize,
    signature: Option<(usize, usize)>,
}

pub(super) fn admit(
    bytes: &[u8],
    limits: DecodeLimits,
    base: usize,
) -> Result<Vec<usize>, DecodeError> {
    let container = container(bytes)?;
    check(LimitKind::Faces, container.faces, limits.faces())?;
    let mut tables = 0usize;
    // First pass proves lengths/counts and computes all allocation requests.
    // No storage grows in response to an unvalidated input count.
    for face in 0..container.faces {
        let offset = face_offset(bytes, &container, face)?;
        let count = directory(bytes, offset, limits, tables)?;
        tables = add(tables, count)?;
        check(LimitKind::Tables, tables, limits.tables())?;
    }
    let retained = multiply(container.faces, size_of::<usize>())?;
    let region_count = add(
        add(tables, container.faces)?,
        1 + usize::from(container.signature.is_some()),
    )?;
    let workspace = multiply(region_count, size_of::<Region>())?;
    check(
        LimitKind::WorkspaceBytes,
        workspace,
        limits.workspace_bytes(),
    )?;
    check(
        LimitKind::LiveBytes,
        add(base, add(retained, workspace)?)?,
        limits.live_bytes(),
    )?;
    let mut offsets = Vec::new();
    offsets
        .try_reserve_exact(container.faces)
        .map_err(|_| DecodeError::AllocationFailed)?;
    let mut regions = Vec::new();
    regions
        .try_reserve_exact(region_count)
        .map_err(|_| DecodeError::AllocationFailed)?;
    if container.header_end != 0 {
        regions.push(Region {
            start: 0,
            end: container.header_end,
            kind: Kind::Header,
            tag: [0; 4],
            face: 0,
        });
    }
    if let Some((start, end)) = container.signature {
        regions.push(Region {
            start,
            end,
            kind: Kind::Signature,
            tag: *b"DSIG",
            face: 0,
        });
    }
    for face in 0..container.faces {
        let offset = face_offset(bytes, &container, face)?;
        offsets.push(offset);
        let count = usize::from(u16_at(bytes, offset + 4)?);
        let end = add(add(offset, 12)?, multiply(count, 16)?)?;
        regions.push(Region {
            start: offset,
            end,
            kind: Kind::Directory,
            tag: [0; 4],
            face,
        });
        for table in 0..count {
            let record = offset + 12 + table * 16;
            let start = offset_at(bytes, record + 8)?;
            let length = offset_at(bytes, record + 12)?;
            if length != 0 {
                regions.push(Region {
                    start,
                    end: add(start, length)?,
                    kind: Kind::Table,
                    tag: bytes[record..record + 4]
                        .try_into()
                        .expect("admitted table record"),
                    face,
                });
            }
        }
    }
    regions.sort_unstable();
    for pair in regions.windows(2) {
        let [left, right] = pair else { unreachable!() };
        if left.end > right.start {
            let shared = left.start == right.start
                && left.end == right.end
                && left.kind == right.kind
                && left.face != right.face
                && (left.kind == Kind::Directory
                    || (left.kind == Kind::Table && left.tag == right.tag));
            if !shared {
                return Err(invalid(InvalidKind::Overlap, right.start));
            }
        }
    }
    Ok(offsets)
}

fn container(bytes: &[u8]) -> Result<Container, DecodeError> {
    let signature = span(bytes, 0, 4)?;
    if signature != b"ttcf" {
        scaler(bytes, 0)?;
        return Ok(Container {
            faces: 1,
            header_end: 0,
            signature: None,
        });
    }
    let version = u32_at(bytes, 4)?;
    if version != 0x0001_0000 && version != 0x0002_0000 {
        return Err(invalid(InvalidKind::CollectionVersion, 4));
    }
    let faces = offset_at(bytes, 8)?;
    if faces == 0 {
        return Err(invalid(InvalidKind::EmptyCollection, 8));
    }
    let offsets_end = add(12, multiply(faces, 4)?)?;
    let header_end = add(offsets_end, if version == 0x0002_0000 { 12 } else { 0 })?;
    span(bytes, 0, header_end)?;
    let signature = if version == 0x0002_0000 {
        let tag = u32_at(bytes, offsets_end)?;
        let length = offset_at(bytes, offsets_end + 4)?;
        let offset = offset_at(bytes, offsets_end + 8)?;
        if tag == 0 && length == 0 && offset == 0 {
            None
        } else if tag == u32::from_be_bytes(*b"DSIG") && length != 0 {
            aligned(offset)?;
            span(bytes, offset, length)?;
            let end = add(offset, length)?;
            if end != bytes.len() {
                return Err(invalid(InvalidKind::CollectionSignature, offsets_end));
            }
            Some((offset, end))
        } else {
            return Err(invalid(InvalidKind::CollectionSignature, offsets_end));
        }
    } else {
        None
    };
    Ok(Container {
        faces,
        header_end,
        signature,
    })
}

fn face_offset(bytes: &[u8], container: &Container, face: usize) -> Result<usize, DecodeError> {
    if container.header_end == 0 {
        Ok(0)
    } else {
        offset_at(bytes, add(12, multiply(face, 4)?)?)
    }
}

fn directory(
    bytes: &[u8],
    offset: usize,
    limits: DecodeLimits,
    previous_tables: usize,
) -> Result<usize, DecodeError> {
    aligned(offset)?;
    scaler(bytes, offset)?;
    let count = usize::from(u16_at(bytes, add(offset, 4)?)?);
    check(
        LimitKind::Tables,
        add(previous_tables, count)?,
        limits.tables(),
    )?;
    let length = add(12, multiply(count, 16)?)?;
    span(bytes, offset, length)?;
    let mut previous: Option<&[u8]> = None;
    for table in 0..count {
        let record = add(add(offset, 12)?, multiply(table, 16)?)?;
        let tag = &bytes[record..record + 4];
        if !valid_tag(tag) {
            return Err(invalid(InvalidKind::TableTag, record));
        }
        if previous.is_some_and(|previous| previous >= tag) {
            return Err(invalid(InvalidKind::TableOrder, record));
        }
        previous = Some(tag);
        let start = offset_at(bytes, record + 8)?;
        let length = offset_at(bytes, record + 12)?;
        aligned(start)?;
        check(LimitKind::TableBytes, length, limits.table_bytes())?;
        span(bytes, start, length)?;
    }
    Ok(count)
}

fn valid_tag(tag: &[u8]) -> bool {
    let mut space = false;
    let mut non_space = false;
    for &byte in tag {
        if !(0x20..=0x7e).contains(&byte) {
            return false;
        }
        if byte == b' ' {
            space = true;
        } else if space {
            return false;
        } else {
            non_space = true;
        }
    }
    non_space
}

fn scaler(bytes: &[u8], offset: usize) -> Result<(), DecodeError> {
    match span(bytes, offset, 4)? {
        b"\0\x01\0\0" | b"OTTO" | b"true" | b"typ1" => Ok(()),
        _ => Err(invalid(InvalidKind::Signature, offset)),
    }
}

fn aligned(offset: usize) -> Result<(), DecodeError> {
    if offset.is_multiple_of(4) {
        Ok(())
    } else {
        Err(invalid(InvalidKind::UnalignedOffset, offset))
    }
}

fn span(bytes: &[u8], offset: usize, length: usize) -> Result<&[u8], DecodeError> {
    let end = add(offset, length)?;
    bytes
        .get(offset..end)
        .ok_or_else(|| invalid(InvalidKind::Truncated, offset))
}

pub(super) fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, DecodeError> {
    Ok(u16::from_be_bytes(
        span(bytes, offset, 2)?.try_into().expect("two-byte span"),
    ))
}
fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, DecodeError> {
    Ok(u32::from_be_bytes(
        span(bytes, offset, 4)?.try_into().expect("four-byte span"),
    ))
}
pub(super) fn offset_at(bytes: &[u8], offset: usize) -> Result<usize, DecodeError> {
    usize::try_from(u32_at(bytes, offset)?).map_err(|_| DecodeError::SizeOverflow)
}

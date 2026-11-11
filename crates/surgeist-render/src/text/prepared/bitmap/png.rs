//! Owned narrow PNG decoding, authored from W3C PNG Third Edition,
//! 24 June 2025, sections 5, 9–11: <https://www.w3.org/TR/2025/REC-png-3-20250624/>.
//! Only IHDR, contiguous IDAT and IEND, noninterlaced RGBA8 are admitted.
//! Other valid profiles/ancillary semantics refuse rather than being ignored.
//! miniz_oxide 0.8.9's safe low-level core owns fixed inline state, not an
//! allocating stream decoder. Its state and source-declared Huffman temporary
//! arrays are charged, alongside every caller-owned buffer capacity. Ordinary
//! scalar call-stack/compiler temporaries are not an allocator-storage promise.

use super::*;
use miniz_oxide::inflate::{
    TINFLStatus,
    core::{DecompressorOxide, decompress, inflate_flags},
};

pub(super) struct Decoded {
    pub width: u32,
    pub height: u32,
    pub pixels: AccountedVec<u8>,
}

struct Header {
    width: u32,
    height: u32,
    row: usize,
    len: usize,
    filtered: usize,
    idat: usize,
}

fn overflow() -> Error {
    GlyphPreparationError::CapacityOverflow.into()
}

fn sum(values: &[usize]) -> Result<usize> {
    values
        .iter()
        .try_fold(0usize, |a, b| a.checked_add(*b).ok_or_else(overflow))
}

fn crc(bytes: &[u8]) -> u32 {
    // PNG's reflected polynomial, initialized/finalized by all-one complement;
    // the length is excluded. Eight fixed steps per source byte, no table owner.
    let mut value = u32::MAX;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            value = (value >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(value & 1));
        }
    }
    !value
}

fn number(bytes: &[u8]) -> Result<u32> {
    Ok(u32::from_be_bytes(bytes.try_into().map_err(|_| invalid())?))
}

struct Chunk<'a> {
    kind: &'a [u8],
    data: &'a [u8],
}

fn chunk<'a>(data: &'a [u8], at: &mut usize, account: &Account) -> Result<Chunk<'a>> {
    let start = *at;
    let prefix = data
        .get(start..start.checked_add(8).ok_or_else(overflow)?)
        .ok_or_else(invalid)?;
    let len = number(&prefix[..4])?;
    if len > 0x7fff_ffff
        || !prefix[4..].iter().all(u8::is_ascii_alphabetic)
        || !prefix[6].is_ascii_uppercase()
    {
        return Err(invalid());
    }
    let len = usize::try_from(len).map_err(|_| overflow())?;
    let end = sum(&[start, 8, len])?;
    let next = end.checked_add(4).ok_or_else(overflow)?;
    let bytes = data.get(start + 4..end).ok_or_else(invalid)?;
    account
        .at(RenderPhase::Png)
        .work(RenderWork::PngByteVisits, bytes.len())?;
    if crc(bytes) != number(data.get(end..next).ok_or_else(invalid)?)? {
        return Err(invalid());
    }
    *at = next;
    Ok(Chunk {
        kind: &prefix[4..],
        data: &data[start + 8..end],
    })
}

fn header(data: &[u8], account: &Account) -> Result<Header> {
    if data.get(..8) != Some(b"\x89PNG\r\n\x1a\n") {
        return Err(invalid());
    }
    let mut at = 8;
    let first = chunk(data, &mut at, account)?;
    if first.kind != b"IHDR" || first.data.len() != 13 {
        return Err(invalid());
    }
    let width = number(&first.data[..4])?;
    let height = number(&first.data[4..8])?;
    if width == 0
        || height == 0
        || width > 0x7fff_ffff
        || height > 0x7fff_ffff
        || first.data[10] != 0
        || first.data[11] != 0
        || first.data[12] > 1
    {
        return Err(invalid());
    }
    if first.data[8] != 8 || first.data[9] != 6 || first.data[12] != 0 {
        return Err(GlyphPreparationError::UnsupportedPng.into());
    }
    let row = usize::try_from(width)
        .map_err(|_| overflow())?
        .checked_mul(4)
        .ok_or_else(overflow)?;
    let rows = usize::try_from(height).map_err(|_| overflow())?;
    let len = row.checked_mul(rows).ok_or_else(overflow)?;
    let filtered = row
        .checked_add(1)
        .and_then(|n| n.checked_mul(rows))
        .ok_or_else(overflow)?;
    let mut idat = 0usize;
    let mut seen = false;
    loop {
        let next = chunk(data, &mut at, account)?;
        match next.kind {
            b"IDAT" => {
                seen = true;
                idat = idat.checked_add(next.data.len()).ok_or_else(overflow)?;
            }
            b"IEND" if seen && next.data.is_empty() && at == data.len() => break,
            b"IEND" | b"IHDR" => return Err(invalid()),
            // Requiring this closed chunk grammar also rules out noncontiguous
            // IDAT, APNG, palette/transparency and unexecuted color profiles.
            _ => return Err(GlyphPreparationError::UnsupportedPng.into()),
        }
    }
    Ok(Header {
        width,
        height,
        row,
        len,
        filtered,
        idat,
    })
}

pub(super) fn decode(data: &[u8], account: &Account) -> Result<Decoded> {
    let h = header(data, account)?;
    // core.rs init_tree's only temporary arrays: total_symbols and next_code.
    let fixed = sum(&[
        size_of::<DecompressorOxide>(),
        size_of::<[u16; 16]>(),
        size_of::<[u32; 17]>(),
    ])?;
    account.preview(
        RenderResource::ScratchCpuBytes,
        sum(&[fixed, h.idat, h.filtered])?,
    )?;
    let mut compressed = account.temporary::<u8>(h.idat)?;
    let mut pixels = account.temporary::<u8>(h.filtered)?;
    let mut at = 8;
    // This second bounded pass copies only already-validated IDAT payloads.
    while at < data.len() {
        let c = chunk(data, &mut at, account)?;
        if c.kind == b"IDAT" {
            account
                .at(RenderPhase::Png)
                .work(RenderWork::SerializedBytes, c.data.len())?;
            compressed.extend_from_slice(c.data)?;
        }
    }
    pixels.resize(h.filtered, 0)?;
    {
        let _workspace = account.acquire(RenderResource::ScratchCpuBytes, fixed)?;
        let mut state = DecompressorOxide::new();
        let (status, consumed, produced) = decompress(
            &mut state,
            &compressed,
            &mut pixels,
            0,
            inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
                | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF,
        );
        // The core verifies the zlib header and Adler checksum. Neither trailing
        // compressed bytes nor a shorter/longer image stream is accepted.
        if status != TINFLStatus::Done || consumed != compressed.len() || produced != h.filtered {
            return Err(invalid());
        }
    }
    drop(compressed);
    unfilter(&mut pixels, h.row, account)?;
    pixels.truncate(h.len);
    Ok(Decoded {
        width: h.width,
        height: h.height,
        pixels,
    })
}

enum ScanlineFilter {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

impl TryFrom<u8> for ScanlineFilter {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Sub),
            2 => Ok(Self::Up),
            3 => Ok(Self::Average),
            4 => Ok(Self::Paeth),
            _ => Err(invalid()),
        }
    }
}

fn unfilter(bytes: &mut [u8], row: usize, account: &Account) -> Result<()> {
    let stride = row.checked_add(1).ok_or_else(overflow)?;
    for y in 0..bytes.len() / stride {
        let from = y * stride;
        let to = y * row;
        let filter = ScanlineFilter::try_from(bytes[from])?;
        for x in 0..row {
            account
                .at(RenderPhase::Png)
                .work(RenderWork::PngByteVisits, 1)?;
            let left = if x >= 4 { bytes[to + x - 4] } else { 0 };
            let up = if y > 0 { bytes[to + x - row] } else { 0 };
            let corner = if x >= 4 && y > 0 {
                bytes[to + x - row - 4]
            } else {
                0
            };
            let predictor = match filter {
                ScanlineFilter::None => 0,
                ScanlineFilter::Sub => left,
                ScanlineFilter::Up => up,
                ScanlineFilter::Average => ((u16::from(left) + u16::from(up)) / 2) as u8,
                ScanlineFilter::Paeth => paeth(left, up, corner),
            };
            bytes[to + x] = bytes[from + 1 + x].wrapping_add(predictor);
        }
    }
    Ok(())
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = i16::from(a) + i16::from(b) - i16::from(c);
    let da = (p - i16::from(a)).abs();
    let db = (p - i16::from(b)).abs();
    let dc = (p - i16::from(c)).abs();
    if da <= db && da <= dc {
        a
    } else if db <= dc {
        b
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIXELS: [u8; 16] = [
        10, 20, 30, 255, 40, 50, 60, 255, 50, 60, 70, 255, 80, 90, 100, 255,
    ];
    // Independent literal filter bytes, including first-row/left-edge zeros,
    // Average's wide sum, modulo reconstruction and Paeth's previous row.
    const FILTERED: [[u8; 18]; 5] = [
        [
            0, 10, 20, 30, 255, 40, 50, 60, 255, 0, 50, 60, 70, 255, 80, 90, 100, 255,
        ],
        [
            1, 10, 20, 30, 255, 30, 30, 30, 0, 1, 50, 60, 70, 255, 30, 30, 30, 0,
        ],
        [
            2, 10, 20, 30, 255, 40, 50, 60, 255, 2, 40, 40, 40, 0, 40, 40, 40, 0,
        ],
        [
            3, 10, 20, 30, 255, 35, 40, 45, 128, 3, 45, 50, 55, 128, 35, 35, 35, 0,
        ],
        [
            4, 10, 20, 30, 255, 30, 30, 30, 0, 4, 40, 40, 40, 0, 30, 30, 30, 0,
        ],
    ];

    fn zlib(bytes: &[u8]) -> Vec<u8> {
        // A single final uncompressed DEFLATE block. The checksum is of the
        // literal filtered data, independently checked by the existing decoder.
        let n = u16::try_from(bytes.len()).unwrap();
        let mut out = vec![0x78, 0x01, 0x01];
        out.extend(n.to_le_bytes());
        out.extend((!n).to_le_bytes());
        out.extend(bytes);
        let (mut a, mut b) = (1u32, 0u32);
        for x in bytes {
            a = (a + u32::from(*x)) % 65521;
            b = (b + a) % 65521;
        }
        out.extend(((b << 16) | a).to_be_bytes());
        out
    }

    fn append(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend(u32::try_from(data.len()).unwrap().to_be_bytes());
        let start = out.len();
        out.extend(kind);
        out.extend(data);
        out.extend(crc(&out[start..]).to_be_bytes());
    }

    fn image(stream: &[u8], split: bool) -> Vec<u8> {
        image_with_height(stream, split, 2)
    }

    fn image_with_height(stream: &[u8], split: bool, height: u8) -> Vec<u8> {
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        append(
            &mut out,
            b"IHDR",
            &[0, 0, 0, 2, 0, 0, 0, height, 8, 6, 0, 0, 0],
        );
        if split {
            append(&mut out, b"IDAT", &stream[..3]);
            append(&mut out, b"IDAT", &[]);
            append(&mut out, b"IDAT", &stream[3..]);
        } else {
            append(&mut out, b"IDAT", stream);
        }
        append(&mut out, b"IEND", &[]);
        out
    }

    struct TestDecoded {
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    }

    #[test]
    fn corrupt_deflate_reports_accepted_scratch_then_releases_it_without_transfer() {
        let mut stream = zlib(&FILTERED[0]);
        *stream.last_mut().unwrap() ^= 1; // wrong Adler; enclosing chunk CRC remains valid
        let bytes = image(&stream, false);
        let mut context = crate::RenderAccountingContext::default();
        let result: Result<()> = context.prepare(
            crate::RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(0, 0, 1 << 20, 0),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                decode(&bytes, account)?;
                Ok(())
            },
        );
        assert!(result.is_err());
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(
            report.status(),
            crate::RenderAttemptStatus::Failed(crate::ErrorCode::GlyphPreparation)
        );
        assert!(report.usage().peak(RenderResource::ScratchCpuBytes) > 0);
        assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
    }
    fn test_decode(data: &[u8], scratch: usize) -> Result<TestDecoded> {
        let mut context = RenderAccountingContext::default();
        decode_in(data, scratch, &mut context)
    }

    fn decode_in(
        data: &[u8],
        scratch: usize,
        context: &mut RenderAccountingContext,
    ) -> Result<TestDecoded> {
        context.prepare(
            RenderAttemptKind::GlyphRun,
            GlyphRunLimits::new(1, 0, scratch, 1 << 20),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                let decoded = decode(data, account)?;
                Ok(TestDecoded {
                    width: decoded.width,
                    height: decoded.height,
                    pixels: decoded.pixels.as_slice().to_vec(),
                })
            },
        )
    }

    #[test]
    fn mixed_scanlines_preserve_pixels_and_the_accepted_work_prefix() {
        use crate::{RenderAccountingOptions, RenderLimits};

        let filtered = [
            1, 10, 20, 30, 255, 30, 30, 30, 0, 2, 40, 40, 40, 0, 40, 40, 40, 0, 4, 50, 50, 50, 0,
            30, 30, 30, 0,
        ];
        let expected = [
            10, 20, 30, 255, 40, 50, 60, 255, 50, 60, 70, 255, 80, 90, 100, 255, 100, 110, 120,
            255, 130, 140, 150, 255,
        ];
        for (split, exact) in [(false, 150), (true, 166)] {
            let bytes = image_with_height(&zlib(&filtered), split, 3);
            let mut malformed = filtered;
            malformed[18] = 5;
            let malformed = image_with_height(&zlib(&malformed), split, 3);
            for limit in [exact - 1, exact] {
                let mut context = RenderAccountingContext::new(
                    RenderAccountingOptions::new().with_limits(
                        RenderLimits::new()
                            .with_work_limit(RenderWork::PngByteVisits, limit)
                            .unwrap(),
                    ),
                );

                let error = decode_in(&malformed, 1 << 20, &mut context).err().unwrap();
                assert_eq!(
                    error.glyph_preparation_diagnostic(),
                    Some(&GlyphPreparationError::InvalidFont)
                );
                // The third tag refuses before visiting its eight payload bytes.
                assert_scanline_usage(&context, exact - 8);

                let result = decode_in(&bytes, 1 << 20, &mut context);
                if limit == exact {
                    let decoded = result.ok().unwrap();
                    assert_eq!((decoded.width, decoded.height), (2, 3));
                    assert_eq!(decoded.pixels, expected);
                } else {
                    assert!(result.err().unwrap().accounting_diagnostic().is_some());
                }
                assert_scanline_usage(&context, limit);

                let smaller = image(&zlib(&FILTERED[0]), split);
                assert_eq!(
                    decode_in(&smaller, 1 << 20, &mut context).unwrap().pixels,
                    PIXELS
                );
            }
        }
    }

    fn assert_scanline_usage(context: &RenderAccountingContext, expected_visits: u64) {
        let terminal = context.last_attempt().unwrap();
        assert_eq!(terminal.cleanup_fault(), None);
        let usage = terminal.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::PngByteVisits), expected_visits);
        assert!(usage.peak(RenderResource::ScratchCpuBytes) > 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
    }

    #[test]
    fn unknown_scanline_tags_refuse_at_the_row_boundary_and_release_decode_storage() {
        for tag in 5..=u8::MAX {
            for (row_start, row_visits) in [(0, 0), (9, 8)] {
                for split in [false, true] {
                    let mut filtered = FILTERED[0];
                    filtered[row_start] = tag;
                    let bytes = image(&zlib(&filtered), split);
                    let original = bytes.clone();
                    let mut context = RenderAccountingContext::default();
                    for _ in 0..2 {
                        let error = decode_in(&bytes, 1 << 20, &mut context).err().unwrap();
                        assert_eq!(
                            error.glyph_preparation_diagnostic(),
                            Some(&GlyphPreparationError::InvalidFont),
                            "tag {tag}, row {row_start}, split {split}"
                        );
                        let terminal = context.last_attempt().unwrap();
                        assert_eq!(terminal.cleanup_fault(), None);
                        let report = terminal.attempt().unwrap();
                        assert_eq!(
                            report.status(),
                            crate::RenderAttemptStatus::Failed(crate::ErrorCode::GlyphPreparation)
                        );
                        // Two CRC passes each visit IHDR's 4+13 bytes, IDAT's
                        // 4+29 bytes, and IEND's 4 bytes. Splitting IDAT into
                        // three chunks adds two 4-byte names per pass.
                        let chunk_visits = if split { 124 } else { 108 };
                        assert_eq!(
                            report.usage().work(RenderWork::PngByteVisits),
                            chunk_visits + row_visits
                        );
                        assert!(report.usage().peak(RenderResource::ScratchCpuBytes) > 0);
                        assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
                        assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
                        assert_eq!(bytes, original);
                    }
                    let valid = image(&zlib(&FILTERED[0]), split);
                    assert_eq!(
                        decode_in(&valid, 1 << 20, &mut context).unwrap().pixels,
                        PIXELS
                    );
                }
            }
        }
    }

    #[test]
    fn png_work_caps_admit_exact_decode_and_refuse_before_the_last_pixel_byte() {
        use crate::{RenderAccountingOptions, RenderLimits};
        for filtered in FILTERED {
            for (split, exact) in [(false, 124), (true, 140)] {
                let bytes = image(&zlib(&filtered), split);
                for limit in [exact - 1, exact] {
                    let mut context = RenderAccountingContext::new(
                        RenderAccountingOptions::new().with_limits(
                            RenderLimits::new()
                                .with_work_limit(RenderWork::PngByteVisits, limit)
                                .unwrap(),
                        ),
                    );
                    let result = decode_in(&bytes, 1 << 20, &mut context);
                    if limit == exact {
                        assert_eq!(result.ok().unwrap().pixels, PIXELS);
                    } else {
                        assert!(result.err().unwrap().accounting_diagnostic().is_some());
                    }
                    let terminal = context.last_attempt().unwrap();
                    assert_eq!(terminal.cleanup_fault(), None);
                    assert_eq!(
                        terminal
                            .attempt()
                            .unwrap()
                            .usage()
                            .work(RenderWork::PngByteVisits),
                        limit
                    );
                    assert_eq!(
                        terminal
                            .attempt()
                            .unwrap()
                            .usage()
                            .live(RenderResource::ScratchCpuBytes),
                        0
                    );
                    assert_eq!(
                        terminal
                            .attempt()
                            .unwrap()
                            .usage()
                            .work(RenderWork::PreparedTransferredBytes),
                        0
                    );
                }
            }
        }
    }

    #[test]
    fn decoded_pixels_and_actual_scratch_admission_are_invariant_across_profiles() {
        use crate::{RenderAccountingOptions, RenderClock, RenderPhaseFilter, RenderProfile};
        fn no_clock() -> std::time::Duration {
            panic!("untimed PNG decoding must not read the clock")
        }
        fn clock() -> std::time::Duration {
            std::time::Duration::ZERO
        }
        for filtered in FILTERED {
            let bytes = image(&zlib(&filtered), true);
            let mut baseline = RenderAccountingContext::default();
            assert_eq!(
                decode_in(&bytes, 1 << 20, &mut baseline).unwrap().pixels,
                PIXELS
            );
            let usage = baseline.last_attempt().unwrap().attempt().unwrap().usage();
            let scratch = usize::try_from(usage.peak(RenderResource::ScratchCpuBytes)).unwrap();
            for (profile, timed) in [
                (RenderProfile::Off, false),
                (RenderProfile::Summary { timing: false }, false),
                (
                    RenderProfile::Detailed {
                        timing: false,
                        event_limit: 0,
                        filter: RenderPhaseFilter::All,
                    },
                    false,
                ),
                (RenderProfile::Summary { timing: true }, true),
                (
                    RenderProfile::Detailed {
                        timing: true,
                        event_limit: 64,
                        filter: RenderPhaseFilter::All,
                    },
                    true,
                ),
            ] {
                let mut context = RenderAccountingContext::new(
                    RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_clock(RenderClock::Host(if timed { clock } else { no_clock })),
                );
                // Use the observed actual backing peak, rather than assuming
                // allocator capacities equal the requested lengths.
                let error = decode_in(&bytes, scratch - 1, &mut context).err().unwrap();
                assert!(matches!(
                    error.glyph_preparation_diagnostic(),
                    Some(GlyphPreparationError::Limit {
                        resource: GlyphResource::ScratchBytes,
                        ..
                    })
                ));
                assert_eq!(
                    context
                        .last_attempt()
                        .unwrap()
                        .attempt()
                        .unwrap()
                        .usage()
                        .live(RenderResource::ScratchCpuBytes),
                    0
                );
                assert_eq!(
                    decode_in(&bytes, scratch, &mut context).unwrap().pixels,
                    PIXELS
                );
                let terminal = context.last_attempt().unwrap();
                assert_eq!(terminal.cleanup_fault(), None);
                let report = terminal.attempt().unwrap();
                assert_eq!(report.elapsed().is_some(), timed);
                assert_eq!(report.usage().work(RenderWork::PngByteVisits), 140);
                assert_eq!(
                    report.usage().peak(RenderResource::ScratchCpuBytes),
                    usage.peak(RenderResource::ScratchCpuBytes)
                );
                assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
                assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
            }
        }
    }
    #[test]
    fn all_png_filters_and_split_idat_reconstruct_literal_rgba() {
        for filtered in FILTERED {
            for split in [false, true] {
                let bytes = image(&zlib(&filtered), split);
                // Real separate provider validates the authored CRC/zlib/filter
                // fixture; this is not an assertion against decoder source.
                let mut provider = ::png::Decoder::new(std::io::Cursor::new(&bytes))
                    .read_info()
                    .unwrap();
                let mut expected = [0; 16];
                provider.next_frame(&mut expected).unwrap();
                provider.finish().unwrap();
                assert_eq!(expected, PIXELS);
                let decoded = test_decode(&bytes, 1 << 20).unwrap();
                assert_eq!((decoded.width, decoded.height), (2, 2));
                assert_eq!(decoded.pixels, PIXELS);
            }
        }
    }

    #[test]
    fn png_corrupt_crc_adler_truncation_extra_stream_and_wrong_lengths_refuse() {
        let stream = zlib(&FILTERED[0]);
        let valid = image(&stream, false);
        let mut crc_bad = valid.clone();
        crc_bad[29] ^= 1;
        let mut adler = stream.clone();
        *adler.last_mut().unwrap() ^= 1;
        let mut extra = stream.clone();
        extra.push(0);
        let mut invalid_filter = FILTERED[0];
        invalid_filter[0] = 5;
        let mut after = valid.clone();
        after.push(0);
        let mut duplicate = valid[..valid.len() - 12].to_vec();
        append(
            &mut duplicate,
            b"IHDR",
            &[0, 0, 0, 2, 0, 0, 0, 2, 8, 6, 0, 0, 0],
        );
        append(&mut duplicate, b"IEND", &[]);
        for bytes in [
            crc_bad,
            image(&adler, false),
            image(&extra, false),
            image(&stream[..stream.len() - 1], false),
            image(&zlib(&FILTERED[0][..17]), false),
            image(&zlib(&[0; 19]), false),
            image(&zlib(&invalid_filter), false),
            after,
            duplicate,
            valid[..valid.len() - 1].to_vec(),
        ] {
            let error = test_decode(&bytes, 1 << 20).err().unwrap();
            assert_eq!(
                error.glyph_preparation_diagnostic(),
                Some(&GlyphPreparationError::InvalidFont)
            );
        }
        assert_eq!(test_decode(&valid, 1 << 20).unwrap().pixels, PIXELS);
    }

    #[test]
    fn png_unexecuted_chunk_semantics_refuse_without_ignoring_color_or_animation() {
        let valid = image(&zlib(&FILTERED[0]), false);
        for kind in [*b"gAMA", *b"iCCP", *b"acTL", *b"tEXt"] {
            let mut bytes = valid[..33].to_vec();
            append(&mut bytes, &kind, &[0, 0, 0, 1]);
            bytes.extend(&valid[33..]);
            assert_eq!(
                test_decode(&bytes, 1 << 20)
                    .err()
                    .unwrap()
                    .glyph_preparation_diagnostic(),
                Some(&GlyphPreparationError::UnsupportedPng)
            );
        }
    }

    #[test]
    fn png_fixed_workspace_and_live_buffers_are_admitted_before_decode_and_retry() {
        let bytes = image(&zlib(&FILTERED[0]), false);
        let error = test_decode(&bytes, size_of::<DecompressorOxide>() - 1)
            .err()
            .unwrap();
        assert!(matches!(
            error.glyph_preparation_diagnostic(),
            Some(GlyphPreparationError::Limit {
                resource: GlyphResource::ScratchBytes,
                ..
            })
        ));
        assert_eq!(test_decode(&bytes, 1 << 20).unwrap().pixels, PIXELS);
    }
}

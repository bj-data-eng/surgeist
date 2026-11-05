use std::io::{self, Read};

use surgeist_font::{
    DecodeError, DecodeLimits, FontResource, LimitKind, ReadFontError, SourceFormat,
};

const AHEM: &[u8] = include_bytes!("fixtures/ahem-original/Ahem.woff2");
const CFF: &[u8] = include_bytes!("fixtures/source-code-pro/SourceCodePro-Regular.otf");

fn limits(encoded: usize, live: usize) -> DecodeLimits {
    DecodeLimits::new(encoded, 1 << 20, 1 << 20, 100, 10, 24 << 20, live)
}

#[test]
fn a_stream_at_its_encoded_limit_retains_the_actual_source() {
    let mut attempts = 0;
    let mut source = AHEM;
    let reader = |buffer: &mut [u8]| {
        attempts += 1;
        if attempts == 1 {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        let request = buffer.len().min(173);
        source.read(&mut buffer[..request])
    };
    let resource = FontResource::read_from(Reader(reader), limits(AHEM.len(), 32 << 20)).unwrap();
    assert!(source.is_empty());
    assert!(attempts > 2);
    assert_eq!(resource.source().bytes(), AHEM);
    assert_eq!(resource.source().format(), SourceFormat::Woff2);
    assert_eq!(resource.face(0).unwrap().sfnt_version(), [0, 1, 0, 0]);
}

#[test]
fn an_oversized_stream_reads_at_most_one_byte_past_the_limit() {
    let mut source = AHEM;
    let result = FontResource::read_from(&mut source, limits(32, 32 << 20));
    assert!(matches!(
        result,
        Err(ReadFontError::Decode(DecodeError::LimitExceeded {
            kind: LimitKind::EncodedBytes,
            required: 33,
            limit: 32,
        }))
    ));
    assert_eq!(AHEM.len() - source.len(), 33);
}

#[test]
fn input_growth_charges_both_live_buffers() {
    let result = FontResource::read_from(&CFF[..16_384], limits(CFF.len(), 20_000));
    assert!(matches!(
        result,
        Err(ReadFontError::Decode(DecodeError::LimitExceeded {
            kind: LimitKind::LiveBytes,
            required,
            limit: 20_000,
        })) if required > 20_000
    ));
}

#[test]
fn an_io_failure_remains_distinct_from_container_rejection() {
    let mut source = AHEM;
    let reader = Reader(|buffer: &mut [u8]| {
        if source.len() < AHEM.len() {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        source.read(&mut buffer[..4])
    });
    let result = FontResource::read_from(reader, limits(AHEM.len(), 32 << 20));
    assert!(matches!(
        result,
        Err(ReadFontError::Read(error)) if error.kind() == io::ErrorKind::PermissionDenied
    ));
}

struct Reader<F>(F);

impl<F: FnMut(&mut [u8]) -> io::Result<usize>> Read for Reader<F> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        (self.0)(buffer)
    }
}

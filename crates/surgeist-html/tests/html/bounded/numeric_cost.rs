#![forbid(unsafe_code)]
//! The number of requests the range ingress and best cursors cost a caller per number, counted
//! through the same service as the ingress and best conformance tests. Run with `--nocapture` to
//! print the table.

use super::numeric::service;
use surgeist_html::bounded::numeric::{
    HtmlBestFiniteNumberCursor, HtmlFiniteDecimalIngressCursor, HtmlFloatingPointGrammarCursor,
    HtmlFloatingPointParseMode, HtmlRangeNumericError, HtmlRangeNumericHeaders,
    HtmlRangeNumericRequest,
};

/// Requests by kind: source units, cell reads, reserves, appends, writes, pops, scalar steps and
/// output requests.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Requests {
    source: usize,
    reads: usize,
    reserves: usize,
    appends: usize,
    writes: usize,
    pops: usize,
    scalar: usize,
    output: usize,
    /// Runs of consecutive pops of one buffer: each rebuild of a dyadic value clears the buffer
    /// it reuses first.
    pop_runs: usize,
}
impl Requests {
    fn count(
        &mut self,
        request: HtmlRangeNumericRequest,
        previous: Option<HtmlRangeNumericRequest>,
    ) {
        if matches!(request, HtmlRangeNumericRequest::PopCell { .. }) && previous != Some(request) {
            self.pop_runs += 1;
        }
        let field = match request {
            HtmlRangeNumericRequest::SourceUnit { .. } => &mut self.source,
            HtmlRangeNumericRequest::ReadCells { .. } => &mut self.reads,
            HtmlRangeNumericRequest::Reserve { .. } => &mut self.reserves,
            HtmlRangeNumericRequest::AppendCell { .. } => &mut self.appends,
            HtmlRangeNumericRequest::WriteCell { .. } => &mut self.writes,
            HtmlRangeNumericRequest::PopCell { .. } => &mut self.pops,
            HtmlRangeNumericRequest::ScalarStep => &mut self.scalar,
            _ => &mut self.output,
        };
        *field += 1;
    }
    fn total(self) -> usize {
        self.source
            + self.reads
            + self.reserves
            + self.appends
            + self.writes
            + self.pops
            + self.scalar
            + self.output
    }
}

/// The requests that converting `spelling`, a valid floating-point number, costs, and the result.
fn ingress(spelling: &str) -> (Requests, Result<u64, HtmlRangeNumericError>) {
    let source: Vec<u16> = spelling.encode_utf16().collect();
    let mut grammar =
        HtmlFloatingPointGrammarCursor::new(&source, HtmlFloatingPointParseMode::Author);
    while !grammar.is_finished() {
        grammar.advance();
    }
    let mut cursor = HtmlFiniteDecimalIngressCursor::new(
        grammar.range_decimal_parts().unwrap(),
        source.len(),
        HtmlRangeNumericHeaders::default(),
    )
    .unwrap();
    let mut buffers = std::array::from_fn(|_| Vec::new());
    let mut output = Vec::new();
    let mut requests = Requests::default();
    let mut previous = None;
    while let Some(request) = cursor.request() {
        requests.count(request, previous);
        previous = Some(request);
        let reply = service(request, &source, &mut buffers, &mut output);
        cursor.reply(reply).unwrap();
    }
    (requests, cursor.result().unwrap())
}

/// The requests that serializing `value` costs, and the text.
fn best(value: f64) -> (Requests, String) {
    let mut cursor =
        HtmlBestFiniteNumberCursor::new(value, HtmlRangeNumericHeaders::default()).unwrap();
    let mut buffers = std::array::from_fn(|_| Vec::new());
    let mut output = Vec::new();
    let mut requests = Requests::default();
    let mut previous = None;
    while let Some(request) = cursor.request() {
        requests.count(request, previous);
        previous = Some(request);
        let reply = service(request, &[], &mut buffers, &mut output);
        cursor.reply(reply).unwrap();
    }
    (requests, String::from_utf16(&output).unwrap())
}

/// Decimals of 30 digits with a decimal exponent from -350 to 270, from a fixed generator.
fn random_decimals(count: usize) -> Vec<String> {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        state >> 33
    };
    (0..count)
        .map(|_| {
            let mut digits = String::new();
            for place in 0..30 {
                let digit = if place == 0 {
                    1 + next() % 9
                } else {
                    next() % 10
                };
                digits.push(char::from(b'0' + digit as u8));
            }
            let exponent = next() % 621;
            format!("{digits}e{}", exponent as i64 - 350)
        })
        .collect()
}

/// The conversion agrees with Rust's `str::parse::<f64>` (nearest, ties to even), and an overflow
/// is the typed error.
fn assert_ingress_matches_parse(spelling: &str, result: Result<u64, HtmlRangeNumericError>) {
    let expected: f64 = spelling.parse().unwrap();
    if expected.is_infinite() {
        assert_eq!(result, Err(HtmlRangeNumericError::Overflow), "{spelling}");
    } else {
        assert_eq!(result, Ok(expected.to_bits()), "{spelling}");
    }
}

/// The measured request counts of one number, by cursor, and the bounds this crate documents for
/// them in `docs/reference/html-operations.md`. Run with `--nocapture` to print the counts.
#[test]
fn range_cursor_request_counts_stay_within_the_documented_bounds() {
    let wide = format!("0.{}1", "0".repeat(100_000));
    let long = "9".repeat(300);
    let ingress_rows: Vec<(String, usize)> = [
        ("1", 100),
        ("12345678901234567890", 10_000),
        ("1e100", 500_000),
        ("1.7976931348623157e308", 4_000_000),
        ("1e-320", 9_000_000),
        ("4.9406564584124654e-324", 8_000_000),
        (long.as_str(), 4_000_000),
        (wide.as_str(), 250_000),
    ]
    .into_iter()
    .map(|(spelling, bound)| {
        let (requests, result) = ingress(spelling);
        assert_ingress_matches_parse(spelling, result);
        eprintln!(
            "ingress {spelling:.30}: {requests:?} total {}",
            requests.total()
        );
        assert!(requests.total() <= bound, "{spelling:.30}: {requests:?}");
        (spelling.to_owned(), requests.total())
    })
    .collect();
    eprintln!(
        "register bytes copied by each reply: ingress {}, best {}",
        HtmlFiniteDecimalIngressCursor::layout().register_bytes(),
        HtmlBestFiniteNumberCursor::layout().register_bytes()
    );
    assert_eq!(ingress_rows[0].1, 16);
    // The cost is set by the value, not by the number of digits: 100,000 zeros cost two requests
    // each and no scratch cell.
    assert_eq!(ingress_rows[7].1, 200_006);
    let mut largest = 0;
    for spelling in random_decimals(8) {
        let (requests, result) = ingress(&spelling);
        assert_ingress_matches_parse(&spelling, result);
        largest = largest.max(requests.total());
    }
    eprintln!("ingress of 8 random 30-digit decimals, exponent -350 to 270: largest {largest}");
    assert!(largest <= 12_000_000, "{largest}");
    for (value, text, bound) in [
        (1.0, "1", 15_000),
        (1e-100, "1e-100", 800_000),
        (1e300, "1e+300", 2_000_000),
        (5e-324, "5e-324", 5_500_000),
    ] {
        let (requests, serialized) = best(value);
        // ECMAScript Number::prototype.toString of the value.
        assert_eq!(serialized, text);
        eprintln!("best {text}: {requests:?} total {}", requests.total());
        assert!(requests.total() <= bound, "{text}: {requests:?}");
    }
}

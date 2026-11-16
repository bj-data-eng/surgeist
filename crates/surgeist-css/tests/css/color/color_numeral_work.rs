//! A colour numeral costs work in proportion to its digits, not its exponent.
//!
//! The controlled colour parse reports every unit of work as a caller-serviced
//! action. These tests count those actions and the scratch records the caller
//! must store, so the bound never depends on timing.

use surgeist_css::color::controlled::{
    CssColorBank, CssColorOperationOwner, CssColorParse, CssColorParseAction, CssColorParseOutcome,
    CssColorParseStep, CssColorSlot,
};
use surgeist_css::color::{
    CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInputKind,
    CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact,
};

#[derive(Debug)]
struct Work {
    /// Every action the caller serviced, of any kind.
    actions: usize,
    /// Final length of the algorithm bank: the lexed input records plus any
    /// decimal search scratch.
    scratch_records: usize,
    used: bool,
}

fn bank_index(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}

fn work(text: &str) -> Work {
    let input: Vec<u16> = text.encode_utf16().collect();
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(1, 1),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    let mut owner = CssColorOperationOwner::new();
    let mut parse = CssColorParse::start(
        &mut owner,
        CssColorInputKind::Utf16,
        input.len(),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
    )
    .unwrap();
    let mut banks: [Vec<CssColorSlot<'_>>; 3] = std::array::from_fn(|_| Vec::new());
    let mut actions = 0;
    loop {
        let action = match parse.step() {
            CssColorParseStep::Complete(outcome) => {
                return Work {
                    actions,
                    scratch_records: banks[0].len(),
                    used: matches!(outcome, CssColorParseOutcome::Used(_)),
                };
            }
            CssColorParseStep::Pending(action) => action,
        };
        actions += 1;
        parse = match action {
            CssColorParseAction::ReadInput(read) => {
                let unit = input[read.offset()];
                read.read(unit).unwrap()
            }
            CssColorParseAction::ReadSlot(read) => {
                let slot = &banks[bank_index(read.bank())][read.index()];
                read.read(slot).unwrap()
            }
            CssColorParseAction::AppendSlot(write) => {
                let target = &mut banks[bank_index(write.bank())];
                assert_eq!(write.index(), target.len());
                let (record, receipt) = write.record();
                target.push(record);
                receipt.stored(&target[target.len() - 1])
            }
            CssColorParseAction::UpdateSlot(write) => {
                let index = write.index();
                let target = &mut banks[bank_index(write.bank())];
                let (record, receipt) = write.record();
                target[index] = record;
                receipt.stored(&target[index])
            }
            CssColorParseAction::Compute(compute) => compute.compute().unwrap(),
            CssColorParseAction::ReadFact(read) => read.read().unwrap(),
        };
    }
}

#[test]
fn ordinary_numerals_need_no_decimal_search_scratch() {
    // Each numeral has at most 15 significant digits and a decimal exponent of
    // at most 22 in magnitude, so one correctly rounded IEEE operation decides
    // it: the algorithm bank holds only the lexed input (a few records per
    // source character), never the decimal search scratch.
    for text in [
        "rgb(10 20 30)",
        "hsl(120 50% 50%)",
        "color(srgb 1 0.5 0)",
        "rgb(0.123456 100.5 255 / 0.25)",
    ] {
        let work = work(text);
        assert!(work.used, "{text}");
        assert!(work.scratch_records <= 4 * text.len(), "{text}: {work:?}");
        // Dominated by lexing and grammar: tens of actions per source character,
        // since an exactly representable numeral needs no decimal search.
        assert!(work.actions <= 50 * text.len(), "{text}: {work:?}");
    }
}

#[test]
fn a_numeral_outside_the_window_costs_the_read_of_its_digits() {
    // A component numeral at or above 1e41 or below 1e-46 is not converted
    // exactly (#2889): the decimal owner stops after the digits are read, so the
    // algorithm bank holds only the lexed input and the actions are those of an
    // ordinary numeral of the same length, whatever the exponent.
    for text in [
        "lab(0 1e41 0)",
        "lab(0 1e308 0)",
        "lab(0 1e999 0)",
        "lab(0 -1e999 0)",
        "lab(0 1e2000000 0)",
        "lab(0 9.99e-47 0)",
        "lab(0 1e-300 0)",
        "lab(0 2.2250738585072014e-308 0)",
        "lab(0 5e-324 0)",
        "lab(0 calc(1e308 * 1e-300) 0)",
        "color-mix(in lab, lab(0 1e308 0), red)",
        "oklch(0.5 0.1 1e308)",
        "hwb(none calc(2.2250738585072014e-308) 20%)",
        "hwb(none calc(5e-324) 20%)",
        "hwb(none calc(1e-300) 20%)",
        "hwb(none calc(1e308) 20%)",
    ] {
        let work = work(text);
        assert!(work.used, "{text}");
        assert!(work.scratch_records <= 4 * text.len(), "{text}: {work:?}");
        assert!(work.actions <= 100 * text.len(), "{text}: {work:?}");
    }
}

#[test]
fn a_numeral_at_the_edge_of_the_window_is_converted_exactly_within_a_bounded_search() {
    let ordinary = work("hwb(none calc(0.5) 20%)");
    assert!(ordinary.used);
    // Inside the window the correctly rounded conversion still searches: a few
    // guided candidates, each rebuilding a midpoint of at most about 170 digits
    // (the numeral 1e-46 has the largest midpoint, 2^-153 = 5^153 / 10^153).
    for text in [
        "lab(0 3.4028234663852886e38 0)",
        "lab(0 9.999999999999999e40 0)",
        "lab(0 1e-46 0)",
        "lab(0 1.401298464324817e-45 0)",
        "lab(0 1e-45 0)",
        "hwb(none calc(1e-46) 20%)",
        "hwb(none calc(9.999999999999999e40) 20%)",
    ] {
        let edge = work(text);
        assert!(edge.used, "{text}");
        assert!(
            edge.actions <= WINDOW_ACTION_BOUND,
            "{text}: {edge:?} against ordinary {ordinary:?}"
        );
        assert!(
            edge.scratch_records <= 4 * text.len() + 200,
            "{text}: the exact boundary needs at most about 170 digits: {edge:?}"
        );
    }
}

/// Upper bound on the actions of one colour with a single numeral inside the
/// window. The largest measured edge numeral needs about 11,000 actions; the
/// bound leaves headroom for the neighbouring numerals.
const WINDOW_ACTION_BOUND: usize = 30_000;

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
fn an_extreme_exponent_costs_a_bounded_multiple_of_an_ordinary_numeral() {
    let ordinary = work("hwb(none calc(0.5) 20%)");
    assert!(ordinary.used);
    // A numeral outside the exact fast path costs a few guided candidates, each
    // rebuilding a midpoint of at most 1,100 digits, whatever its exponent.
    for text in [
        "hwb(none calc(2.2250738585072014e-308) 20%)",
        "hwb(none calc(5e-324) 20%)",
        "hwb(none calc(1e-300) 20%)",
        "hwb(none calc(1e308) 20%)",
    ] {
        let extreme = work(text);
        assert!(extreme.used, "{text}");
        assert!(
            extreme.actions <= SEARCH_ACTION_BOUND,
            "{text}: {extreme:?} against ordinary {ordinary:?}"
        );
        assert!(
            extreme.scratch_records <= 4 * text.len() + 1_100,
            "{text}: the exact boundary needs at most 1100 digits: {extreme:?}"
        );
    }
}

/// Upper bound on the actions of one colour with a single searched numeral. The
/// largest of the four extreme inputs measures 237,698 actions
/// (`2.2250738585072014e-308`, the smallest normal); the bound is about twice that.
const SEARCH_ACTION_BOUND: usize = 500_000;

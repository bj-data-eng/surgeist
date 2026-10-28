//! The charged work and live bytes of `input::preprocess_utf16` equal those of
//! the UTF-16 preparation loop it replaces, on shared inputs.
//!
//! The expected values are in `tests/receipts/utf16-preprocessing.tsv`. They
//! are literal values that were recorded once from the old function, the loop
//! of `prepare_utf16` in `crates/surgeist-css/src/source/utf16_input.rs`, run on
//! each input by a scratch harness (the file's header says exactly how); this
//! crate's output is not their source.
//!
//! What is compared. The old function also formed the source snapshot (its
//! checkpoints, which stay in `surgeist-css`) and owned the original units. Its
//! charges before the loop differ from this function's by the registers and
//! the original backing that only the old owner had:
//!
//! - the original `Vec<u16>` backing, `2 * capacity` bytes, is the host's; and
//! - the registers only the old function held: a `Vec<CssSourcePosition>` (24
//!   bytes), a `CssSourcePosition` (16), a `Vec<u16>` (24), an `Arc` (8) and
//!   its `Preparation` (128), 200 bytes in all.
//!
//! So the old first live level is `2 * capacity + 200` plus this function's
//! first charge (the cursor, 144, a `String`, 24, and the map's `Vec`, 24: 192),
//! which is 392 plus the original backing. Everything after that first charge,
//! up to the end of the loop, is compared one change at a time, with the
//! steps (the work charged). This function then returns the cursor's 144 bytes;
//! what remains is the result's own storage.
//!
//! By hand, for the input `color: red; margin: 1px 2px` (27 ASCII units): the
//! loop makes a read and an emit request per unit and one end request, 55
//! steps. The text grows at the 1st, 2nd, 3rd, 5th, 9th and 17th scalar,
//! moving 0 + 1 + 2 + 4 + 8 + 16 = 31 scalars, and the map moves the same 31
//! entries, so 55 + 62 = 117 steps. The live level rises by the final capacity
//! of the text (32) and of the map (32 entries of 16 bytes, 512): 544 bytes,
//! which is the old 990 - 446. For `content: "é😀"; color: blue` (27 units, 26
//! scalars of 30 bytes): 27 reads, 26 emits and one end make 54 requests. The
//! text grows (and moves what it holds, a step per scalar) when a scalar needs
//! its 1st, 2nd, 3rd, 5th, 9th and 17th byte, at which points it holds 0, 1, 2,
//! 4, 8 and 12 scalars (10 ASCII, é and 😀 before the 17th byte): 27 moves. The
//! map moves 31 entries as before. So 54 + 27 + 31 = 112 steps. The final
//! capacities are again 32 bytes and 32 entries, 544 bytes.

use std::cell::{Cell, RefCell};
use std::mem::size_of;

use surgeist_css_parser::input::{InputCursor, PreprocessLimits, preprocess_utf16};
use surgeist_css_parser::limits::{Failure, Meter, Refused};

const OLD: &str = include_str!("receipts/utf16-preprocessing.tsv");

/// Registers that only the old function held: `Vec<CssSourcePosition>` (24),
/// `CssSourcePosition` (16), `Vec<u16>` (24), `Arc<Vec<u16>>` (8) and
/// `Preparation` (128).
const OLD_ONLY_REGISTERS: usize = 24 + 16 + 24 + 8 + 128;

/// The registers this function charges first: the cursor, a `String` and the
/// map's `Vec`.
const NEW_REGISTERS: usize = 144 + 24 + 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Change {
    Step,
    Add(usize),
    Released(usize),
}

#[derive(Default)]
struct Trace {
    changes: RefCell<Vec<Change>>,
    live: Cell<usize>,
}

impl Meter for Trace {
    fn step(&self) -> Result<(), Refused> {
        self.changes.borrow_mut().push(Change::Step);
        Ok(())
    }

    fn add(&self, bytes: usize) -> Result<(), Refused> {
        self.changes.borrow_mut().push(Change::Add(bytes));
        self.live.set(self.live.get() + bytes);
        Ok(())
    }

    fn released(&self, bytes: usize) {
        self.changes.borrow_mut().push(Change::Released(bytes));
        self.live.set(self.live.get() - bytes);
    }

    fn note(&self, _: Failure) -> Refused {
        Refused
    }
}

/// One recorded row: what the old loop charged for an input.
struct OldRow {
    capacity: usize,
    first_live: usize,
    steps: usize,
    level_after_loop: usize,
    changes: Vec<Change>,
}

fn old_rows() -> Vec<(&'static str, OldRow)> {
    let mut rows = Vec::new();
    for line in OLD
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
    {
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 6, "{line}");
        let number = |text: &str| text.parse::<usize>().expect("a recorded number");
        let changes = columns[5]
            .split(' ')
            .map(|token| match token.as_bytes()[0] {
                b'S' => {
                    assert_eq!(token, "S");
                    Change::Step
                }
                b'A' => Change::Add(number(&token[1..])),
                b'R' => Change::Released(number(&token[1..])),
                _ => panic!("change {token}"),
            })
            .collect();
        rows.push((
            columns[0],
            OldRow {
                capacity: number(columns[1]),
                first_live: number(columns[2]),
                steps: number(columns[3]),
                level_after_loop: number(columns[4]),
                changes,
            },
        ));
    }
    rows
}

/// The shared inputs, by the names the recorded rows use.
fn input(name: &str) -> Vec<u16> {
    let utf16 = |text: &str| text.encode_utf16().collect::<Vec<u16>>();
    match name {
        "empty" => vec![],
        "mixed" => vec![13, 10, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00],
        "color" => utf16("color: red; margin: 1px 2px"),
        "nonascii" => utf16("content: \"é😀\"; color: blue"),
        "ascii130" => utf16(&"a".repeat(130)),
        "sheet" => utf16(".a { color: red } @media print { .b { width: 1px } }"),
        "crlf" => utf16(&"x\r\n".repeat(40)),
        "wide" => utf16(&format!("{}{}a", "é".repeat(100), "😀".repeat(20))),
        other => panic!("no input named {other}"),
    }
}

#[test]
fn the_recorded_rows_cover_every_shared_input() {
    let names: Vec<&str> = old_rows().iter().map(|(name, _)| *name).collect();
    assert_eq!(
        names,
        [
            "empty", "mixed", "color", "nonascii", "ascii130", "sheet", "crlf", "wide"
        ]
    );
}

#[test]
fn the_registers_of_the_old_and_new_function_differ_by_the_original_backing_and_two_hundred_bytes()
{
    assert_eq!(size_of::<InputCursor>(), 144);
    assert_eq!(
        NEW_REGISTERS,
        size_of::<InputCursor>() + size_of::<String>() + size_of::<Vec<(usize, usize)>>()
    );
    for (name, row) in old_rows() {
        assert_eq!(
            row.first_live,
            2 * row.capacity + OLD_ONLY_REGISTERS + NEW_REGISTERS,
            "{name}"
        );
    }
}

#[test]
fn the_work_and_the_live_bytes_charged_equal_those_of_the_old_loop() {
    for (name, row) in old_rows() {
        let units = input(name);
        let trace = Trace::default();
        let result = preprocess_utf16(&units, &PreprocessLimits::default(), &trace)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let changes = trace.changes.borrow().clone();
        // The first charge, the loop, and the cursor's release at its end.
        assert_eq!(changes.first(), Some(&Change::Add(NEW_REGISTERS)), "{name}");
        assert_eq!(changes.last(), Some(&Change::Released(144)), "{name}");
        let middle = &changes[1..changes.len() - 1];
        assert_eq!(
            middle,
            row.changes.as_slice(),
            "{name}: every change, in order"
        );
        // The work charged is the number of steps.
        let steps = middle
            .iter()
            .filter(|change| **change == Change::Step)
            .count();
        assert_eq!(steps, row.steps, "{name}: work");
        // The live level the loop reaches, relative to its first charge.
        let net = middle.iter().fold(0_isize, |net, change| match change {
            Change::Add(bytes) => net + isize::try_from(*bytes).unwrap(),
            Change::Released(bytes) => net - isize::try_from(*bytes).unwrap(),
            Change::Step => net,
        });
        assert_eq!(
            usize::try_from(net).unwrap(),
            row.level_after_loop - row.first_live,
            "{name}: live bytes"
        );
        // What the result keeps charged: its text and map, with one header each.
        assert_eq!(trace.live.get(), result.charged_bytes(), "{name}");
        assert_eq!(
            result.charged_bytes(),
            NEW_REGISTERS - 144 + usize::try_from(net).unwrap(),
            "{name}"
        );
    }
}

#[test]
fn the_hand_derived_step_counts_match_the_recorded_old_loop() {
    // See the arithmetic in the module documentation.
    let rows = old_rows();
    let steps = |name: &str| rows.iter().find(|(row, _)| *row == name).unwrap().1.steps;
    assert_eq!(steps("empty"), 1);
    assert_eq!(steps("color"), 55 + 62);
    assert_eq!(steps("nonascii"), 54 + 27 + 31);
    // The final capacities of the two 27-unit inputs: 32 bytes and 32 entries.
    for name in ["color", "nonascii"] {
        let row = &rows.iter().find(|(row, _)| *row == name).unwrap().1;
        assert_eq!(row.level_after_loop - row.first_live, 32 + 32 * 16);
    }
}

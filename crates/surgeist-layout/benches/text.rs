//! Release-mode layout benchmark over deterministic generated Text paragraphs.
//!
//! Each document is one block container of prepared Ahem Text. A round runs one full
//! `compute_layout_with_flow` from the root over a freshly checked flow input and reports its
//! document's node count and depth, with the root at depth one, beside the median, minimum and
//! maximum time over the rounds. The documents are long paragraphs under each `text-wrap-style`,
//! forced-break groups under `balance`, and shape-backed floats that the paragraph wraps around.
//!
//! Run with `cargo bench --offline --locked -j 1 -p surgeist-layout --bench text`.
//! A first argument without a leading dash selects the documents whose name contains it, as in
//! `--bench text -- pretty`. A name that matches no document exits with status 2 and lists the
//! known names. The Ahem font is the test fixture file of `surgeist-text`, read by path, and the
//! font limits come from the `surgeist-test` dev-dependency.

#[path = "support/harness.rs"]
mod harness;
#[path = "support/host.rs"]
mod host;
#[path = "support/text.rs"]
mod text;

use std::hint::black_box;

use harness::{Document, Family};
use surgeist_layout::text_flow::TextWrapStyle;
use text::Paragraphs;

const WIDTH: f32 = 400.0;
const PARAGRAPH_WORDS: usize = 25;
const GROUPS: usize = 10;
const GROUP_WORDS: usize = 10;
const FLOATS: usize = 10;
const FLOAT_WORDS: usize = 100;

fn document(paragraphs: Paragraphs) -> Document {
    let (nodes, depth) = paragraphs.host.shape();
    Document {
        nodes,
        depth,
        round: Box::new(move || {
            black_box(paragraphs.compute(WIDTH));
        }),
    }
}

fn paragraph(style: TextWrapStyle) -> Document {
    document(Paragraphs::new(
        &[text::words(PARAGRAPH_WORDS, 1)],
        WIDTH,
        style,
        0,
        false,
    ))
}

fn groups(style: TextWrapStyle) -> Document {
    let groups: Vec<_> = (0..GROUPS)
        .map(|index| text::words(GROUP_WORDS, index as u64 + 1))
        .collect();
    document(Paragraphs::new(&groups, WIDTH, style, 0, false))
}

fn floats(shape: bool) -> Document {
    document(Paragraphs::new(
        &[text::words(FLOAT_WORDS, 1)],
        WIDTH,
        TextWrapStyle::Auto,
        FLOATS,
        shape,
    ))
}

static FAMILIES: &[Family] = &[
    Family {
        name: "paragraph-auto-25-words",
        build: || paragraph(TextWrapStyle::Auto),
    },
    Family {
        name: "paragraph-balance-25-words",
        build: || paragraph(TextWrapStyle::Balance),
    },
    Family {
        name: "paragraph-stable-25-words",
        build: || paragraph(TextWrapStyle::Stable),
    },
    Family {
        name: "paragraph-avoid-short-last-line-25-words",
        build: || paragraph(TextWrapStyle::AvoidShortLastLine),
    },
    Family {
        name: "paragraph-pretty-25-words",
        build: || paragraph(TextWrapStyle::Pretty),
    },
    Family {
        name: "groups-auto-10x10-words",
        build: || groups(TextWrapStyle::Auto),
    },
    Family {
        name: "groups-balance-10x10-words",
        build: || groups(TextWrapStyle::Balance),
    },
    Family {
        name: "groups-pretty-10x10-words",
        build: || groups(TextWrapStyle::Pretty),
    },
    Family {
        name: "floats-margin-box-10-floats-100-words",
        build: || floats(false),
    },
    Family {
        name: "floats-shape-10-floats-100-words",
        build: || floats(true),
    },
];

fn main() {
    harness::run("text", FAMILIES);
}

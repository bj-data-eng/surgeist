//! Handwritten recipes. The recipe is the oracle: each states its tree through
//! the closed model, derived from the HTML Standard, and none is a sampled
//! parser output.
use super::model::*;
use super::{Body, Case, Mode};
use crate::captured::{Fault, Plan};
use crate::error;
use surgeist_fuzz::Result;

fn t(s: &str) -> Item {
    Item::Text(s.encode_utf16().map(Piece::Unit).collect())
}
fn tp(p: Vec<Piece>) -> Item {
    Item::Text(p)
}
fn u(s: &str) -> Vec<Piece> {
    s.encode_utf16().map(Piece::Unit).collect()
}
fn c(s: &str) -> Item {
    Item::Comment(u(s))
}
fn at(name: usize, quote: u8, value: &str) -> Attribute {
    Attribute {
        name: name as u8,
        case: 0,
        quote,
        value: u(value),
    }
}
fn el(tag: usize, attrs: Vec<Attribute>, children: Vec<Item>) -> Item {
    Item::Elem(Elem {
        tag: tag as u8,
        spell: 0,
        attrs,
        shadow: None,
        children,
    })
}
fn spelled(tag: usize, spell: u8, attrs: Vec<Attribute>, children: Vec<Item>) -> Item {
    Item::Elem(Elem {
        tag: tag as u8,
        spell,
        attrs,
        shadow: None,
        children,
    })
}
fn shadow_host(tag: usize, bits: u8, inner: Vec<Item>, light: Vec<Item>) -> Item {
    Item::Elem(Elem {
        tag: tag as u8,
        spell: 0,
        attrs: Vec::new(),
        shadow: Some(Shadow {
            bits,
            children: inner,
        }),
        children: light,
    })
}
const DOC: fn(u8, u8) -> Mode = |doctype, shell| Mode::Document { doctype, shell };
const FRAG: fn(u8, bool) -> Mode = |context, shadow| Mode::Fragment { context, shadow };

/// Capture-import seeds pair a balanced recipe with external facts or one fault.
const CAPTURE: [(u8, Plan); 16] = [
    (0, Plan::DEFAULT),
    (
        1,
        Plan {
            scripting: 1,
            registry: 2,
            ..Plan::DEFAULT
        },
    ),
    (
        2,
        Plan {
            scripting: 2,
            registry: 1,
            is_value: 1,
            ..Plan::DEFAULT
        },
    ),
    (
        3,
        Plan {
            registry: 2,
            is_value: 2,
            ..Plan::DEFAULT
        },
    ),
    (
        4,
        Plan {
            fault: Fault::Revision,
            ..Plan::DEFAULT
        },
    ),
    (
        5,
        Plan {
            fault: Fault::MissingFacts,
            pick: 3,
            ..Plan::DEFAULT
        },
    ),
    (
        6,
        Plan {
            fault: Fault::DuplicateFacts,
            pick: 2,
            ..Plan::DEFAULT
        },
    ),
    (
        7,
        Plan {
            fault: Fault::RoleMismatch,
            pick: 1,
            ..Plan::DEFAULT
        },
    ),
    (
        8,
        Plan {
            registry: 2,
            shadow: 5,
            ..Plan::DEFAULT
        },
    ),
    (
        9,
        Plan {
            fault: Fault::ForeignNode,
            ..Plan::DEFAULT
        },
    ),
    (
        10,
        Plan {
            registry: 2,
            fault: Fault::ForeignRegistry,
            pick: 1,
            ..Plan::DEFAULT
        },
    ),
    (
        11,
        Plan {
            fault: Fault::MappingLimit,
            ..Plan::DEFAULT
        },
    ),
    (
        8,
        Plan {
            fault: Fault::Cancelled,
            ..Plan::DEFAULT
        },
    ),
    (
        4,
        Plan {
            fault: Fault::ScratchLimit,
            ..Plan::DEFAULT
        },
    ),
    (
        3,
        Plan {
            fault: Fault::WorkLimit,
            pick: 5,
            ..Plan::DEFAULT
        },
    ),
    (
        8,
        Plan {
            fault: Fault::SubjectLimit,
            ..Plan::DEFAULT
        },
    ),
];

pub(crate) fn case(id: u8, capture: bool) -> Result<Case> {
    let (recipe, plan) = if capture {
        let (recipe, plan) = CAPTURE
            .get(usize::from(id))
            .copied()
            .ok_or_else(|| error("unknown capture seed"))?;
        (recipe, Some(plan))
    } else {
        (id, None)
    };
    let (mode, utf16, items) = match recipe {
        // References, CRLF normalization and attribute folding in a no-quirks document.
        0 => (
            DOC(1, 0),
            false,
            vec![el(
                DIV,
                vec![at(0, 0, "a"), at(1, 1, "x y")],
                vec![
                    tp([
                        u("A"),
                        vec![Piece::Named(0)],
                        u("B"),
                        vec![Piece::CrLf],
                        u("C"),
                    ]
                    .concat()),
                    c("note"),
                    el(BR, vec![], vec![]),
                ],
            )],
        ),
        // C1 and out-of-range numeric references, ignored NUL, nesting of phrasing.
        1 => (
            DOC(3, 1),
            false,
            vec![el(
                SECTION,
                vec![],
                vec![
                    el(
                        SPAN,
                        vec![],
                        vec![el(B, vec![], vec![t("x")]), el(EM, vec![], vec![t("y")])],
                    ),
                    el(
                        P,
                        vec![],
                        vec![tp([
                            u("z"),
                            vec![
                                Piece::Numeric {
                                    code: 0x80,
                                    hex: true,
                                },
                                Piece::Numeric {
                                    code: 0xD800,
                                    hex: false,
                                },
                                Piece::Numeric {
                                    code: 0,
                                    hex: false,
                                },
                                Piece::Nul,
                            ],
                        ]
                        .concat())],
                    ),
                ],
            )],
        ),
        // Exact UTF-16 with an isolated surrogate; the first duplicate attribute wins.
        2 => (
            DOC(4, 2),
            true,
            vec![el(
                DIV,
                vec![at(0, 0, "x"), at(0, 2, "y"), at(2, 0, "t")],
                vec![tp(vec![
                    Piece::Unit(0x41),
                    Piece::Unit(0xD800),
                    Piece::Unit(0x42),
                    Piece::Cr,
                    Piece::Unit(0x43),
                ])],
            )],
        ),
        // SVG case adjustments, qualified attributes and an HTML integration point.
        3 => (
            DOC(6, 0),
            false,
            vec![spelled(
                SVG,
                0b0101,
                vec![
                    Attribute {
                        name: 6,
                        case: 1,
                        quote: 0,
                        value: u("0 0 1 1"),
                    },
                    at(7, 1, "x"),
                    at(8, 0, "en"),
                ],
                vec![
                    el(
                        G,
                        vec![],
                        vec![el(RECT, vec![], vec![]), el(CIRCLE, vec![], vec![])],
                    ),
                    spelled(
                        FOREIGN_OBJECT,
                        0b1010,
                        vec![],
                        vec![el(DIV, vec![], vec![t("h")])],
                    ),
                ],
            )],
        ),
        // Template contents and MathML name adjustment.
        4 => (
            DOC(1, 0),
            false,
            vec![
                el(
                    TEMPLATE,
                    vec![],
                    vec![
                        el(DIV, vec![], vec![t("t")]),
                        el(TEMPLATE, vec![], vec![t("n")]),
                    ],
                ),
                el(
                    MATH,
                    vec![at(10, 0, "u")],
                    vec![el(
                        MROW,
                        vec![],
                        vec![el(MI, vec![], vec![t("x")]), el(MN, vec![], vec![t("1")])],
                    )],
                ),
            ],
        ),
        // Fragment parsing in a div context.
        5 => (
            FRAG(0, false),
            false,
            vec![el(SPAN, vec![], vec![t("a")]), t("b"), c("c")],
        ),
        // Foreign context: NUL becomes U+FFFD in foreign text.
        6 => (
            FRAG(3, false),
            false,
            vec![
                el(RECT, vec![], vec![]),
                el(G, vec![], vec![el(CIRCLE, vec![], vec![])]),
                tp(vec![Piece::Unit(0x61), Piece::Nul]),
            ],
        ),
        // Template context.
        7 => (
            FRAG(2, false),
            false,
            vec![el(DIV, vec![], vec![t("x")]), el(P, vec![], vec![t("y")])],
        ),
        // Declarative shadow root with all properties, permitted.
        8 => (
            FRAG(0, true),
            false,
            vec![shadow_host(
                DIV,
                0b10101,
                vec![el(SPAN, vec![], vec![t("s")])],
                vec![t("light")],
            )],
        ),
        // The same recipe without permission is an ordinary first-child template.
        9 => (
            FRAG(0, false),
            false,
            vec![shadow_host(
                DIV,
                0b10101,
                vec![el(SPAN, vec![], vec![t("s")])],
                vec![t("light")],
            )],
        ),
        // HTML integration point context.
        10 => (
            FRAG(4, false),
            false,
            vec![el(DIV, vec![], vec![el(P, vec![], vec![t("x")])])],
        ),
        // Astral text by pair and by numeric reference in a section context.
        11 => (
            FRAG(1, false),
            false,
            vec![tp(vec![
                Piece::Unit(0xD83D),
                Piece::Unit(0xDE00),
                Piece::Numeric {
                    code: 0x1F600,
                    hex: true,
                },
                Piece::Named(5),
            ])],
        ),
        _ => return Err(error("unknown markup seed")),
    };
    Case::new(
        mode,
        utf16,
        Body::Exact(items),
        crate::codec::CAPS,
        false,
        plan,
    )
}

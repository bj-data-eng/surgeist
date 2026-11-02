//! Entropy-driven native recipes and checked structural reduction.
//!
//! Every choice is drawn from closed pools chosen to reach the contract's edges:
//! near-miss template spellings, shared and distinct attribute expanded names,
//! reserved and malformed processing-instruction targets, leaf and template
//! parents, repeated shadow attachment, foreign handles, lowered quotas and
//! isolated surrogates. An exhausted entropy slice reads as zero bytes, so a
//! given slice always yields the same recipe.
use super::*;

const SVG: &str = "http://www.w3.org/2000/svg";
const MATH: &str = "http://www.w3.org/1998/Math/MathML";
const XLINK: &str = "http://www.w3.org/1999/xlink";
const UNITS: [u16; 17] = [
    97, 98, 0, 0xD800, 0xDC00, 0xD83D, 0xDE00, 13, 10, 60, 62, 38, 0x3B1, 0xFFFF, 0xFEFF, 32, 0x178,
];
const LOCALS: [&str; 8] = [
    "template", "Template", "TEMPLATE", "div", "svg", "a:b", "", "é",
];
const URIS: [&str; 5] = ["", SVG, MATH, "urn:x", XLINK];
const PREFIXES: [Option<&str>; 4] = [None, Some(""), Some("p"), Some("x")];
const ATTRIBUTE_LOCALS: [&str; 5] = ["id", "class", "hidden", "href", "ID"];
const PI_TARGETS: [&str; 9] = [
    "target",
    "_t-9",
    "xml",
    "XML",
    "Xml-StyleSheet",
    "9x",
    "a b",
    "",
    "\u{178}ml",
];
const PI_DATA: [&str; 5] = ["data", "", "a>b", "a?", "?x"];
const QUOTA_CEILINGS: [usize; 5] = [12, 12, 6, 40, 4];

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl Cursor<'_> {
    fn byte(&mut self) -> u8 {
        let b = self.bytes.get(self.at).copied().unwrap_or(0);
        self.at = self.at.saturating_add(1);
        b
    }
    fn below(&mut self, n: usize) -> usize {
        usize::from(self.byte()) % n.max(1)
    }
    fn units(&mut self) -> Vec<u16> {
        let n = self.below(6);
        (0..n).map(|_| UNITS[self.below(UNITS.len())]).collect()
    }
    fn name(&mut self) -> Name {
        match self.below(8) {
            0 => Name::Known(0),
            1 => Name::Known(1),
            2 | 3 => Name::Known(2),
            4 => Name::Known(3),
            _ => {
                // HTML or foreign namespace with a near-miss or ordinary local.
                let uri = if self.byte().is_multiple_of(2) {
                    seeds::HTML.to_vec()
                } else {
                    seeds::units(URIS[self.below(URIS.len())])
                };
                Name::Raw {
                    uri,
                    prefix: PREFIXES[self.below(PREFIXES.len())].map(seeds::units),
                    local: seeds::units(LOCALS[self.below(LOCALS.len())]),
                }
            }
        }
    }
    fn attribute(&mut self) -> Attr {
        if self.byte().is_multiple_of(8) {
            return Attr::Hidden(self.byte() % 2);
        }
        let uri = seeds::units(URIS[self.below(URIS.len())]);
        Attr::Raw(
            Name::Raw {
                uri,
                prefix: PREFIXES[self.below(PREFIXES.len())].map(seeds::units),
                local: seeds::units(ATTRIBUTE_LOCALS[self.below(ATTRIBUTE_LOCALS.len())]),
            },
            self.units(),
        )
    }
    fn optional(&mut self) -> Option<Vec<u16>> {
        (!self.byte().is_multiple_of(3)).then(|| self.units())
    }
}

pub(super) fn synthesize(input: &[u8]) -> NativeCase {
    let mut c = Cursor {
        bytes: input.get(1..).unwrap_or_default(),
        at: 0,
    };
    let root = c.byte() % 4;
    let mut caps = CAPS;
    if c.byte().is_multiple_of(4) {
        let axis = c.below(caps.len());
        caps[axis] = c.below(QUOTA_CEILINGS[axis] + 1);
    }
    let count = 1 + c.below(20);
    let mut ops = Vec::new();
    // Roots: the main builder, the foreign probe and the inspection probe.
    let (mut nodes, mut attrs, mut units) = (3, 0, 0);
    for index in 0..count {
        let parent = if c.byte() % 16 == 15 {
            255
        } else {
            u8::try_from(index - c.below(index.min(3) + 1)).unwrap_or(0)
        };
        let op = match c.below(10) {
            0..=3 => {
                let name = c.name();
                let n = c.below(4);
                let list: Vec<Attr> = (0..n).map(|_| c.attribute()).collect();
                Op::Element(parent, name, list)
            }
            4 => Op::Text(parent, c.units()),
            5 => Op::Comment(parent, c.units()),
            6 => Op::Pi(
                parent,
                seeds::units(PI_TARGETS[c.below(PI_TARGETS.len())]),
                seeds::units(PI_DATA[c.below(PI_DATA.len())]),
            ),
            7 => Op::Doctype(
                parent,
                seeds::units(["html", "", "svg"][c.below(3)]),
                c.optional(),
                c.optional(),
            ),
            8 => Op::Contents(parent),
            _ => Op::Shadow(parent, c.byte() % 32),
        };
        let cost = match &op {
            Op::Element(..) => 2,
            Op::Contents(_) => 0,
            _ => 1,
        };
        let more_attrs = if let Op::Element(_, _, a) = &op {
            a.len()
        } else {
            0
        };
        if nodes + cost > 64 || attrs + more_attrs > 128 || units + op.units() > 8192 {
            break;
        }
        nodes += cost;
        attrs += more_attrs;
        units += op.units();
        ops.push(op);
    }
    NativeCase {
        root,
        literal: None,
        caps,
        ops,
    }
}

/// Valid command-list reductions: suffix removal, removal of a command no later
/// command names, and halving one text or comment payload. Handles are
/// renumbered, never repaired, and caps, mode and UTF16 units are unchanged.
pub(super) fn candidates(c: &NativeCase) -> Vec<NativeCase> {
    let mut out = Vec::new();
    let n = c.ops.len();
    let with = |ops: Vec<Op>| NativeCase {
        root: c.root,
        literal: None,
        caps: c.caps,
        ops,
    };
    for keep in [0, n / 2, n.saturating_sub(1)] {
        if keep < n {
            out.push(with(c.ops[..keep].to_vec()));
        }
    }
    for i in (0..n).rev().take(8) {
        let slot = u8::try_from(i + 1).unwrap_or(u8::MAX);
        if c.ops[i + 1..].iter().any(|o| o.parent() == slot) {
            continue;
        }
        let ops = c
            .ops
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, o)| {
                let mut o = o.clone();
                o.renumber_after(slot);
                o
            })
            .collect();
        out.push(with(ops));
    }
    if let Some(i) = c
        .ops
        .iter()
        .position(|o| matches!(o, Op::Text(_, v) | Op::Comment(_, v) if !v.is_empty()))
    {
        let mut ops = c.ops.clone();
        if let Op::Text(_, v) | Op::Comment(_, v) = &mut ops[i] {
            v.truncate(v.len() / 2);
        }
        out.push(with(ops));
    }
    out
}

//! Entropy-driven markup recipes and structural reduction.
//!
//! Choices come from closed pools: units that stress UTF-16 and newline
//! handling, every character-reference shape the tokenizer tables define, the
//! document prologues and shells, the balanced element set and the free-form
//! token vocabulary. An exhausted entropy slice reads as zero bytes, so a given
//! slice always yields the same recipe.
use super::model::*;
use super::{Body, Case, MAX_SEGMENTS, Mode, SOUP, Segment};
use crate::captured::{Fault, Plan};

const UNITS: [u16; 22] = [
    0x61, 0x42, 0x30, 0x20, 0x0A, 0x09, 0x2D, 0x3E, 0x3D, 0x2F, 0xE9, 0x4E2D, 0xFEFF, 0xA0, 0xD800,
    0xDC00, 0xFFFD, 0x2028, 0x01, 0x7F, 0x0C, 0xFFFF,
];
const CODES: [u32; 16] = [
    65, 0x1F600, 0, 0xD800, 0xDFFF, 0x11_0000, 0x80, 0x81, 0x9F, 0x0D, 0x0A, 0xFFFE, 0x10_FFFF,
    0xFFFD, 0x20, 0x2603,
];

struct Cur<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl Cur<'_> {
    fn byte(&mut self) -> u8 {
        let b = self.bytes.get(self.at).copied().unwrap_or(0);
        self.at = self.at.saturating_add(1);
        b
    }
    fn below(&mut self, n: usize) -> usize {
        usize::from(self.byte()) % n.max(1)
    }
    fn word(&mut self) -> u64 {
        u64::from(self.byte()) << 8 | u64::from(self.byte())
    }
}
struct Gen<'a> {
    c: Cur<'a>,
    utf16: bool,
    budget: usize,
}
impl Gen<'_> {
    fn unit(&mut self) -> u16 {
        let u = UNITS[self.c.below(UNITS.len())];
        let lone = (0xD800..=0xDFFF).contains(&u);
        if lone && !self.utf16 { 0x61 } else { u }
    }
    fn text_pieces(&mut self, foreign: bool, comment: bool) -> Vec<Piece> {
        let n = 1 + self.c.below(4);
        let mut out: Vec<Piece> = Vec::new();
        for _ in 0..n {
            let kind = self.c.below(20);
            match kind {
                0..=11 => {
                    let u = self.unit();
                    let blocked = comment && matches!(u, 0x2D | 0x3E | 0x21);
                    out.push(Piece::Unit(if blocked { 0x61 } else { u }));
                }
                12 => {
                    // An astral scalar as an adjacent surrogate pair.
                    out.extend([Piece::Unit(0xD83D), Piece::Unit(0xDE00)]);
                }
                13 | 14 if !comment => out.push(Piece::Named(self.c.below(NAMED.len()) as u8)),
                15 | 16 if !comment => out.push(Piece::Numeric {
                    code: CODES[self.c.below(CODES.len())],
                    hex: self.c.byte() % 2 == 1,
                }),
                17 => out.push(Piece::Cr),
                18 => out.push(Piece::CrLf),
                _ => out.push(Piece::Nul),
            }
        }
        // A spelled LF directly after a CR would fuse into one newline.
        for i in 1..out.len() {
            if out[i - 1] == Piece::Cr && out[i] == Piece::Unit(10) {
                out[i] = Piece::Unit(0x6E);
            }
        }
        let flavor = if foreign || comment {
            Flavor::Replaced
        } else {
            Flavor::HtmlText
        };
        if values(&out, flavor).is_empty() {
            out.push(Piece::Unit(0x78));
        }
        out
    }
    fn value_pieces(&mut self) -> Vec<Piece> {
        if self.c.byte().is_multiple_of(5) {
            return Vec::new();
        }
        let mut pieces = self.text_pieces(true, false);
        // Attribute values are never empty after folding when pieces exist.
        pieces.truncate(MAX_PIECES);
        pieces
    }
    fn element(&mut self, tag: usize, class_anchor: bool, depth: usize) -> Item {
        let info = &TAGS[tag];
        let applicable: Vec<usize> = (0..ATTRS.len())
            .filter(|i| attribute_applies(*i, info.ns))
            .collect();
        let count = if self.c.byte().is_multiple_of(3) {
            self.c.below(MAX_ATTRIBUTES + 1)
        } else {
            0
        };
        let attrs = (0..count)
            .map(|_| Attribute {
                name: applicable[self.c.below(applicable.len())] as u8,
                case: self.c.below(4) as u8,
                quote: self.c.below(3) as u8,
                value: self.value_pieces(),
            })
            .collect();
        let mut spell = (self.c.below(4) | self.c.below(4) << 2) as u8;
        let mut shadow = None;
        let mut children = Vec::new();
        let is_host = matches!(tag, DIV | SECTION | SPAN | P);
        if !info.void {
            if is_host && self.c.byte().is_multiple_of(6) {
                shadow = Some(Shadow {
                    bits: self.c.byte() % 32,
                    children: self.items(info.class, class_anchor || tag == A, depth + 1),
                });
            }
            children = self.items(info.class, class_anchor || tag == A, depth + 1);
        }
        let can_close = children.is_empty() && shadow.is_none();
        if can_close && (info.void || info.ns != Ns::Html) && self.c.byte().is_multiple_of(3) {
            spell |= 0x80;
        }
        Item::Elem(Elem {
            tag: tag as u8,
            spell,
            attrs,
            shadow,
            children,
        })
    }
    fn items(&mut self, class: Class, anchor: bool, depth: usize) -> Vec<Item> {
        if depth > MAX_DEPTH - 1 {
            return Vec::new();
        }
        let n = if depth >= 3 {
            self.c.below(3)
        } else {
            self.c.below(5)
        };
        let allowed: Vec<usize> = (0..TAGS.len())
            .filter(|t| class.allows(*t, anchor))
            .collect();
        let mut out: Vec<Item> = Vec::new();
        for _ in 0..n {
            if self.budget == 0 {
                break;
            }
            self.budget -= 1;
            let previous_text = matches!(out.last(), Some(Item::Text(_)));
            let kind = self.c.below(10);
            if kind <= 2 && class.accepts_text() && !previous_text {
                let pieces = self.text_pieces(class.foreign_text(), false);
                out.push(Item::Text(pieces));
            } else if kind == 3 || allowed.is_empty() {
                let pieces = self.text_pieces(false, true);
                out.push(Item::Comment(pieces));
            } else {
                let tag = allowed[self.c.below(allowed.len())];
                let item = self.element(tag, anchor, depth);
                out.push(item);
            }
        }
        out
    }
}

pub(crate) fn synthesize(input: &[u8], capture: bool) -> Option<Case> {
    let mut g = Gen {
        c: Cur {
            bytes: input,
            at: 0,
        },
        utf16: false,
        budget: 30,
    };
    g.utf16 = g.c.byte() % 2 == 1;
    let document = g.c.byte().is_multiple_of(2);
    let mode = if document {
        Mode::Document {
            doctype: g.c.below(DOCTYPES.len()) as u8,
            shell: g.c.below(SHELLS.len()) as u8,
        }
    } else {
        let context = if g.c.byte() % 8 < 6 {
            g.c.below(5)
        } else {
            g.c.below(CONTEXTS.len())
        } as u8;
        Mode::Fragment {
            context,
            shadow: g.c.byte() % 2 == 1,
        }
    };
    let class = match mode {
        Mode::Document { .. } => Some(Class::Flow),
        Mode::Fragment { context, .. } => CONTEXTS[usize::from(context)].class,
    };
    let soup = class.is_none() || g.c.byte() % 4 == 3;
    let mut caps = crate::codec::CAPS;
    if g.c.byte().is_multiple_of(8) {
        let axis = g.c.below(caps.len());
        let raw = g.c.word();
        caps[axis] = match axis {
            0 | 1 => raw % 160,
            16..=18 => raw.saturating_mul(256),
            19 => raw.saturating_mul(8),
            _ => raw % (caps[axis].min(4096) + 1),
        }
        .min(caps[axis]);
    }
    let cancelled = g.c.byte().is_multiple_of(32);
    let body = if let (false, Some(class)) = (soup, class) {
        Body::Exact(g.items(class, false, 0))
    } else {
        let n = 1 + g.c.below(24);
        let mut segments = Vec::new();
        for _ in 0..n {
            if g.c.byte().is_multiple_of(8) {
                let len = 1 + g.c.below(6);
                let mut units = Vec::new();
                for _ in 0..len {
                    units.push(match g.c.below(12) {
                        0 => 0,
                        1 => 13,
                        2 => 10,
                        _ => g.unit(),
                    });
                }
                segments.push(Segment::Units(units));
            } else {
                segments.push(Segment::Token(g.c.below(SOUP.len()) as u8));
            }
        }
        segments.truncate(MAX_SEGMENTS);
        Body::Soup(segments)
    };
    let plan = capture.then(|| Plan {
        scripting: g.c.below(3) as u8,
        registry: g.c.below(3) as u8,
        is_value: g.c.below(3) as u8,
        shadow: g.c.below(8) as u8,
        // Half of the cases import cleanly; the rest inject one fault.
        fault: if g.c.byte().is_multiple_of(2) {
            Fault::None
        } else {
            Fault::ALL[1 + g.c.below(Fault::ALL.len() - 1)]
        },
        pick: g.c.byte(),
    });
    Case::new(mode, g.utf16, body, caps, cancelled, plan).ok()
}

/// All single-step reductions of a list: remove an item, unwrap an element,
/// drop an attribute or shadow, halve a text, or reduce inside a child.
fn list_variants(items: &[Item], out: &mut Vec<Vec<Item>>, limit: usize) {
    for i in 0..items.len() {
        if out.len() >= limit {
            return;
        }
        let mut removed = items.to_vec();
        removed.remove(i);
        out.push(removed);
        match &items[i] {
            Item::Text(pieces) | Item::Comment(pieces) if pieces.len() > 1 => {
                let mut halved = items.to_vec();
                let keep = pieces[..pieces.len() / 2].to_vec();
                halved[i] = match &items[i] {
                    Item::Text(_) => Item::Text(keep),
                    _ => Item::Comment(keep),
                };
                out.push(halved);
            }
            Item::Elem(e) => {
                let mut unwrapped = items[..i].to_vec();
                unwrapped.extend(e.children.iter().cloned());
                unwrapped.extend(items[i + 1..].iter().cloned());
                out.push(unwrapped);
                if e.shadow.is_some() {
                    let mut plain = e.clone();
                    plain.shadow = None;
                    let mut v = items.to_vec();
                    v[i] = Item::Elem(plain);
                    out.push(v);
                }
                for k in 0..e.attrs.len() {
                    let mut fewer = e.clone();
                    fewer.attrs.remove(k);
                    let mut v = items.to_vec();
                    v[i] = Item::Elem(fewer);
                    out.push(v);
                }
                let mut inner = Vec::new();
                list_variants(&e.children, &mut inner, limit);
                for children in inner {
                    let mut changed = e.clone();
                    changed.children = children;
                    let mut v = items.to_vec();
                    v[i] = Item::Elem(changed);
                    out.push(v);
                }
                if let Some(s) = &e.shadow {
                    let mut inner = Vec::new();
                    list_variants(&s.children, &mut inner, limit);
                    for children in inner {
                        let mut changed = e.clone();
                        changed.shadow = Some(Shadow {
                            bits: s.bits,
                            children,
                        });
                        let mut v = items.to_vec();
                        v[i] = Item::Elem(changed);
                        out.push(v);
                    }
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn candidates(case: &Case) -> Vec<Case> {
    let with = |body| Case {
        mode: case.mode,
        utf16: case.utf16,
        body,
        caps: case.caps,
        cancelled: case.cancelled,
        plan: case.plan,
    };
    let mut out = Vec::new();
    match &case.body {
        Body::Exact(items) => {
            if !items.is_empty() {
                out.push(with(Body::Exact(Vec::new())));
            }
            let mut variants = Vec::new();
            list_variants(items, &mut variants, 48);
            out.extend(variants.into_iter().map(|v| with(Body::Exact(v))));
        }
        Body::Soup(segments) => {
            let n = segments.len();
            for keep in [0, n / 2, n.saturating_sub(1)] {
                if keep < n {
                    out.push(with(Body::Soup(segments[..keep].to_vec())));
                }
            }
            for i in 0..n.min(16) {
                let mut v = segments.clone();
                v.remove(i);
                out.push(with(Body::Soup(v)));
            }
            for (i, s) in segments.iter().enumerate().take(16) {
                if let Segment::Units(u) = s
                    && u.len() > 1
                {
                    let mut v = segments.clone();
                    v[i] = Segment::Units(u[..u.len() / 2].to_vec());
                    out.push(with(Body::Soup(v)));
                }
            }
        }
    }
    out.truncate(64);
    out
}

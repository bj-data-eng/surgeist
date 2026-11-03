//! HMK1: strict lossless little-endian markup recipes.
//!
//! | Bytes | Meaning |
//! | --- | --- |
//! | 0..4 | `HMK1`, or `HCP1` for a capture-import case |
//! | 4 | mode: 0 document, 1 fragment |
//! | 5 | source: 0 UTF-8, 1 UTF-16 |
//! | 6 | document prologue or fragment context |
//! | 7 | document shell, or fragment declarative-shadow permission |
//! | 8 | body: 0 balanced recipe, 1 free-form token soup |
//! | 9 | flags: bit 0 cancelled before the parse |
//! | 10..162 | nineteen u64 parser caps |
//! | 162..168 | `HCP1` only: scripting, registry, `is` slot, shadow facts, fault, pick |
//! | 162.. or 168.. | the body, with no trailing bytes |
use super::model::*;
use super::{Body, Case, Mode, SOUP, Segment};
use crate::captured::{Fault, Plan};
use crate::error;
use surgeist_fuzz::Result;

pub(crate) const HEADER: usize = 162;

struct Reader<'a> {
    input: &'a [u8],
    at: usize,
    items: usize,
    pieces: usize,
}
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| error("length overflow"))?;
        let s = self
            .input
            .get(self.at..end)
            .ok_or_else(|| error("truncated markup case"))?;
        self.at = end;
        Ok(s)
    }
    fn byte(&mut self) -> Result<u8> {
        self.take(1)?
            .first()
            .copied()
            .ok_or_else(|| error("truncated"))
    }
    fn u16(&mut self) -> Result<u16> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn pieces(&mut self) -> Result<Vec<Piece>> {
        let n = usize::from(self.u16()?);
        self.pieces += n;
        if n > MAX_PIECES || self.pieces > MAX_PIECES * 2 {
            return Err(error("piece ceiling"));
        }
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(match self.byte()? {
                0 => Piece::Unit(self.u16()?),
                1 => Piece::Named(self.byte()?),
                2 => {
                    let code = self.u32()?;
                    let hex = match self.byte()? {
                        0 => false,
                        1 => true,
                        _ => return Err(error("numeric radix tag")),
                    };
                    Piece::Numeric { code, hex }
                }
                3 => Piece::Cr,
                4 => Piece::CrLf,
                5 => Piece::Nul,
                _ => return Err(error("piece tag")),
            });
        }
        Ok(out)
    }
    fn items(&mut self, depth: usize) -> Result<Vec<Item>> {
        if depth > MAX_DEPTH {
            return Err(error("recipe depth"));
        }
        let n = usize::from(self.byte()?);
        self.items += n;
        if self.items > MAX_ITEMS {
            return Err(error("recipe item ceiling"));
        }
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(match self.byte()? {
                0 => Item::Text(self.pieces()?),
                1 => Item::Comment(self.pieces()?),
                2 => {
                    let tag = self.byte()?;
                    let spell = self.byte()?;
                    let count = usize::from(self.byte()?);
                    if count > MAX_ATTRIBUTES {
                        return Err(error("attribute ceiling"));
                    }
                    let mut attrs = Vec::with_capacity(count);
                    for _ in 0..count {
                        attrs.push(Attribute {
                            name: self.byte()?,
                            case: self.byte()?,
                            quote: self.byte()?,
                            value: self.pieces()?,
                        });
                    }
                    let shadow = match self.byte()? {
                        0 => None,
                        1 => Some(Shadow {
                            bits: self.byte()?,
                            children: self.items(depth + 1)?,
                        }),
                        _ => return Err(error("shadow tag")),
                    };
                    Item::Elem(Elem {
                        tag,
                        spell,
                        attrs,
                        shadow,
                        children: self.items(depth + 1)?,
                    })
                }
                _ => return Err(error("item tag")),
            });
        }
        Ok(out)
    }
}

pub(crate) fn decode(input: &[u8], capture: bool) -> Result<Case> {
    let magic: &[u8] = if capture { b"HCP1" } else { b"HMK1" };
    if input.len() < HEADER + 6 * usize::from(capture) || input.get(..4) != Some(magic) {
        return Err(error("markup header"));
    }
    let mut r = Reader {
        input,
        at: 4,
        items: 0,
        pieces: 0,
    };
    let mode = r.byte()?;
    let utf16 = match r.byte()? {
        0 => false,
        1 => true,
        _ => return Err(error("source tag")),
    };
    let (a, b) = (r.byte()?, r.byte()?);
    let mode = match mode {
        0 => Mode::Document {
            doctype: a,
            shell: b,
        },
        1 => Mode::Fragment {
            context: a,
            shadow: match b {
                0 => false,
                1 => true,
                _ => return Err(error("shadow permission tag")),
            },
        },
        _ => return Err(error("mode tag")),
    };
    let body_kind = r.byte()?;
    let cancelled = match r.byte()? {
        0 => false,
        1 => true,
        _ => return Err(error("flag bits")),
    };
    let mut caps = [0; 19];
    for cap in &mut caps {
        let b = r.take(8)?;
        *cap = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
    }
    let plan = if capture {
        let b = r.take(6)?;
        let plan = Plan {
            scripting: b[0],
            registry: b[1],
            is_value: b[2],
            shadow: b[3],
            fault: Fault::from_tag(b[4]).ok_or_else(|| error("fault tag"))?,
            pick: b[5],
        };
        if !plan.valid() {
            return Err(error("capture fact tag"));
        }
        Some(plan)
    } else {
        None
    };
    let body = match body_kind {
        0 => Body::Exact(r.items(0)?),
        1 => {
            let n = usize::from(r.u16()?);
            if n > super::MAX_SEGMENTS {
                return Err(error("soup segment ceiling"));
            }
            let mut segments = Vec::with_capacity(n);
            for _ in 0..n {
                segments.push(match r.byte()? {
                    0 => {
                        let t = r.byte()?;
                        if usize::from(t) >= SOUP.len() {
                            return Err(error("soup token"));
                        }
                        Segment::Token(t)
                    }
                    1 => {
                        let len = usize::from(r.byte()?);
                        if len > super::MAX_SEGMENT_UNITS {
                            return Err(error("soup unit ceiling"));
                        }
                        let mut units = Vec::with_capacity(len);
                        for _ in 0..len {
                            units.push(r.u16()?);
                        }
                        Segment::Units(units)
                    }
                    _ => return Err(error("soup segment tag")),
                });
            }
            Body::Soup(segments)
        }
        _ => return Err(error("body tag")),
    };
    if r.at != input.len() {
        return Err(error("trailing markup bytes"));
    }
    Case::new(mode, utf16, body, caps, cancelled, plan)
}

struct Writer(Vec<u8>);
impl Writer {
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn pieces(&mut self, pieces: &[Piece]) -> Result<()> {
        self.u16(u16::try_from(pieces.len()).map_err(|_| error("piece count"))?);
        for p in pieces {
            match p {
                Piece::Unit(u) => {
                    self.0.push(0);
                    self.u16(*u);
                }
                Piece::Named(i) => self.0.extend([1, *i]),
                Piece::Numeric { code, hex } => {
                    self.0.push(2);
                    self.0.extend_from_slice(&code.to_le_bytes());
                    self.0.push(u8::from(*hex));
                }
                Piece::Cr => self.0.push(3),
                Piece::CrLf => self.0.push(4),
                Piece::Nul => self.0.push(5),
            }
        }
        Ok(())
    }
    fn items(&mut self, items: &[Item]) -> Result<()> {
        self.0
            .push(u8::try_from(items.len()).map_err(|_| error("item count"))?);
        for item in items {
            match item {
                Item::Text(p) => {
                    self.0.push(0);
                    self.pieces(p)?;
                }
                Item::Comment(p) => {
                    self.0.push(1);
                    self.pieces(p)?;
                }
                Item::Elem(e) => {
                    self.0.extend([2, e.tag, e.spell]);
                    self.0
                        .push(u8::try_from(e.attrs.len()).map_err(|_| error("attribute count"))?);
                    for a in &e.attrs {
                        self.0.extend([a.name, a.case, a.quote]);
                        self.pieces(&a.value)?;
                    }
                    match &e.shadow {
                        None => self.0.push(0),
                        Some(s) => {
                            self.0.extend([1, s.bits]);
                            self.items(&s.children)?;
                        }
                    }
                    self.items(&e.children)?;
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn encode(case: &Case) -> Result<Vec<u8>> {
    case.validate()?;
    let mut w = Writer(Vec::with_capacity(HEADER + 256));
    w.0.extend_from_slice(if case.plan.is_some() {
        b"HCP1"
    } else {
        b"HMK1"
    });
    let (mode, a, b) = match case.mode {
        Mode::Document { doctype, shell } => (0, doctype, shell),
        Mode::Fragment { context, shadow } => (1, context, u8::from(shadow)),
    };
    w.0.extend([
        mode,
        u8::from(case.utf16),
        a,
        b,
        match case.body {
            Body::Exact(_) => 0,
            Body::Soup(_) => 1,
        },
        u8::from(case.cancelled),
    ]);
    for cap in case.caps {
        w.0.extend_from_slice(&cap.to_le_bytes());
    }
    if let Some(p) = &case.plan {
        w.0.extend([
            p.scripting,
            p.registry,
            p.is_value,
            p.shadow,
            p.fault.tag(),
            p.pick,
        ]);
    }
    match &case.body {
        Body::Exact(items) => w.items(items)?,
        Body::Soup(segments) => {
            w.u16(u16::try_from(segments.len()).map_err(|_| error("segment count"))?);
            for s in segments {
                match s {
                    Segment::Token(t) => w.0.extend([0, *t]),
                    Segment::Units(units) => {
                        w.0.extend([
                            1,
                            u8::try_from(units.len()).map_err(|_| error("segment units"))?,
                        ]);
                        for u in units {
                            w.u16(*u);
                        }
                    }
                }
            }
        }
    }
    Ok(w.0)
}

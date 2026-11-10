//! Prototype for #2228 (scratch, not for merge): the run-scanning tokenizer of
//! the design note, writing decoded code units straight into a text arena.
//!
//! The scanner covers the data, RCDATA, RAWTEXT, script data, tag, attribute,
//! comment, doctype and bogus-comment states, newline normalization, NUL
//! handling, numeric character references with the C1 replacement table, and a
//! 14-name subset of named references (the full 2,231-name table is not
//! included, so reference-heavy input is measured without that lookup). Tag and
//! attribute names resolve through a perfect-hash-style table of known names, so
//! the sink receives integer ids. A per-token credit check and a cancellation
//! load stand in for the meter.
//!
//! Correctness is checked against counts computed independently by
//! `tmp/html-issues/work/2228/scripts/docstats.py` and, for each document, by
//! comparing the arena produced from a UTF-8 source with the one produced from
//! the same text as UTF-16.
//!
//! Run: `cargo run --release --offline --locked -j 1 -p surgeist-html-parser --example model_scan`

use std::{
    hint::black_box,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

#[path = "../benches/support/documents.rs"]
mod documents;

trait Unit: Copy {
    const UTF8: bool;
    fn get(self) -> u32;
}
impl Unit for u8 {
    const UTF8: bool = true;
    #[inline(always)]
    fn get(self) -> u32 {
        u32::from(self)
    }
}
impl Unit for u16 {
    const UTF8: bool = false;
    #[inline(always)]
    fn get(self) -> u32 {
        u32::from(self)
    }
}

const ELEMENTS: &[&str] = &[
    "a", "abbr", "address", "area", "article", "aside", "audio", "b", "base", "blockquote", "body",
    "br", "button", "canvas", "caption", "code", "col", "colgroup", "dd", "details", "dialog",
    "div", "dl", "dt", "em", "fieldset", "figure", "footer", "form", "h1", "h2", "h3", "h4", "h5",
    "h6", "head", "header", "hr", "html", "i", "iframe", "img", "input", "label", "legend", "li",
    "link", "main", "meta", "nav", "noscript", "object", "ol", "optgroup", "option", "p", "pre",
    "script", "section", "select", "small", "source", "span", "strong", "style", "sub", "sup",
    "table", "tbody", "td", "template", "textarea", "tfoot", "th", "thead", "title", "tr", "u",
    "ul", "video",
];
const ATTRIBUTES: &[&str] = &[
    "alt", "charset", "class", "colspan", "content", "for", "height", "href", "id", "lang", "name",
    "rel", "rowspan", "scope", "src", "style", "title", "type", "value", "width",
];
const ATTR_BASE: u32 = 1_000;
const DYN_BASE: u32 = 100_000;

#[inline(always)]
fn fnv(mut h: u32, u: u32) -> u32 {
    h = (h ^ u).wrapping_mul(0x0100_0193);
    h
}
const FNV0: u32 = 0x811c_9dc5;

struct Known {
    slots: Vec<u16>, // id + 1
    names: Vec<&'static str>,
    base: u32,
}
impl Known {
    fn new(names: &[&'static str], base: u32) -> Self {
        let mut slots = vec![0_u16; 512];
        for (i, n) in names.iter().enumerate() {
            let h = n.bytes().fold(FNV0, |h, b| fnv(h, u32::from(b)));
            let mut j = h as usize & 511;
            while slots[j] != 0 {
                j = (j + 1) & 511;
            }
            slots[j] = i as u16 + 1;
        }
        Self {
            slots,
            names: names.to_vec(),
            base,
        }
    }
    #[inline]
    fn find(&self, h: u32, units: &[u16]) -> Option<u32> {
        let mut j = h as usize & 511;
        loop {
            let s = self.slots[j];
            if s == 0 {
                return None;
            }
            let n = self.names[s as usize - 1].as_bytes();
            if n.len() == units.len() && n.iter().zip(units).all(|(&a, &b)| u16::from(a) == b) {
                return Some(self.base + u32::from(s) - 1);
            }
            j = (j + 1) & 511;
        }
    }
}

#[derive(Clone, Copy)]
struct AttrTok {
    name: u32,
    start: u32,
    len: u32,
}

/// Token sink. The checksum makes the counts observable.
struct Sink {
    starts: u64,
    ends: u64,
    texts: u64,
    text_units: u64,
    comments: u64,
    doctypes: u64,
    attrs: u64,
    attr_units: u64,
    dup_dropped: u64,
    sum: u64,
}

struct Scanner<'a, U: Unit> {
    src: &'a [U],
    pos: usize,
    arena: Vec<u16>,
    attrs: Vec<AttrTok>,
    elements: Known,
    attributes: Known,
    dynamic: Vec<(u32, u32, u32)>, // start, len, hash
    credits: u64,
    credit_cap: u64,
    cancel: &'a AtomicBool,
    sink: Sink,
    stopped: bool,
}

/// Unit classes for run scanning. `stop` ends a run: the terminals plus CR, NUL and
/// (UTF-8) non-ASCII bytes, which the scanner handles in place. `term` returns
/// control to the caller.
struct Tabs {
    stop: [bool; 256],
    term: [bool; 256],
}
fn tabs(terms: &[u8]) -> Tabs {
    let mut stop = [false; 256];
    let mut term = [false; 256];
    for b in [b'\r', 0_u8] {
        stop[b as usize] = true;
    }
    for b in 0x80..=0xff_usize {
        stop[b] = true;
    }
    for &b in terms {
        stop[b as usize] = true;
        term[b as usize] = true;
    }
    Tabs { stop, term }
}

const C1: [u16; 32] = [
    0x20AC, 0x81, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039,
    0x0152, 0x8D, 0x017D, 0x8F, 0x90, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014,
    0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0x9D, 0x017E, 0x0178,
];
const NAMED: &[(&str, u16)] = &[
    ("amp", b'&' as u16),
    ("lt", b'<' as u16),
    ("gt", b'>' as u16),
    ("quot", b'"' as u16),
    ("apos", b'\'' as u16),
    ("nbsp", 0xA0),
    ("copy", 0xA9),
    ("reg", 0xAE),
    ("hellip", 0x2026),
    ("mdash", 0x2014),
    ("ndash", 0x2013),
    ("lsquo", 0x2018),
    ("rsquo", 0x2019),
    ("trade", 0x2122),
];
/// Named references that the specification also accepts without a semicolon.
const LEGACY: &[&str] = &["amp", "lt", "gt", "quot", "nbsp", "copy", "reg"];

impl<'a, U: Unit> Scanner<'a, U> {
    fn new(src: &'a [U], cancel: &'a AtomicBool, reserve: usize) -> Self {
        Self {
            src,
            pos: 0,
            arena: Vec::with_capacity(reserve),
            attrs: Vec::with_capacity(64),
            elements: Known::new(ELEMENTS, 0),
            attributes: Known::new(ATTRIBUTES, ATTR_BASE),
            dynamic: Vec::new(),
            credits: 0,
            credit_cap: u64::MAX,
            cancel,
            sink: Sink {
                starts: 0,
                ends: 0,
                texts: 0,
                text_units: 0,
                comments: 0,
                doctypes: 0,
                attrs: 0,
                attr_units: 0,
                dup_dropped: 0,
                sum: 0,
            },
            stopped: false,
        }
    }

    #[inline(always)]
    fn at(&self, i: usize) -> Option<u32> {
        self.src.get(i).map(|u| u.get())
    }

    /// Charges one token or one 256-unit block before the work it covers.
    #[inline(always)]
    fn charge(&mut self, credits: u64) -> bool {
        if self.cancel.load(Ordering::Relaxed) || self.credits + credits > self.credit_cap {
            self.stopped = true;
            return false;
        }
        self.credits += credits;
        true
    }

    /// Appends src[from..to], which holds no stop byte, to the arena.
    #[inline(always)]
    fn copy_plain(&mut self, from: usize, to: usize) {
        let run = &self.src[from..to];
        self.arena.extend(run.iter().map(|u| u.get() as u16));
    }

    #[inline(always)]
    fn push_cp(&mut self, cp: u32) {
        if cp >= 0x10000 {
            let v = cp - 0x10000;
            self.arena.push(0xD800 | (v >> 10) as u16);
            self.arena.push(0xDC00 | (v & 0x3FF) as u16);
        } else {
            self.arena.push(cp as u16);
        }
    }

    /// Decodes one UTF-8 sequence at `i` (a lead or stray byte >= 0x80).
    #[cold]
    fn decode8(&self, i: usize) -> (u32, usize) {
        let b0 = self.src[i].get();
        let (need, min, init) = match b0 {
            0xC2..=0xDF => (1, 0x80, b0 & 0x1F),
            0xE0..=0xEF => (2, 0x800, b0 & 0x0F),
            0xF0..=0xF4 => (3, 0x10000, b0 & 0x07),
            _ => return (0xFFFD, 1),
        };
        let mut cp = init;
        for k in 1..=need {
            match self.at(i + k) {
                Some(b) if b & 0xC0 == 0x80 => cp = (cp << 6) | (b & 0x3F),
                _ => return (0xFFFD, k),
            }
        }
        if cp < min || cp > 0x10FFFF || (0xD800..=0xDFFF).contains(&cp) {
            (0xFFFD, need + 1)
        } else {
            (cp, need + 1)
        }
    }

    /// One special unit at `pos` in text-like content. `nul` is what a NUL becomes.
    #[inline]
    fn special(&mut self, nul: u32) {
        let u = self.src[self.pos].get();
        if u == 0x0D {
            self.arena.push(0x0A);
            self.pos += 1;
            if self.at(self.pos) == Some(0x0A) {
                self.pos += 1;
            }
        } else if u == 0 {
            self.arena.push(nul as u16);
            self.pos += 1;
        } else if U::UTF8 && u >= 0x80 {
            let (cp, n) = self.decode8(self.pos);
            self.push_cp(cp);
            self.pos += n;
        } else {
            self.arena.push(u as u16);
            self.pos += 1;
        }
    }

    /// Appends a run of text content; returns at a terminal unit (or the end of
    /// the input). Newline normalization, NUL and non-ASCII are handled here.
    fn text_run(&mut self, t: &Tabs, nul: u32) -> bool {
        loop {
            let start = self.pos;
            let mut i = start;
            let end = (start + 256).min(self.src.len());
            while i < end {
                let u = self.src[i].get();
                if u < 256 && t.stop[u as usize] {
                    break;
                }
                i += 1;
            }
            if i > start {
                if !self.charge((i - start) as u64 * 4 + 8) {
                    return false;
                }
                self.copy_plain(start, i);
                self.pos = i;
            }
            if i == end && i < self.src.len() {
                let u = self.src[i].get();
                if !(u < 256 && t.stop[u as usize]) {
                    continue; // block boundary, no stop unit yet
                }
            }
            let Some(u) = self.at(self.pos) else {
                return true;
            };
            if u < 256 && t.term[u as usize] {
                return true;
            }
            self.special(nul);
        }
    }

    fn char_ref(&mut self, in_attr: bool) {
        // At '&'.
        let start = self.pos;
        self.pos += 1;
        match self.at(self.pos) {
            Some(0x23) => {
                self.pos += 1;
                let hex = matches!(self.at(self.pos), Some(0x78 | 0x58));
                if hex {
                    self.pos += 1;
                }
                let digits = self.pos;
                let mut cp: u32 = 0;
                let mut overflow = false;
                while let Some(u) = self.at(self.pos) {
                    let d = match (hex, u) {
                        (_, 0x30..=0x39) => u - 0x30,
                        (true, 0x41..=0x46) => u - 0x41 + 10,
                        (true, 0x61..=0x66) => u - 0x61 + 10,
                        _ => break,
                    };
                    cp = cp.saturating_mul(if hex { 16 } else { 10 }).saturating_add(d);
                    if cp > 0x10FFFF {
                        overflow = true;
                        cp = 0x110000;
                    }
                    self.pos += 1;
                }
                if self.pos == digits {
                    // No digits: the ampersand and the markers stay text.
                    self.copy_ascii(start, self.pos);
                    return;
                }
                if self.at(self.pos) == Some(0x3B) {
                    self.pos += 1;
                }
                let cp = if overflow || cp > 0x10FFFF || cp == 0 || (0xD800..=0xDFFF).contains(&cp)
                {
                    0xFFFD
                } else if (0x80..=0x9F).contains(&cp) {
                    u32::from(C1[(cp - 0x80) as usize])
                } else {
                    cp
                };
                self.push_cp(cp);
            }
            Some(u) if (u as u8 as u32 == u) && (u as u8).is_ascii_alphanumeric() => {
                let name_start = self.pos;
                let mut j = self.pos;
                while let Some(c) = self.at(j) {
                    if c < 128 && (c as u8).is_ascii_alphanumeric() {
                        j += 1;
                    } else {
                        break;
                    }
                }
                let semi = self.at(j) == Some(0x3B);
                let name: Vec<u8> = (name_start..j).map(|k| self.src[k].get() as u8).collect();
                let mut hit = None;
                if semi {
                    hit = NAMED
                        .iter()
                        .find(|(n, _)| n.as_bytes() == &name[..])
                        .map(|&(_, c)| (c, j + 1));
                } else {
                    // Legacy form: the longest legacy prefix, not followed by '=' or an
                    // alphanumeric in an attribute value.
                    for l in LEGACY {
                        if name.starts_with(l.as_bytes()) {
                            let end = name_start + l.len();
                            let next = self.at(end);
                            let blocked = in_attr
                                && next.is_some_and(|c| {
                                    c == 0x3D || (c < 128 && (c as u8).is_ascii_alphanumeric())
                                });
                            if !blocked {
                                let c = NAMED.iter().find(|(n, _)| n == l).map(|&(_, c)| c);
                                hit = c.map(|c| (c, end));
                            }
                            break;
                        }
                    }
                }
                match hit {
                    Some((c, end)) => {
                        self.arena.push(c);
                        self.pos = end;
                    }
                    None => {
                        self.copy_ascii(start, if semi { j + 1 } else { j });
                    }
                }
            }
            _ => self.arena.push(0x26),
        }
    }

    fn copy_ascii(&mut self, from: usize, to: usize) {
        self.copy_plain(from, to);
        self.pos = to;
    }

    fn emit_text(&mut self, start: usize) {
        let n = self.arena.len() - start;
        if n > 0 {
            self.sink.texts += 1;
            self.sink.text_units += n as u64;
            self.sink.sum = self.sink.sum.wrapping_add(n as u64 * 31 + start as u64);
        }
    }

    /// Data (or RCDATA with `refs`) content until `<` that starts markup.
    fn content(&mut self, t: &Tabs, refs: bool) -> bool {
        let start = self.arena.len();
        loop {
            if !self.text_run(t, 0) {
                return false;
            }
            match self.at(self.pos) {
                None => break,
                Some(0x26) if refs => self.char_ref(false),
                Some(0x26) => {
                    self.arena.push(0x26);
                    self.pos += 1;
                }
                Some(_) => {
                    // '<': markup only if a tag, declaration or bogus comment starts.
                    match self.at(self.pos + 1) {
                        Some(c)
                            if c < 128
                                && ((c as u8).is_ascii_alphabetic()
                                    || matches!(c as u8, b'/' | b'!' | b'?')) =>
                        {
                            break;
                        }
                        _ => {
                            self.arena.push(0x3C);
                            self.pos += 1;
                        }
                    }
                }
            }
        }
        self.emit_text(start);
        true
    }

    fn lower_name(&mut self, extra_stop: &dyn Fn(u32) -> bool) -> (u32, usize) {
        // Writes the lowercased name at the arena tail; returns the hash and start.
        let start = self.arena.len();
        let mut h = FNV0;
        while let Some(u) = self.at(self.pos) {
            if extra_stop(u) {
                break;
            }
            let c = if (0x41..=0x5A).contains(&u) {
                u + 32
            } else if u == 0 {
                0xFFFD
            } else {
                u
            };
            h = fnv(h, c);
            self.arena.push(c as u16);
            self.pos += 1;
        }
        (h, start)
    }

    fn resolve(&mut self, h: u32, start: usize, attr: bool) -> u32 {
        let units = &self.arena[start..];
        let known = if attr { &self.attributes } else { &self.elements };
        if let Some(id) = known.find(h, units) {
            self.arena.truncate(start);
            return id;
        }
        for (k, &(s, l, eh)) in self.dynamic.iter().enumerate() {
            if eh == h && self.arena[s as usize..(s + l) as usize] == self.arena[start..] {
                self.arena.truncate(start);
                return DYN_BASE + k as u32;
            }
        }
        self.dynamic
            .push((start as u32, (self.arena.len() - start) as u32, h));
        DYN_BASE + self.dynamic.len() as u32 - 1
    }

    fn is_ws(u: u32) -> bool {
        matches!(u, 0x09 | 0x0A | 0x0C | 0x0D | 0x20)
    }

    fn skip_ws(&mut self) {
        while self.at(self.pos).is_some_and(Self::is_ws) {
            self.pos += 1;
        }
    }

    /// At '<' followed by a letter or '/': a start or end tag. Returns the raw
    /// text mode to enter after a start tag.
    fn tag(&mut self) -> Option<(u32, bool)> {
        if !self.charge(32) {
            return None;
        }
        let tag_start_arena = self.arena.len();
        self.pos += 1;
        let end_tag = self.at(self.pos) == Some(0x2F);
        if end_tag {
            self.pos += 1;
        }
        let (h, start) = self.lower_name(&|u| Self::is_ws(u) || u == 0x2F || u == 0x3E);
        let name = self.resolve(h, start, false);
        let after_name = self.arena.len();
        self.attrs.clear();
        let mut self_closing = false;
        let mut dup = 0;
        loop {
            self.skip_ws();
            match self.at(self.pos) {
                None => {
                    // EOF in tag: the tag is dropped.
                    self.arena.truncate(tag_start_arena);
                    return None;
                }
                Some(0x3E) => {
                    self.pos += 1;
                    break;
                }
                Some(0x2F) => {
                    self.pos += 1;
                    if self.at(self.pos) == Some(0x3E) {
                        self_closing = true;
                        self.pos += 1;
                        break;
                    }
                    continue;
                }
                Some(_) => {}
            }
            if !self.charge(24) {
                return None;
            }
            let first = self.pos == 0; // never true; keeps '=' at name start handled below
            let _ = first;
            // Attribute name ('=' as the first unit belongs to the name).
            let mut lead = false;
            if self.at(self.pos) == Some(0x3D) {
                lead = true;
            }
            let nstart = self.arena.len();
            let mut h = FNV0;
            if lead {
                h = fnv(h, 0x3D);
                self.arena.push(0x3D);
                self.pos += 1;
            }
            while let Some(u) = self.at(self.pos) {
                if Self::is_ws(u) || matches!(u, 0x2F | 0x3E | 0x3D) {
                    break;
                }
                let c = if (0x41..=0x5A).contains(&u) {
                    u + 32
                } else if u == 0 {
                    0xFFFD
                } else {
                    u
                };
                h = fnv(h, c);
                self.arena.push(c as u16);
                self.pos += 1;
            }
            let id = if end_tag { 0 } else { self.resolve(h, nstart, true) };
            self.skip_ws();
            let vstart = self.arena.len();
            if self.at(self.pos) == Some(0x3D) {
                self.pos += 1;
                self.skip_ws();
                match self.at(self.pos) {
                    Some(q @ (0x22 | 0x27)) => {
                        self.pos += 1;
                        let t = tabs(&[b'&', q as u8]);
                        loop {
                            if !self.text_run(&t, 0xFFFD) {
                                return None;
                            }
                            match self.at(self.pos) {
                                None => {
                                    self.arena.truncate(tag_start_arena);
                                    return None;
                                }
                                Some(0x26) => self.char_ref(true),
                                Some(c) if c == q => {
                                    self.pos += 1;
                                    break;
                                }
                                Some(_) => {
                                    self.arena.push(0x3C);
                                    self.pos += 1;
                                }
                            }
                        }
                    }
                    Some(0x3E) | None => {}
                    Some(_) => {
                        let t = tabs(&[b'&', b' ', b'\t', b'\n', 0x0C, b'\r', b'>']);
                        loop {
                            if !self.text_run(&t, 0xFFFD) {
                                return None;
                            }
                            match self.at(self.pos) {
                                None => break,
                                Some(0x26) => self.char_ref(true),
                                Some(c) if Self::is_ws(c) || c == 0x3E => break,
                                Some(c) => {
                                    self.arena.push(c as u16);
                                    self.pos += 1;
                                }
                            }
                        }
                    }
                }
            }
            let len = (self.arena.len() - vstart) as u32;
            if end_tag {
                self.arena.truncate(after_name);
                continue;
            }
            if self.attrs.iter().any(|a| a.name == id) {
                self.arena.truncate(vstart);
                dup += 1;
            } else {
                self.attrs.push(AttrTok {
                    name: id,
                    start: vstart as u32,
                    len,
                });
            }
        }
        if end_tag {
            self.sink.ends += 1;
            self.sink.sum = self.sink.sum.wrapping_add(u64::from(name));
            self.arena.truncate(after_name);
            return Some((0, false));
        }
        self.sink.starts += 1;
        self.sink.attrs += self.attrs.len() as u64;
        self.sink.dup_dropped += dup;
        for a in &self.attrs {
            self.sink.attr_units += u64::from(a.len);
            self.sink.sum = self
                .sink
                .sum
                .wrapping_add(u64::from(a.name) * 7 + u64::from(a.start) + u64::from(a.len));
        }
        self.sink.sum = self.sink.sum.wrapping_add(u64::from(name));
        let _ = self_closing;
        Some((name, true))
    }

    fn markup_declaration(&mut self) -> bool {
        // At "<!".
        if !self.charge(32) {
            return false;
        }
        let rest = self.pos + 2;
        if self.at(rest) == Some(0x2D) && self.at(rest + 1) == Some(0x2D) {
            self.pos = rest + 2;
            let start = self.arena.len();
            // "<!-->" and "<!--->" are empty comments.
            if self.at(self.pos) == Some(0x3E) {
                self.pos += 1;
            } else if self.at(self.pos) == Some(0x2D) && self.at(self.pos + 1) == Some(0x3E) {
                self.pos += 2;
            } else {
                loop {
                    let Some(u) = self.at(self.pos) else { break };
                    if u == 0x2D && self.at(self.pos + 1) == Some(0x2D) {
                        if self.at(self.pos + 2) == Some(0x3E) {
                            self.pos += 3;
                            break;
                        }
                        if self.at(self.pos + 2) == Some(0x21) && self.at(self.pos + 3) == Some(0x3E) {
                            self.pos += 4;
                            break;
                        }
                    }
                    self.special_or_plain();
                }
            }
            self.sink.comments += 1;
            self.arena.truncate(start);
            return true;
        }
        let is_doctype = (0..7).all(|k| {
            self.at(rest + k)
                .is_some_and(|u| u < 128 && (u as u8).eq_ignore_ascii_case(&b"doctype"[k]))
        });
        if is_doctype {
            self.pos = rest + 7;
            self.skip_ws();
            let (h, start) = self.lower_name(&|u| Self::is_ws(u) || u == 0x3E);
            let _ = self.resolve(h, start, false);
            while let Some(u) = self.at(self.pos) {
                self.pos += 1;
                if u == 0x3E {
                    break;
                }
            }
            self.sink.doctypes += 1;
            return true;
        }
        self.bogus_comment(self.pos + 2);
        true
    }

    fn special_or_plain(&mut self) {
        let u = self.src[self.pos].get();
        if u == 0x0D || u == 0 || (U::UTF8 && u >= 0x80) {
            self.special(0xFFFD);
        } else {
            self.arena.push(u as u16);
            self.pos += 1;
        }
    }

    fn bogus_comment(&mut self, from: usize) {
        self.pos = from;
        let start = self.arena.len();
        while let Some(u) = self.at(self.pos) {
            if u == 0x3E {
                self.pos += 1;
                break;
            }
            self.special_or_plain();
        }
        self.sink.comments += 1;
        self.arena.truncate(start);
    }

    /// Script data, RAWTEXT and RCDATA: content to the matching end tag.
    fn raw(&mut self, name: u32, refs: bool) -> bool {
        let tag: &[u8] = ELEMENTS[name as usize].as_bytes();
        let start = self.arena.len();
        let t = tabs(&[b'<', b'&']);
        loop {
            if !self.text_run(&t, 0xFFFD) {
                return false;
            }
            match self.at(self.pos) {
                None => break,
                Some(0x26) if refs => self.char_ref(false),
                Some(0x26) => {
                    self.arena.push(0x26);
                    self.pos += 1;
                }
                Some(_) => {
                    let end_here = self.at(self.pos + 1) == Some(0x2F)
                        && (0..tag.len()).all(|k| {
                            self.at(self.pos + 2 + k)
                                .is_some_and(|u| u < 128 && (u as u8).eq_ignore_ascii_case(&tag[k]))
                        })
                        && self
                            .at(self.pos + 2 + tag.len())
                            .is_none_or(|u| Self::is_ws(u) || u == 0x2F || u == 0x3E);
                    if end_here {
                        break;
                    }
                    self.arena.push(0x3C);
                    self.pos += 1;
                }
            }
        }
        self.emit_text(start);
        true
    }

    fn run(&mut self) {
        let data = tabs(&[b'<', b'&']);
        let lt_name = |s: &Self, n: u32| -> Option<(bool, bool)> {
            // (raw text mode, character references decoded)
            let name = ELEMENTS.get(n as usize).copied()?;
            let _ = s;
            match name {
                "style" | "script" | "iframe" => Some((true, false)),
                "title" | "textarea" => Some((true, true)),
                _ => None,
            }
        };
        while !self.stopped {
            if !self.content(&data, true) {
                break;
            }
            if self.pos >= self.src.len() {
                break;
            }
            match self.at(self.pos + 1) {
                Some(0x21) => {
                    if !self.markup_declaration() {
                        break;
                    }
                }
                Some(0x3F) => {
                    if !self.charge(16) {
                        break;
                    }
                    self.bogus_comment(self.pos + 2);
                }
                Some(0x2F) if self.at(self.pos + 2).is_none_or(|u| !(u < 128 && (u as u8).is_ascii_alphabetic())) => {
                    if self.at(self.pos + 2) == Some(0x3E) {
                        self.pos += 3;
                    } else {
                        self.bogus_comment(self.pos + 2);
                    }
                }
                _ => {
                    let Some((name, start_tag)) = self.tag() else {
                        break;
                    };
                    if start_tag && let Some((raw, refs)) = lt_name(self, name) && raw {
                        if !self.raw(name, refs) {
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn scan<U: Unit>(src: &[U], reserve: usize) -> (Sink, Vec<u16>, u64) {
    let cancel = AtomicBool::new(false);
    let mut s = Scanner::new(src, &cancel, reserve);
    s.run();
    (s.sink, s.arena, s.credits)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn checksum(units: &[u16]) -> u64 {
    units.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &u| {
        (h ^ u64::from(u)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn main() {
    let filter = std::env::args().nth(1);
    let docs: [(&str, String); 5] = [
        ("ordinary", documents::ordinary()),
        ("deeply-nested", documents::deeply_nested()),
        ("many-attributes", documents::many_attributes()),
        ("table-heavy", documents::table_heavy()),
        ("many-diagnostics", documents::many_diagnostics()),
    ];
    // Mixed non-ASCII text, to compare the UTF-8 and UTF-16 paths.
    let mixed = "<p title=\"é&amp;😀\">a\r\nb\rc\u{0}é😀&copy;&#x80;&#0;&bogus;&#x110000;</p><!-- c\u{e9} --><?x y>\u{1F600}"
        .repeat(50);
    {
        let utf16: Vec<u16> = mixed.encode_utf16().collect();
        let (s8, a8, _) = scan(mixed.as_bytes(), 0);
        let (s16, a16, _) = scan(&utf16, 0);
        assert_eq!(a8, a16, "UTF-8 and UTF-16 paths differ");
        assert_eq!(s8.sum, s16.sum);
        let (_, one, _) = scan(mixed[..mixed.len() / 50].as_bytes(), 0);
        println!("mixed-content arena (one repetition): {:?}", String::from_utf16_lossy(&one));
        println!(
            "mixed-content check: UTF-8 and UTF-16 arenas equal ({} units, checksum {:016x})",
            a8.len(),
            checksum(&a8)
        );
    }
    println!(
        "{:<17} {:>8} {:>8} {:>8} {:>8} {:>9} {:>8} {:>9} {:>8} {:>8}",
        "document", "bytes", "starts", "ends", "texts", "text units", "attrs", "utf8 ms", "ns/byte", "utf16 ms"
    );
    for (name, text) in &docs {
        if filter.as_ref().is_some_and(|f| !name.contains(f.as_str())) {
            continue;
        }
        let utf16: Vec<u16> = text.encode_utf16().collect();
        let (s8, a8, credits) = scan(text.as_bytes(), 0);
        let (s16, a16, _) = scan(&utf16, 0);
        assert_eq!(a8, a16, "{name}: UTF-8 and UTF-16 arenas differ");
        let mut t8 = Vec::new();
        let mut t16 = Vec::new();
        let mut t8r = Vec::new();
        for _ in 0..31 {
            let t = Instant::now();
            let r = scan(black_box(text.as_bytes()), 0);
            t8.push(t.elapsed().as_secs_f64() * 1e3);
            black_box(r);
            let t = Instant::now();
            let r = scan(black_box(text.as_bytes()), text.len());
            t8r.push(t.elapsed().as_secs_f64() * 1e3);
            black_box(r);
            let t = Instant::now();
            let r = scan(black_box(&utf16[..]), 0);
            t16.push(t.elapsed().as_secs_f64() * 1e3);
            black_box(r);
        }
        let m8 = median(t8);
        println!(
            "{:<17} {:>8} {:>8} {:>8} {:>8} {:>9} {:>8} {:>9.3} {:>8.2} {:>8.3}",
            name,
            text.len(),
            s8.starts,
            s8.ends,
            s8.texts,
            s8.text_units,
            s8.attrs,
            m8,
            m8 * 1e6 / text.len() as f64,
            median(t16)
        );
        println!(
            "  arena {} units (checksum {:016x}), attr value units {}, duplicate attributes dropped {}, comments {}, doctypes {}, credits {}, utf8 with arena reserved {:.3} ms",
            a8.len(),
            checksum(&a8),
            s8.attr_units,
            s8.dup_dropped,
            s8.comments,
            s8.doctypes,
            credits,
            median(t8r)
        );
        assert_eq!(s8.starts, s16.starts);
    }
}

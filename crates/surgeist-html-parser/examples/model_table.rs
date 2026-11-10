//! Prototype for #2228 (scratch, not for merge): the dense node table, the text
//! arena and the publication transform of the design note.
//!
//! The tree builder's algorithms are not part of the prototype. A test-only
//! scanner turns each benchmark document into a flat operation stream (start
//! tag, text piece, end tag) outside the timed region. The timed region applies
//! the stream to the builder: interning names, writing text and attribute
//! values into the arena, appending rows, linking children, the per-action
//! limit checks, and the stack of open elements. It then publishes (child array
//! and 20-byte rows) and drops.
//!
//! Run: `cargo run --release --offline --locked -j 1 -p surgeist-html-parser --example model_table`

use std::{hint::black_box, mem::size_of, sync::Arc, time::Instant};

#[path = "../benches/support/documents.rs"]
mod documents;

const NONE: u32 = 0;
const ROOT: u32 = 1;

const K_DOCUMENT: u8 = 0;
const K_ELEMENT: u8 = 2;
const K_TEXT: u8 = 3;
const K_DOCTYPE: u8 = 6;

/// Build row: five links, kind, flags, family mask, two payload words.
#[repr(C)]
#[derive(Clone, Copy)]
struct Row {
    parent: u32,
    first: u32,
    last: u32,
    next: u32,
    prev: u32,
    kind: u8,
    flags: u8,
    family: u16,
    a: u32,
    b: u32,
}
const _: () = assert!(size_of::<Row>() == 32);

/// Published row: parent, child offset, kind, flags, family mask, two payload words.
#[repr(C)]
#[derive(Clone, Copy)]
struct PubRow {
    parent: u32,
    child_off: u32,
    kind: u8,
    flags: u8,
    family: u16,
    a: u32,
    b: u32,
}
const _: () = assert!(size_of::<PubRow>() == 20);

#[derive(Clone, Copy)]
struct AttrRow {
    name: u32,
    start: u32,
    len: u32,
}

/// Interner whose strings live in the text arena.
struct Names {
    slots: Vec<u32>,
    entries: Vec<(u32, u32, u32)>, // start, len, hash
}
impl Names {
    fn new() -> Self {
        Self {
            slots: vec![0; 256],
            entries: Vec::new(),
        }
    }
    #[inline]
    fn hash(units: &[u16]) -> u32 {
        let mut h = 0x811c_9dc5_u32;
        for &u in units {
            h = (h ^ u32::from(u)).wrapping_mul(0x0100_0193);
        }
        h
    }
    #[inline]
    fn intern(&mut self, arena: &mut Vec<u16>, units: &[u16]) -> u32 {
        let h = Self::hash(units);
        let mask = self.slots.len() - 1;
        let mut i = h as usize & mask;
        loop {
            let s = self.slots[i];
            if s == 0 {
                break;
            }
            let (start, len, eh) = self.entries[s as usize - 1];
            if eh == h && arena[start as usize..(start + len) as usize] == *units {
                return s - 1;
            }
            i = (i + 1) & mask;
        }
        let start = arena.len() as u32;
        arena.extend_from_slice(units);
        self.entries.push((start, units.len() as u32, h));
        let id = self.entries.len() as u32;
        self.slots[i] = id;
        if self.entries.len() * 2 > self.slots.len() {
            let n = self.slots.len() * 2;
            self.slots = vec![0; n];
            let mask = n - 1;
            for (k, &(_, _, eh)) in self.entries.iter().enumerate() {
                let mut j = eh as usize & mask;
                while self.slots[j] != 0 {
                    j = (j + 1) & mask;
                }
                self.slots[j] = k as u32 + 1;
            }
        }
        id - 1
    }
}

/// Counters of the per-action admission (nodes, edges, attributes, units).
struct Admit {
    live: [u64; 4],
    cap: [u64; 4],
    credits: u64,
    credit_cap: u64,
}
impl Admit {
    #[inline]
    fn action(&mut self, delta: [u64; 4], credits: u64) -> Result<(), usize> {
        for i in 0..4 {
            if self.live[i] + delta[i] > self.cap[i] {
                return Err(i);
            }
        }
        if self.credits + credits > self.credit_cap {
            return Err(4);
        }
        for i in 0..4 {
            self.live[i] += delta[i];
        }
        self.credits += credits;
        Ok(())
    }
}

struct Builder {
    rows: Vec<Row>,
    arena: Vec<u16>,
    attrs: Vec<AttrRow>,
    runs: Vec<(u32, u32)>,
    names: Names,
    stack: Vec<u32>,
    admit: Admit,
    max_depth: usize,
}

impl Builder {
    fn new(reserve: Option<(usize, usize)>) -> Self {
        let (rows, units) = reserve.unwrap_or((0, 0));
        let mut b = Self {
            rows: Vec::with_capacity(rows),
            arena: Vec::with_capacity(units),
            attrs: Vec::new(),
            runs: vec![(0, 0)],
            names: Names::new(),
            stack: Vec::new(),
            admit: Admit {
                live: [0; 4],
                cap: [1 << 30; 4],
                credits: 0,
                credit_cap: u64::MAX,
            },
            max_depth: 0,
        };
        b.push_row(K_DOCUMENT, NONE, 0, 0);
        b.stack.push(ROOT);
        b
    }
    #[inline]
    fn push_row(&mut self, kind: u8, parent: u32, a: u32, b: u32) -> u32 {
        self.rows.push(Row {
            parent,
            first: NONE,
            last: NONE,
            next: NONE,
            prev: NONE,
            kind,
            flags: 0,
            family: 0,
            a,
            b,
        });
        self.rows.len() as u32
    }
    #[inline]
    fn link_last(&mut self, parent: u32, node: u32) {
        let last = self.rows[parent as usize - 1].last;
        self.rows[node as usize - 1].prev = last;
        if last == NONE {
            self.rows[parent as usize - 1].first = node;
        } else {
            self.rows[last as usize - 1].next = node;
        }
        self.rows[parent as usize - 1].last = node;
    }
    fn element(&mut self, name: &[u16], attrs: &[(&[u16], &[u16])]) -> Result<u32, usize> {
        let parent = *self.stack.last().unwrap_or(&ROOT);
        let mut units = name.len() as u64 + 28; // local name plus the HTML namespace URI
        for (n, v) in attrs {
            units += (n.len() + v.len()) as u64;
        }
        self.admit
            .action([1, 1, attrs.len() as u64, units], 64 + 8 * attrs.len() as u64)?;
        let name_id = self.names.intern(&mut self.arena, name);
        let run = if attrs.is_empty() {
            0
        } else {
            let start = self.attrs.len() as u32;
            for (n, v) in attrs {
                let nid = self.names.intern(&mut self.arena, n);
                let vs = self.arena.len() as u32;
                self.arena.extend_from_slice(v);
                self.attrs.push(AttrRow {
                    name: nid,
                    start: vs,
                    len: v.len() as u32,
                });
            }
            self.runs.push((start, attrs.len() as u32));
            self.runs.len() as u32 - 1
        };
        let node = self.push_row(K_ELEMENT, parent, name_id, run);
        self.link_last(parent, node);
        self.stack.push(node);
        self.max_depth = self.max_depth.max(self.stack.len());
        Ok(node)
    }
    fn pop_to(&mut self, name: &[u16]) {
        // Pop to the nearest open element with this name; ignore when absent.
        let name_id = self.names.intern(&mut self.arena, name);
        let mut i = self.stack.len();
        while i > 1 {
            i -= 1;
            if self.rows[self.stack[i] as usize - 1].a == name_id {
                self.stack.truncate(i);
                return;
            }
        }
    }
    fn text(&mut self, units: &[u16]) -> Result<(), usize> {
        let parent = *self.stack.last().unwrap_or(&ROOT);
        let tail = self.arena.len() as u32;
        let n = units.len() as u32;
        let last = self.rows[parent as usize - 1].last;
        if last != NONE {
            let r = &self.rows[last as usize - 1];
            // The tail check precedes the write, so a piece that continues the
            // previous text node extends it in place.
            if r.kind == K_TEXT && r.a + r.b == tail {
                self.admit.action([0, 0, 0, u64::from(n)], 4 + u64::from(n))?;
                self.arena.extend_from_slice(units);
                self.rows[last as usize - 1].b += n;
                return Ok(());
            }
        }
        self.admit
            .action([1, 1, 0, u64::from(n)], 64 + u64::from(n))?;
        self.arena.extend_from_slice(units);
        let node = self.push_row(K_TEXT, parent, tail, n);
        self.link_last(parent, node);
        Ok(())
    }
    fn doctype(&mut self) -> Result<(), usize> {
        self.admit.action([1, 1, 0, 4], 68)?;
        let node = self.push_row(K_DOCTYPE, ROOT, 0, 0);
        self.link_last(ROOT, node);
        Ok(())
    }
}

struct Tree {
    rows: Vec<PubRow>,
    children: Vec<u32>,
    attrs: Vec<AttrRow>,
    runs: Vec<(u32, u32)>,
    names: Vec<(u32, u32, u32)>,
    block: Arc<Vec<u16>>,
}

fn publish(b: Builder) -> Tree {
    let n = b.rows.len();
    let mut off = vec![0_u32; n + 1];
    for r in &b.rows[1..] {
        off[r.parent as usize] += 1; // degree of parent (1-based slot)
    }
    // Exclusive prefix sum into child_off by node index.
    let mut running = 0_u32;
    let mut start = vec![0_u32; n + 1];
    for i in 0..n {
        start[i] = running;
        running += off[i + 1];
    }
    start[n] = running;
    let mut children = vec![0_u32; running as usize];
    let mut rows = Vec::with_capacity(n);
    for (i, r) in b.rows.iter().enumerate() {
        let mut o = start[i] as usize;
        let mut c = r.first;
        while c != NONE {
            children[o] = c;
            o += 1;
            c = b.rows[c as usize - 1].next;
        }
        rows.push(PubRow {
            parent: r.parent,
            child_off: start[i],
            kind: r.kind,
            flags: r.flags,
            family: r.family,
            a: r.a,
            b: r.b,
        });
    }
    Tree {
        rows,
        children,
        attrs: b.attrs,
        runs: b.runs,
        names: b.names.entries,
        block: Arc::new(b.arena),
    }
}

impl Tree {
    fn retained_len(&self) -> usize {
        self.rows.len() * size_of::<PubRow>()
            + self.children.len() * 4
            + self.attrs.len() * size_of::<AttrRow>()
            + self.runs.len() * 8
            + self.names.len() * 12
            + self.block.len() * 2
    }
    fn retained_capacity(&self) -> usize {
        self.rows.capacity() * size_of::<PubRow>()
            + self.children.capacity() * 4
            + self.attrs.capacity() * size_of::<AttrRow>()
            + self.runs.capacity() * 8
            + self.names.capacity() * 12
            + self.block.capacity() * 2
    }
}

// ---- operation stream (outside the timed region) --------------------------

enum Op {
    Doctype,
    Start {
        name: (u32, u32),
        attrs: (u32, u32),
    },
    Text(u32, u32),
    End {
        name: (u32, u32),
    },
}

struct Ops {
    ops: Vec<Op>,
    pool: Vec<u16>,
    attrs: Vec<((u32, u32), (u32, u32))>,
}

fn build_ops(doc: &str) -> Ops {
    let bytes = doc.as_bytes();
    let mut o = Ops {
        ops: Vec::new(),
        pool: Vec::new(),
        attrs: Vec::new(),
    };
    let mut i = 0;
    let push = |pool: &mut Vec<u16>, s: &[u8]| -> (u32, u32) {
        let st = pool.len() as u32;
        pool.extend(s.iter().map(|&b| u16::from(b)));
        (st, s.len() as u32)
    };
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if bytes[i + 1] == b'!' {
                o.ops.push(Op::Doctype);
                while bytes[i] != b'>' {
                    i += 1;
                }
                i += 1;
                continue;
            }
            let end_tag = bytes[i + 1] == b'/';
            let mut j = i + if end_tag { 2 } else { 1 };
            let ns = j;
            while !matches!(bytes[j], b' ' | b'>' | b'/') {
                j += 1;
            }
            let name = push(&mut o.pool, &bytes[ns..j]);
            if end_tag {
                while bytes[j] != b'>' {
                    j += 1;
                }
                o.ops.push(Op::End { name });
                i = j + 1;
                continue;
            }
            let lo = o.attrs.len() as u32;
            loop {
                while bytes[j] == b' ' {
                    j += 1;
                }
                if matches!(bytes[j], b'>' | b'/') {
                    break;
                }
                let an = j;
                while !matches!(bytes[j], b'=' | b' ' | b'>' | b'/') {
                    j += 1;
                }
                let aname = push(&mut o.pool, &bytes[an..j]);
                let mut aval = (o.pool.len() as u32, 0);
                if bytes[j] == b'=' {
                    j += 1;
                    let q = bytes[j];
                    j += 1;
                    let vs = j;
                    while bytes[j] != q {
                        j += 1;
                    }
                    aval = push(&mut o.pool, &bytes[vs..j]);
                    j += 1;
                }
                o.attrs.push((aname, aval));
            }
            let hi = o.attrs.len() as u32;
            while bytes[j] != b'>' {
                j += 1;
            }
            o.ops.push(Op::Start {
                name,
                attrs: (lo, hi),
            });
            let void = matches!(&bytes[ns..ns + name.1 as usize], b"meta" | b"br" | b"img" | b"input" | b"hr" | b"link");
            if void {
                o.ops.push(Op::End { name });
            }
            i = j + 1;
        } else {
            let ts = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            // Split at character references so that merging is exercised.
            let mut seg = ts;
            let mut k = ts;
            while k < i {
                if bytes[k] == b'&' {
                    let semi = k + bytes[k..i].iter().position(|&c| c == b';').unwrap_or(0);
                    let decoded: Option<u16> = match &bytes[k..=semi] {
                        b"&amp;" => Some(b'&'.into()),
                        b"&lt;" => Some(b'<'.into()),
                        b"&gt;" => Some(b'>'.into()),
                        b"&copy;" => Some(0xA9),
                        _ => None,
                    };
                    if let Some(u) = decoded {
                        if seg < k {
                            let p = push(&mut o.pool, &bytes[seg..k]);
                            o.ops.push(Op::Text(p.0, p.1));
                        }
                        let st = o.pool.len() as u32;
                        o.pool.push(u);
                        o.ops.push(Op::Text(st, 1));
                        k = semi + 1;
                        seg = k;
                        continue;
                    }
                }
                k += 1;
            }
            if seg < i {
                let p = push(&mut o.pool, &bytes[seg..i]);
                o.ops.push(Op::Text(p.0, p.1));
            }
        }
    }
    o
}

fn replay(ops: &Ops, reserve: Option<(usize, usize)>) -> Builder {
    let pool = &ops.pool[..];
    let slice = |r: (u32, u32)| &pool[r.0 as usize..(r.0 + r.1) as usize];
    let mut b = Builder::new(reserve);
    let mut attr_buf: Vec<(&[u16], &[u16])> = Vec::with_capacity(32);
    for op in &ops.ops {
        match op {
            Op::Doctype => {
                b.doctype().unwrap();
            }
            Op::Start { name, attrs } => {
                attr_buf.clear();
                for &(n, v) in &ops.attrs[attrs.0 as usize..attrs.1 as usize] {
                    attr_buf.push((slice(n), slice(v)));
                }
                b.element(slice(*name), &attr_buf).unwrap();
            }
            Op::Text(s, l) => {
                b.text(&pool[*s as usize..(*s + *l) as usize]).unwrap();
            }
            Op::End { name } => b.pop_to(slice(*name)),
        }
    }
    b
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn check_shape(tree: &Tree) -> (usize, usize, usize) {
    // Independent walk of the published tree: counts by kind, depth.
    let mut elements = 0;
    let mut texts = 0;
    let mut depth = 0;
    let mut stack = vec![(0_usize, 1_usize)];
    while let Some((i, d)) = stack.pop() {
        depth = depth.max(d);
        let r = tree.rows[i];
        match r.kind {
            K_ELEMENT => elements += 1,
            K_TEXT => texts += 1,
            _ => {}
        }
        let end = tree.rows.get(i + 1).map_or(tree.children.len() as u32, |n| n.child_off);
        for &c in &tree.children[r.child_off as usize..end as usize] {
            assert_eq!(tree.rows[c as usize - 1].parent as usize, i + 1);
            stack.push((c as usize - 1, d + 1));
        }
    }
    (elements, texts, depth)
}

fn main() {
    let filter = std::env::args().nth(1);
    let docs: [(&str, String); 4] = [
        ("ordinary", documents::ordinary()),
        ("deeply-nested", documents::deeply_nested()),
        ("many-attributes", documents::many_attributes()),
        ("table-heavy", documents::table_heavy()),
    ];
    println!(
        "{:<16} {:>8} {:>7} {:>10} {:>10} {:>10} {:>9} {:>9} {:>10} {:>10} {:>7}",
        "document", "bytes", "nodes", "append ms", "ns/node", "reserved", "publish ms", "drop us", "retained B", "capacity B", "x input"
    );
    for (name, text) in &docs {
        if filter.as_ref().is_some_and(|f| !name.contains(f.as_str())) {
            continue;
        }
        let ops = build_ops(text);
        let probe = replay(&ops, None);
        let nodes = probe.rows.len();
        let est = (text.len() / 4, text.len());
        let mut t_append = Vec::new();
        let mut t_reserved = Vec::new();
        let mut t_publish = Vec::new();
        let mut t_drop = Vec::new();
        let mut retained = (0, 0);
        let mut shape = (0, 0, 0);
        for round in 0..25 {
            let t0 = Instant::now();
            let b = black_box(replay(black_box(&ops), None));
            let t1 = Instant::now();
            let tree = publish(b);
            let t2 = Instant::now();
            if round == 0 {
                retained = (tree.retained_len(), tree.retained_capacity());
                shape = check_shape(&tree);
            }
            drop(black_box(tree));
            let t3 = Instant::now();
            let br = black_box(replay(black_box(&ops), Some(est)));
            let t4 = Instant::now();
            drop(black_box(br));
            t_append.push((t1 - t0).as_secs_f64() * 1e3);
            t_publish.push((t2 - t1).as_secs_f64() * 1e3);
            t_drop.push((t3 - t2).as_secs_f64() * 1e3);
            t_reserved.push((t4 - t3).as_secs_f64() * 1e3);
        }
        let a = median(t_append);
        println!(
            "{:<16} {:>8} {:>7} {:>10.3} {:>10.1} {:>10.3} {:>9.3} {:>9.1} {:>10} {:>10} {:>7.2}",
            name,
            text.len(),
            nodes,
            a,
            a * 1e6 / nodes as f64,
            median(t_reserved),
            median(t_publish),
            median(t_drop) * 1e3,
            retained.0,
            retained.1,
            retained.0 as f64 / text.len() as f64
        );
        println!(
            "  published tree: {} elements, {} text nodes, max depth {}",
            shape.0, shape.1, shape.2
        );
        println!(
            "  arena {} units, {} names, {} attribute rows, build rows {} B",
            probe.arena.len(),
            probe.names.entries.len(),
            probe.attrs.len(),
            nodes * size_of::<Row>()
        );
    }
}

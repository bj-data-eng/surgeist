//! HNA1: strict lossless little-endian native operation recipes.
use super::*;

struct Reader<'a> {
    input: &'a [u8],
    at: usize,
    units: usize,
    attrs: usize,
    nodes: usize,
    allocate: bool,
}
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| invalid("length overflow"))?;
        let s = self
            .input
            .get(self.at..end)
            .ok_or_else(|| invalid("truncated native case"))?;
        self.at = end;
        Ok(s)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn add_units(&mut self, n: usize) -> Result<()> {
        self.units = self
            .units
            .checked_add(n)
            .ok_or_else(|| invalid("unit overflow"))?;
        if self.units > 8192 {
            return Err(invalid("aggregate native units"));
        }
        Ok(())
    }
    fn field(&mut self, max: usize) -> Result<Vec<u16>> {
        let n = usize::from(u16::from_le_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ));
        if n > max {
            return Err(invalid("native field ceiling"));
        }
        self.add_units(n)?;
        let allocate = self.allocate;
        let bytes = self.take(n * 2)?;
        let mut out = Vec::new();
        if allocate {
            reserve(&mut out, n)?;
            for b in bytes.chunks_exact(2) {
                out.push(u16::from_le_bytes([b[0], b[1]]));
            }
        }
        Ok(out)
    }
    fn optional(&mut self, max: usize) -> Result<Option<Vec<u16>>> {
        match self.byte()? {
            0 => Ok(None),
            1 => self.field(max).map(Some),
            _ => Err(invalid("optional field tag")),
        }
    }
    fn name(&mut self) -> Result<Name> {
        match self.byte()? {
            n @ 0..=3 => {
                let name = Name::Known(n);
                self.add_units(name.units())?;
                Ok(name)
            }
            4 => Ok(Name::Raw {
                uri: self.field(128)?,
                prefix: self.optional(128)?,
                local: self.field(128)?,
            }),
            _ => Err(invalid("name tag")),
        }
    }
    fn attr(&mut self) -> Result<Attr> {
        self.attrs += 1;
        if self.attrs > 128 {
            return Err(invalid("attribute ceiling"));
        }
        match self.byte()? {
            0 => Ok(Attr::Raw(self.name()?, self.field(512)?)),
            1 => {
                let n = self.byte()?;
                if n > 1 {
                    return Err(invalid("hidden keyword"));
                }
                let a = Attr::Hidden(n);
                self.add_units(a.units())?;
                Ok(a)
            }
            _ => Err(invalid("attribute tag")),
        }
    }
    fn operation(&mut self, index: usize) -> Result<Op> {
        let tag = self.byte()?;
        let p = self.byte()?;
        if p != 255 && usize::from(p) > index {
            return Err(invalid("forward or invalid handle slot"));
        }
        self.nodes += match tag {
            0 => 2,
            1..=4 | 6 => 1,
            5 => 0,
            _ => return Err(invalid("operation tag")),
        };
        if self.nodes > 64 {
            return Err(invalid("potential native node ceiling"));
        }
        match tag {
            0 => {
                let name = self.name()?;
                let n = usize::from(self.byte()?);
                if n > 8 {
                    return Err(invalid("per-element attributes"));
                }
                let mut attrs = Vec::new();
                if self.allocate {
                    reserve(&mut attrs, n)?;
                }
                for _ in 0..n {
                    let a = self.attr()?;
                    if self.allocate {
                        attrs.push(a);
                    }
                }
                Ok(Op::Element(p, name, attrs))
            }
            1 => Ok(Op::Text(p, self.field(512)?)),
            2 => Ok(Op::Comment(p, self.field(512)?)),
            3 => Ok(Op::Pi(p, self.field(128)?, self.field(512)?)),
            4 => Ok(Op::Doctype(
                p,
                self.field(128)?,
                self.optional(512)?,
                self.optional(512)?,
            )),
            5 => Ok(Op::Contents(p)),
            6 => {
                let bits = self.byte()?;
                if bits > 31 {
                    return Err(invalid("shadow properties"));
                }
                Ok(Op::Shadow(p, bits))
            }
            _ => unreachable!("checked operation"),
        }
    }
}
fn read(t: &NativeTarget, input: &[u8], allocate: bool) -> Result<NativeCase> {
    if input.len() > MAX_BYTES {
        return Err(invalid("canonical native byte ceiling"));
    }
    let mut r = Reader {
        input,
        at: 0,
        units: 0,
        attrs: 0,
        // Main root, foreign-builder root, then the independently frozen
        // inspection root. Credit is cumulative even after a builder drops.
        nodes: 3,
        allocate,
    };
    if r.take(4)? != b"HNA1" {
        return Err(invalid("native format revision"));
    }
    let root = r.byte()?;
    if root > 3 {
        return Err(invalid("native root/mode"));
    }
    let literal = match r.byte()? {
        255 => None,
        n @ 0..=7 => Some(n),
        _ => return Err(invalid("native literal tag")),
    };
    let mut caps = [0; 5];
    for (cap, max) in caps.iter_mut().zip(CAPS) {
        *cap = usize::try_from(u64::from_le_bytes(
            r.take(8)?.try_into().expect("eight bytes"),
        ))
        .map_err(|_| invalid("cap representation"))?;
        if *cap > max {
            return Err(invalid("native cap ceiling"));
        }
    }
    let n = usize::from(r.byte()?);
    if n > 64 {
        return Err(invalid("native command ceiling"));
    }
    let mut ops = Vec::new();
    if allocate {
        reserve(&mut ops, n)?;
    }
    for i in 0..n {
        let o = r.operation(i)?;
        if allocate {
            ops.push(o);
        }
    }
    if r.at != input.len() {
        return Err(invalid("trailing native bytes"));
    }
    if !allocate {
        t.credit(r.nodes, r.units, n)?;
    }
    Ok(NativeCase {
        root,
        literal,
        caps,
        ops,
    })
}
pub(super) fn decode(t: &NativeTarget, input: &[u8]) -> Result<NativeCase> {
    // First pass validates all fields and reserves aggregate credit without
    // allocating a payload. Second pass constructs the identical checked case.
    read(t, input, false)?;
    let c = read(t, input, true)?;
    if let Some(id) = c.literal
        && c != t.literal(id)?
    {
        return Err(invalid("changed native literal"));
    }
    Ok(c)
}
struct Writer {
    v: Vec<u8>,
}
impl Writer {
    fn byte(&mut self, n: u8) {
        self.v.push(n);
    }
    fn field(&mut self, v: &[u16]) -> Result<()> {
        if v.len() > 512 {
            return Err(invalid("native field ceiling"));
        }
        self.v.extend_from_slice(&(v.len() as u16).to_le_bytes());
        for n in v {
            self.v.extend_from_slice(&n.to_le_bytes());
        }
        Ok(())
    }
    fn optional(&mut self, v: &Option<Vec<u16>>) -> Result<()> {
        match v {
            None => self.byte(0),
            Some(v) => {
                self.byte(1);
                self.field(v)?;
            }
        }
        Ok(())
    }
    fn name(&mut self, n: &Name) -> Result<()> {
        match n {
            Name::Known(n) => self.byte(*n),
            Name::Raw { uri, prefix, local } => {
                self.byte(4);
                self.field(uri)?;
                self.optional(prefix)?;
                self.field(local)?;
            }
        }
        Ok(())
    }
    fn attr(&mut self, a: &Attr) -> Result<()> {
        match a {
            Attr::Raw(n, v) => {
                self.byte(0);
                self.name(n)?;
                self.field(v)?;
            }
            Attr::Hidden(n) => {
                self.byte(1);
                self.byte(*n);
            }
        }
        Ok(())
    }
}
pub(super) fn encode(t: &NativeTarget, c: &NativeCase) -> Result<Vec<u8>> {
    t.credit(64, 8192, 64)?;
    let mut w = Writer { v: Vec::new() };
    reserve(&mut w.v, ENCODED_CAPACITY)?;
    w.v.extend_from_slice(b"HNA1");
    w.byte(c.root);
    w.byte(c.literal.unwrap_or(255));
    for n in c.caps {
        w.v.extend_from_slice(&(n as u64).to_le_bytes());
    }
    w.byte(u8::try_from(c.ops.len()).map_err(|_| invalid("operation count"))?);
    for o in &c.ops {
        let tag = match o {
            Op::Element(..) => 0,
            Op::Text(..) => 1,
            Op::Comment(..) => 2,
            Op::Pi(..) => 3,
            Op::Doctype(..) => 4,
            Op::Contents(..) => 5,
            Op::Shadow(..) => 6,
        };
        w.byte(tag);
        w.byte(o.parent());
        match o {
            Op::Element(_, n, a) => {
                w.name(n)?;
                w.byte(u8::try_from(a.len()).map_err(|_| invalid("attribute count"))?);
                for a in a {
                    w.attr(a)?;
                }
            }
            Op::Text(_, v) | Op::Comment(_, v) => w.field(v)?,
            Op::Pi(_, a, b) => {
                w.field(a)?;
                w.field(b)?;
            }
            Op::Doctype(_, n, p, s) => {
                w.field(n)?;
                w.optional(p)?;
                w.optional(s)?;
            }
            Op::Contents(_) => {}
            Op::Shadow(_, b) => w.byte(*b),
        }
    }
    read(t, &w.v, false)?;
    if let Some(id) = c.literal
        && *c != t.literal(id)?
    {
        return Err(invalid("changed native literal"));
    }
    Ok(w.v)
}
pub(super) fn validate(t: &NativeTarget, c: &NativeCase) -> Result<()> {
    encode(t, c).map(|_| ())
}

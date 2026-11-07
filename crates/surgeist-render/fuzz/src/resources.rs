//! Bounded resource cases. Font admission never implies outline conformance.
use surgeist_fuzz::generation::{Budget, BudgetCharge};
use surgeist_fuzz::{CaseOutcome, CaseTarget, Generation, Result, TargetDescriptor};

use crate::{
    codec::{Reader, Writer},
    descriptor, error,
};

mod evaluation;
#[cfg(test)]
mod tests;

const MAGIC: [u8; 4] = *b"SRFR";
const MAX_FONT: usize = 32 * 1024;
const MAX_RANDOM_FONT: usize = 2 * 1024;
const MAX_GLYPHS: usize = 16;
const MAX_COORDINATES: usize = 8;
const MAX_RGBA: usize = 4 * 1024;
const MAX_PRODUCER_BYTES: u32 = 1 << 20;
// Exact licensed fixture identity, not a font name or a generated substitute.
// Provenance: ../../tests/fixtures/fonts/ahem/PROVENANCE.md, SHA-256
// f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc.
const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
const FONT_CAPS: [u32; 7] = [32768, 32768, 32768, 256, 16, 1 << 20, 1 << 20];
const GLYPH_CAPS: [u32; 4] = [16, 1024, 1 << 20, 1 << 20];
const FONT: &str = "surgeist-render::font-admission";
const GLYPH: &str = "surgeist-render::glyph-preparation";
const IMAGE: &str = "surgeist-render::image-admission";
const RETRY: &str = "surgeist-render::resource-refusal-retry";

/// Concrete, checked resource transport. Numeric inputs retain their IEEE bits;
/// semantic invalidity is exercised at the public Render boundary.
#[derive(Clone, Debug)]
pub struct ResourceCase {
    operation: Operation,
}

#[derive(Clone, Debug)]
enum Operation {
    Font(FontCase),
    Glyph(GlyphCase),
    Image {
        width: f64,
        height: f64,
        rgba: Vec<u8>,
    },
    Retry {
        glyph: GlyphCase,
        refusal: Refusal,
        refused_limits: [u32; 4],
        refused_path_work: u64,
        raised_path_work: u64,
    },
}

#[derive(Clone, Debug)]
struct FontCase {
    bytes: Vec<u8>,
    index: u32,
    limits: [u32; 7],
}

#[derive(Clone, Debug)]
struct GlyphCase {
    font: FontCase,
    size: f32,
    skew: f32,
    embolden: f32,
    coordinates: Vec<i16>,
    glyphs: Vec<GlyphInput>,
    limits: [u32; 4],
}

#[derive(Clone, Copy, Debug)]
struct GlyphInput {
    id: u32,
    x: f64,
    y: f64,
    advance: f32,
}

#[derive(Clone, Copy, Debug)]
enum Refusal {
    Glyphs,
    Path,
    Scratch,
    Retained,
    Accounting,
}

/// Public headless resource-admission target; no device or host is created.
pub struct ResourceTarget {
    descriptor: TargetDescriptor,
}

impl Default for ResourceTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceTarget {
    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptor: descriptor("surgeist-render::resource-admission"),
        }
    }

    fn budget(&self, bytes: usize, nodes: usize, operations: u64) -> Result<Budget> {
        let mut budget = Budget::new(self.descriptor.limits());
        budget.credit(BudgetCharge {
            bytes: bytes as u64,
            nodes: nodes as u64,
            depth: 1,
            operations,
            ..BudgetCharge::default()
        })?;
        Ok(budget)
    }

    /// Materializes a source-backed literal for canonical seed creation.
    /// Unknown literal indices reject rather than selecting another fixture.
    pub fn literal(&self, which: u8) -> Result<ResourceCase> {
        // Conservative harness allocation credit precedes all fixture copying.
        self.budget(MAX_FONT + 4096, 32, 64)?;
        let font = || FontCase {
            bytes: AHEM.to_vec(),
            index: 0,
            limits: FONT_CAPS,
        };
        let glyph = || GlyphCase {
            font: font(),
            size: 1000.0,
            skew: 0.0,
            embolden: 0.0,
            coordinates: vec![],
            glyphs: vec![GlyphInput {
                id: 58,
                x: -20.0,
                y: 30.0,
                advance: -123.0,
            }],
            limits: GLYPH_CAPS,
        };
        let operation = match which {
            0 => Operation::Font(font()),
            1 => Operation::Font(FontCase {
                bytes: vec![],
                index: 0,
                limits: FONT_CAPS,
            }),
            2 => Operation::Font(FontCase { index: 1, ..font() }),
            3 => {
                let mut f = font();
                f.limits[0] = 0;
                Operation::Font(f)
            }
            4 => Operation::Glyph(glyph()),
            5 => {
                let mut g = glyph();
                g.glyphs[0].id = 3;
                Operation::Glyph(g)
            }
            6 => {
                let mut g = glyph();
                g.skew = 0.5;
                Operation::Glyph(g)
            }
            7 => {
                let mut g = glyph();
                g.glyphs[0].x = f64::from_bits(0x7ff8_0000_0000_1234);
                Operation::Glyph(g)
            }
            8 => {
                let mut g = glyph();
                g.size = -0.0;
                Operation::Glyph(g)
            }
            9 => {
                let mut g = glyph();
                g.coordinates.push(0);
                Operation::Glyph(g)
            }
            10 => {
                let mut g = glyph();
                g.glyphs[0].id = u32::MAX;
                Operation::Glyph(g)
            }
            11 => Operation::Image {
                width: 2.0,
                height: 2.0,
                rgba: vec![0, 1, 2, 255, 4, 5, 6, 255, 8, 9, 10, 255, 12, 13, 14, 255],
            },
            12 => Operation::Image {
                width: 0.5,
                height: 1.0,
                rgba: vec![0; 4],
            },
            13 => Operation::Image {
                width: 1.0,
                height: 1.0,
                rgba: vec![0; 3],
            },
            14..=18 => Operation::Retry {
                glyph: glyph(),
                refusal: match which {
                    14 => Refusal::Glyphs,
                    15 => Refusal::Path,
                    16 => Refusal::Scratch,
                    17 => Refusal::Retained,
                    _ => Refusal::Accounting,
                },
                refused_limits: match which {
                    14 => [0, 1024, 1 << 20, 1 << 20],
                    15 => [16, 4, 1 << 20, 1 << 20],
                    16 => [16, 1024, 0, 1 << 20],
                    17 => [16, 1024, 1 << 20, 0],
                    _ => GLYPH_CAPS,
                },
                refused_path_work: if which == 18 { 4 } else { u64::MAX },
                raised_path_work: u64::MAX,
            },
            19 => {
                let mut g = glyph();
                g.coordinates.push(16385);
                Operation::Glyph(g)
            }
            20 => {
                let mut g = glyph();
                g.embolden = -1.0;
                Operation::Glyph(g)
            }
            21 => {
                let mut g = glyph();
                g.glyphs.clear();
                Operation::Glyph(g)
            }
            22 => Operation::Image {
                width: f64::INFINITY,
                height: 1.0,
                rgba: vec![],
            },
            23 => {
                let mut g = glyph();
                g.limits[1] = 4;
                Operation::Glyph(g)
            }
            _ => return Err(error("unknown resource literal")),
        };
        let case = ResourceCase { operation };
        case.validate()?;
        Ok(case)
    }
}

impl ResourceCase {
    fn validate(&self) -> Result<()> {
        match &self.operation {
            Operation::Font(f) => f.validate(),
            Operation::Glyph(g) => g.validate(),
            Operation::Image {
                width,
                height,
                rgba,
            } => {
                if rgba.len() > MAX_RGBA {
                    return Err(error("RGBA harness ceiling"));
                }
                // Integral dimensions over 16 are outside this harness. Invalid
                // numeric inputs remain concrete public geometry/image probes.
                if [*width, *height]
                    .iter()
                    .any(|v| v.is_finite() && *v >= 0.0 && v.fract() == 0.0 && *v > 16.0)
                {
                    return Err(error("image dimension harness ceiling"));
                }
                Ok(())
            }
            Operation::Retry {
                glyph,
                refusal,
                refused_limits,
                refused_path_work,
                raised_path_work,
            } => {
                glyph.validate()?;
                if glyph.font.index != 0
                    || glyph.font.limits != FONT_CAPS
                    || glyph.limits != GLYPH_CAPS
                    || glyph.size.to_bits() != 1000.0f32.to_bits()
                    || glyph.skew != 0.0
                    || glyph.embolden != 0.0
                    || !glyph.coordinates.is_empty()
                    || glyph.glyphs.is_empty()
                    || glyph.glyphs.iter().any(|g| {
                        g.id != 58
                            || !g.x.is_finite()
                            || !g.y.is_finite()
                            || !g.advance.is_finite()
                            || g.x.abs() > 4096.0
                            || g.y.abs() > 4096.0
                    })
                {
                    return Err(error(
                        "retry requires bounded successful Ahem inputs and explicit raised limits",
                    ));
                }
                let dimension = match refusal {
                    Refusal::Glyphs => Some(0),
                    Refusal::Path => Some(1),
                    Refusal::Scratch => Some(2),
                    Refusal::Retained => Some(3),
                    Refusal::Accounting => None,
                };
                if refused_limits.iter().zip(GLYPH_CAPS).any(|(v, m)| *v > m)
                    || *raised_path_work < (glyph.glyphs.len() * 5) as u64
                    || (0..4).any(|i| Some(i) != dimension && refused_limits[i] != glyph.limits[i])
                    || match refusal {
                        Refusal::Glyphs => refused_limits[0] as usize >= glyph.glyphs.len(),
                        Refusal::Path => refused_limits[1] as usize >= glyph.glyphs.len() * 5,
                        Refusal::Scratch => refused_limits[2] != 0,
                        Refusal::Retained => refused_limits[3] != 0,
                        Refusal::Accounting => {
                            *refused_path_work >= (glyph.glyphs.len() * 5) as u64
                        }
                    }
                    || (dimension.is_some() && *refused_path_work != *raised_path_work)
                {
                    return Err(error(
                        "retry limit settings do not select the declared refusal and a successful raised request",
                    ));
                }
                Ok(())
            }
        }
    }
}

impl FontCase {
    fn validate(&self) -> Result<()> {
        let maxima = [
            32768,
            32768,
            32768,
            256,
            16,
            MAX_PRODUCER_BYTES,
            MAX_PRODUCER_BYTES,
        ];
        if self.bytes.len() > MAX_FONT || self.limits.iter().zip(maxima).any(|(v, m)| *v > m) {
            return Err(error("font harness or producer ceiling"));
        }
        Ok(())
    }
}

impl GlyphCase {
    fn validate(&self) -> Result<()> {
        self.font.validate()?;
        if self.font.bytes != AHEM {
            return Err(error("glyph oracle requires exact licensed Ahem bytes"));
        }
        if self.coordinates.len() > MAX_COORDINATES
            || self.glyphs.len() > MAX_GLYPHS
            || self.limits.iter().zip(GLYPH_CAPS).any(|(v, m)| *v > m)
            || self
                .glyphs
                .iter()
                .any(|g| !matches!(g.id, 3 | 58) && g.id < 281)
        {
            return Err(error("glyph harness ceiling or unsupported fixture glyph"));
        }
        Ok(())
    }
}

impl CaseTarget for ResourceTarget {
    type Case = ResourceCase;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<ResourceCase> {
        self.budget(input.len().saturating_add(4096), 32, 64)?;
        let mut r = Reader::new(input, MAGIC)?;
        let operation = match r.u8()? {
            0 => Operation::Font(read_font(&mut r)?),
            1 => Operation::Glyph(read_glyph(&mut r)?),
            2 => Operation::Image {
                width: r.f64()?,
                height: r.f64()?,
                rgba: r.bytes(MAX_RGBA)?,
            },
            3 => {
                let refusal = match r.u8()? {
                    0 => Refusal::Glyphs,
                    1 => Refusal::Path,
                    2 => Refusal::Scratch,
                    3 => Refusal::Retained,
                    4 => Refusal::Accounting,
                    _ => return Err(error("unknown resource refusal")),
                };
                let mut refused_limits = [0; 4];
                for value in &mut refused_limits {
                    *value = r.u32()?;
                }
                let refused_path_work = r.u64()?;
                let raised_path_work = r.u64()?;
                Operation::Retry {
                    glyph: read_glyph(&mut r)?,
                    refusal,
                    refused_limits,
                    refused_path_work,
                    raised_path_work,
                }
            }
            _ => return Err(error("unknown resource operation")),
        };
        r.finish()?;
        let case = ResourceCase { operation };
        case.validate()?;
        Ok(case)
    }
    fn encode(&self, case: &ResourceCase) -> Result<Vec<u8>> {
        case.validate()?;
        self.budget(MAX_FONT + 4096, 32, 64)?;
        let mut w = Writer::new(MAGIC)?;
        match &case.operation {
            Operation::Font(f) => {
                w.u8(0)?;
                write_font(&mut w, f)?;
            }
            Operation::Glyph(g) => {
                w.u8(1)?;
                write_glyph(&mut w, g)?;
            }
            Operation::Image {
                width,
                height,
                rgba,
            } => {
                w.u8(2)?;
                w.f64(*width)?;
                w.f64(*height)?;
                w.bytes(rgba, MAX_RGBA)?;
            }
            Operation::Retry {
                glyph,
                refusal,
                refused_limits,
                refused_path_work,
                raised_path_work,
            } => {
                w.u8(3)?;
                w.u8(match refusal {
                    Refusal::Glyphs => 0,
                    Refusal::Path => 1,
                    Refusal::Scratch => 2,
                    Refusal::Retained => 3,
                    Refusal::Accounting => 4,
                })?;
                for value in refused_limits {
                    w.u32(*value)?;
                }
                w.u64(*refused_path_work)?;
                w.u64(*raised_path_work)?;
                write_glyph(&mut w, glyph)?;
            }
        }
        Ok(w.finish())
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<ResourceCase>> {
        let lane = input.first().copied().unwrap_or(0);
        let case = if lane % 3 == 0 {
            let bytes = input.get(1..).unwrap_or_default();
            let count = bytes.len().min(MAX_RANDOM_FONT);
            self.budget(count + 256, 1, 4)?;
            ResourceCase {
                operation: Operation::Font(FontCase {
                    bytes: bytes[..count].to_vec(),
                    index: u32::from(lane / 3),
                    limits: FONT_CAPS,
                }),
            }
        } else if lane % 3 == 1 {
            self.literal(input.get(1).copied().unwrap_or(lane) % 24)?
        } else {
            let width = usize::from(input.get(1).copied().unwrap_or(1) % 17);
            let height = usize::from(input.get(2).copied().unwrap_or(1) % 17);
            let count = width * height * 4;
            self.budget(count + 128, 1, 4)?;
            let mut rgba = Vec::new();
            rgba.try_reserve_exact(count)
                .map_err(|_| error("generated RGBA allocation"))?;
            for i in 0..count {
                rgba.push(input.get(3 + i).copied().unwrap_or((i % 256) as u8));
            }
            ResourceCase {
                operation: Operation::Image {
                    width: width as f64,
                    height: height as f64,
                    rgba,
                },
            }
        };
        case.validate()?;
        Ok(Generation::Generated(case))
    }
    fn evaluate(&self, case: &ResourceCase) -> Result<CaseOutcome> {
        case.validate()?;
        evaluation::evaluate(case)
    }
}

fn read_font(r: &mut Reader<'_>) -> Result<FontCase> {
    let index = r.u32()?;
    let mut limits = [0; 7];
    for value in &mut limits {
        *value = r.u32()?;
    }
    let f = FontCase {
        bytes: r.bytes(MAX_FONT)?,
        index,
        limits,
    };
    f.validate()?;
    Ok(f)
}
fn write_font(w: &mut Writer, f: &FontCase) -> Result<()> {
    w.u32(f.index)?;
    for value in f.limits {
        w.u32(value)?;
    }
    w.bytes(&f.bytes, MAX_FONT)
}
fn read_glyph(r: &mut Reader<'_>) -> Result<GlyphCase> {
    let font = read_font(r)?;
    let size = r.f32()?;
    let skew = r.f32()?;
    let embolden = r.f32()?;
    let count = r.count(MAX_COORDINATES)?;
    let mut coordinates = Vec::new();
    coordinates
        .try_reserve_exact(count)
        .map_err(|_| error("coordinate allocation"))?;
    for _ in 0..count {
        coordinates.push(i16::from_le_bytes(r.u16()?.to_le_bytes()));
    }
    let count = r.count(MAX_GLYPHS)?;
    let mut glyphs = Vec::new();
    glyphs
        .try_reserve_exact(count)
        .map_err(|_| error("glyph allocation"))?;
    for _ in 0..count {
        glyphs.push(GlyphInput {
            id: r.u32()?,
            x: r.f64()?,
            y: r.f64()?,
            advance: r.f32()?,
        });
    }
    let mut limits = [0; 4];
    for value in &mut limits {
        *value = r.u32()?;
    }
    Ok(GlyphCase {
        font,
        size,
        skew,
        embolden,
        coordinates,
        glyphs,
        limits,
    })
}
fn write_glyph(w: &mut Writer, g: &GlyphCase) -> Result<()> {
    write_font(w, &g.font)?;
    w.f32(g.size)?;
    w.f32(g.skew)?;
    w.f32(g.embolden)?;
    w.count(g.coordinates.len(), MAX_COORDINATES)?;
    for value in &g.coordinates {
        w.u16(u16::from_le_bytes(value.to_le_bytes()))?;
    }
    w.count(g.glyphs.len(), MAX_GLYPHS)?;
    for glyph in &g.glyphs {
        w.u32(glyph.id)?;
        w.f64(glyph.x)?;
        w.f64(glyph.y)?;
        w.f32(glyph.advance)?;
    }
    for value in g.limits {
        w.u32(value)?;
    }
    Ok(())
}

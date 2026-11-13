//! Headless representation admission for one authentic finalized pattern.

use super::color;
use crate::{
    Error, FontData, GlyphSynthesis, NormalizedCoordinate, PreparedGlyphRun, Rect,
    RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork, Result, TextGlyph,
    TextInstance, TextPaint, TextRunBounds, Transform,
    accounting::{Account, AccountedVec},
    text::{map_preparation_bounds, qualify_preparation_placement},
};
use std::mem::size_of;
use surgeist_font::ResourceId;
use surgeist_text::{FinalizedLeaderPattern, GlyphFrame, LeaderPaintGroup};

/// Preserved originating cause at an actual finalized leader run.
#[derive(Debug)]
pub enum LeaderPatternCause {
    /// Text could not qualify its retained instance or physical glyph frame.
    Text(Box<surgeist_text::Error>),
    /// Render could not admit the actual representation, paint or numeric facts.
    Render(Box<Error>),
}

/// Atomic leader admission failure. Central accounting errors remain direct.
#[derive(Debug)]
pub enum LeaderPatternError {
    /// A lower-level failure at the original paint-order run ordinal.
    Run {
        /// Actual run ordinal, including any preceding glyphless runs.
        run: usize,
        /// Original typed cause.
        cause: LeaderPatternCause,
    },
}

impl LeaderPatternError {
    fn text(run: usize, error: surgeist_text::Error) -> Error {
        Self::Run {
            run,
            cause: LeaderPatternCause::Text(Box::new(error)),
        }
        .into()
    }
    fn render(run: usize, error: Error) -> Error {
        if error.accounting_diagnostic().is_some() {
            return error;
        }
        Self::Run {
            run,
            cause: LeaderPatternCause::Render(Box::new(error)),
        }
        .into()
    }
}

#[derive(Debug)]
struct LeaderRun {
    ordinal: usize,
    run: PreparedGlyphRun,
    frame: GlyphFrame,
    transform: Transform,
    paint: TextPaint,
    glyphs: Vec<LeaderGlyph>,
}

#[derive(Debug)]
struct LeaderGlyph {
    group: LeaderPaintGroup,
    bounds: TextRunBounds,
}

#[derive(Clone, Copy, Debug)]
struct Member {
    run: usize,
    glyph: usize,
}

#[derive(Debug)]
struct LeaderGroup {
    group: LeaderPaintGroup,
    bounds: TextRunBounds,
    members: Vec<Member>,
}

struct GroupBuilder {
    group: LeaderPaintGroup,
    bounds: TextRunBounds,
    members: AccountedVec<Member>,
}

/// An admission receipt borrowing the exact finalized Text pattern.
///
/// Ink envelopes are conservative representation-derived bounds in Text's
/// physical pattern-local frame, anchored at baseline (0,0). They are neither
/// advance rectangles nor positive-area mask tests. This receipt creates no
/// placement, repetition period, source identity or generated occurrence.
#[derive(Debug)]
pub struct PreparedLeaderPattern<'text> {
    pattern: &'text FinalizedLeaderPattern,
    runs: Vec<LeaderRun>,
    groups: Vec<LeaderGroup>,
    bounds: TextRunBounds,
}

impl<'text> PreparedLeaderPattern<'text> {
    /// Borrows the exact original source, mappings and font/group authority.
    #[must_use]
    pub const fn pattern(&self) -> &'text FinalizedLeaderPattern {
        self.pattern
    }
    /// Returns physical pattern-local Ink or proved Empty.
    #[must_use]
    pub const fn bounds(&self) -> TextRunBounds {
        self.bounds
    }
    /// Borrows original admitted runs in paint order.
    pub fn runs(&self) -> impl ExactSizeIterator<Item = PreparedLeaderRun<'_>> {
        self.runs.iter().map(|run| PreparedLeaderRun { run })
    }
    /// Borrows authentic groups, including groups with no contributing paint.
    pub fn groups(&self) -> impl ExactSizeIterator<Item = PreparedLeaderGroup<'_>> {
        self.groups.iter().map(|group| PreparedLeaderGroup {
            receipt: self,
            group,
        })
    }
}

/// One admitted run view, scoped to its receipt.
#[derive(Clone, Copy, Debug)]
pub struct PreparedLeaderRun<'a> {
    run: &'a LeaderRun,
}
impl<'a> PreparedLeaderRun<'a> {
    /// Original paint-order run ordinal in the finalized pattern.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self.run.ordinal
    }
    /// The actual canonical run, including original ordered paints and resources.
    #[must_use]
    pub const fn run(self) -> &'a PreparedGlyphRun {
        &self.run.run
    }
    /// Actual Text frame, including its independent pen direction.
    #[must_use]
    pub const fn frame(self) -> GlyphFrame {
        self.run.frame
    }
    /// Executed run-to-physical-pattern transform, applied once.
    #[must_use]
    pub const fn transform(self) -> Transform {
        self.run.transform
    }
    /// Original effective foreground paint.
    #[must_use]
    pub const fn paint(self) -> &'a TextPaint {
        &self.run.paint
    }
    /// Original glyph/group associations, including glyphs with Empty ink.
    pub fn glyphs(self) -> impl ExactSizeIterator<Item = PreparedLeaderGlyph<'a>> {
        self.run
            .glyphs
            .iter()
            .enumerate()
            .map(|(index, glyph)| PreparedLeaderGlyph {
                run: self.run,
                index,
                glyph,
            })
    }
}

/// One original glyph contribution borrowed from its admitted run.
#[derive(Clone, Copy, Debug)]
pub struct PreparedLeaderGlyph<'a> {
    run: &'a LeaderRun,
    index: usize,
    glyph: &'a LeaderGlyph,
}
impl<'a> PreparedLeaderGlyph<'a> {
    /// Original glyph ordinal within this run.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self.index
    }
    /// Actual admitted glyph identifier, pen/offset and advance facts.
    #[must_use]
    pub fn glyph(self) -> &'a TextGlyph {
        &self.run.run.glyphs()[self.index]
    }
    /// Original pattern-scoped opaque group; not a cross-pattern identity.
    #[must_use]
    pub fn group(self) -> LeaderPaintGroup {
        self.glyph.group
    }
    /// Admitted physical pattern-local envelope, including proved Empty.
    #[must_use]
    pub fn bounds(self) -> TextRunBounds {
        self.glyph.bounds
    }
    /// Actual owning run, retaining the source font and canonical program.
    #[must_use]
    pub const fn run(self) -> PreparedLeaderRun<'a> {
        PreparedLeaderRun { run: self.run }
    }
}

/// An authentic group and all its admitted contributions across original runs.
#[derive(Clone, Copy, Debug)]
pub struct PreparedLeaderGroup<'a> {
    receipt: &'a PreparedLeaderPattern<'a>,
    group: &'a LeaderGroup,
}
impl<'a> PreparedLeaderGroup<'a> {
    /// Original pattern-scoped group. Compare only within the borrowed pattern.
    #[must_use]
    pub const fn group(self) -> LeaderPaintGroup {
        self.group.group
    }
    /// Union of all admitted physical envelopes in this group, or proved Empty.
    #[must_use]
    pub const fn bounds(self) -> TextRunBounds {
        self.group.bounds
    }
    /// All original member glyph contributions, including Empty members.
    pub fn glyphs(self) -> impl ExactSizeIterator<Item = PreparedLeaderGlyph<'a>> {
        self.group.members.iter().map(|member| PreparedLeaderGlyph {
            run: &self.receipt.runs[member.run],
            index: member.glyph,
            glyph: &self.receipt.runs[member.run].glyphs[member.glyph],
        })
    }
}

/// Admits one finalized Text leader without reshaping or choosing placement.
///
/// Uses only the supplied context's central Ledger limits. All runs and opaque
/// group associations succeed before a receipt is returned. Selected intrinsic
/// Empty glyphs remain paint-order barriers; consecutive foreground outlines
/// retain their original combined NonZero fill. Borrowed Text backing remains
/// outside Render's byte bounds. Later masking must account any metadata it
/// retains; this operation supplies no Scene or frame import of the sidecar.
pub fn prepare_leader_pattern<'text>(
    pattern: &'text FinalizedLeaderPattern,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedLeaderPattern<'text>> {
    accounting.run(
        RenderAttemptKind::LeaderPattern,
        RenderPhase::Preparation,
        |account| account.phase(RenderPhase::Preparation, || prepare_inner(pattern, account)),
    )
}

fn prepare_inner<'text>(
    pattern: &'text FinalizedLeaderPattern,
    account: &Account,
) -> Result<PreparedLeaderPattern<'text>> {
    let max_glyphs = account.phase(RenderPhase::TextPreflight, || {
        let account = account.at(RenderPhase::TextPreflight);
        let mut max_glyphs = 0;
        for (index, run) in pattern.run_views().enumerate() {
            if run.glyphs().next().is_none() {
                continue;
            }
            account.work(RenderWork::TextRuns, 1)?;
            account.work(RenderWork::GlyphInputs, run.glyphs().len())?;
            let instance = run
                .run()
                .effective_instance()
                .map_err(|e| LeaderPatternError::text(index, e))?;
            account.work(RenderWork::CoordinateInputs, instance.axis_count())?;
            let frame = run
                .glyph_frame()
                .map_err(|e| LeaderPatternError::text(index, e))?;
            frame_transform(frame, &account).map_err(|e| LeaderPatternError::render(index, e))?;
            account.work(RenderWork::PaintVisits, 1)?;
            color(run.run().brush()).map_err(|e| LeaderPatternError::render(index, e))?;
            max_glyphs = max_glyphs.max(run.glyphs().len());
        }
        Ok(max_glyphs)
    })?;
    account.retain(size_of::<PreparedLeaderPattern<'_>>())?;
    let mut runs = account.retained(pattern.run_views().len())?;
    let mut groups: AccountedVec<GroupBuilder> =
        account.temporary(pattern.cluster_views().len())?;
    account.phase(RenderPhase::TextPreflight, || {
        let account = account.at(RenderPhase::TextPreflight);
        for cluster in pattern.cluster_views() {
            account.work(RenderWork::LeaderClusterVisits, 1)?;
            if let Some(group) = cluster.paint_group() {
                ensure_group(&mut groups, group, &account)?;
            }
        }
        Ok(())
    })?;
    let mut resources: AccountedVec<ResourceId> = account.temporary(pattern.run_views().len())?;
    let mut glyphs = account.temporary(max_glyphs)?;
    let mut total_bounds = TextRunBounds::empty();
    for (ordinal, run) in pattern.run_views().enumerate() {
        if run.glyphs().next().is_none() {
            continue;
        }
        let instance = run
            .run()
            .effective_instance()
            .map_err(|e| LeaderPatternError::text(ordinal, e))?;
        let frame = run
            .glyph_frame()
            .map_err(|e| LeaderPatternError::text(ordinal, e))?;
        let transform = account.phase(RenderPhase::Coordinates, || {
            frame_transform(frame, &account.at(RenderPhase::Coordinates))
                .map_err(|e| LeaderPatternError::render(ordinal, e))
        })?;
        let face = instance.face().admitted_face();
        let identity = face.resource().identity();
        let mut known = false;
        for prior in &resources {
            account.work(RenderWork::ResourceProbes, 1)?;
            if *prior == identity {
                known = true;
                break;
            }
        }
        if !known {
            account.retain(face.resource().retained_bytes())?;
            resources.push(identity)?;
        }
        glyphs.clear();
        account.phase(RenderPhase::Coordinates, || {
            let account = account.at(RenderPhase::Coordinates);
            for (glyph, _) in run.glyphs() {
                account.work(RenderWork::GlyphVisits, 1)?;
                let direction = frame.pen_direction();
                account.work(RenderWork::CoordinateVisits, 6)?;
                let (dx, dy, pen) = (f64::from(glyph.x()), f64::from(glyph.y()), glyph.pen());
                let (x, y) = if direction.x > 0.0 {
                    (pen + dx, dy)
                } else if direction.x < 0.0 {
                    (dx - pen, dy)
                } else if direction.y > 0.0 {
                    (dx, pen + dy)
                } else {
                    (dx, dy - pen)
                };
                glyphs.push(
                    TextGlyph::try_new(glyph.id(), x, y, glyph.advance())
                        .map_err(|e| LeaderPatternError::render(ordinal, e))?,
                )?;
            }
            Ok(())
        })?;
        let mut coordinates = account.temporary(instance.axis_count())?;
        account.phase(RenderPhase::Coordinates, || {
            let account = account.at(RenderPhase::Coordinates);
            for bits in instance.normalized_coords() {
                account.work(RenderWork::CoordinateVisits, 1)?;
                coordinates.push(
                    NormalizedCoordinate::try_from_bits(bits)
                        .map_err(|e| LeaderPatternError::render(ordinal, e))?,
                )?;
            }
            Ok(())
        })?;
        let (coordinates, coordinate_hold) = coordinates.into_parts()?;
        let synthesis = GlyphSynthesis::try_new(instance.x_shear(), instance.embolden())
            .map_err(|e| LeaderPatternError::render(ordinal, e))?;
        let instance = TextInstance::try_new(instance.font_size(), coordinates, synthesis)
            .map_err(|e| LeaderPatternError::render(ordinal, e))?;
        let font = FontData::from_face(face.clone());
        let (prepared, glyph_bounds) = font
            .prepare_glyph_run_with_bounds(&instance, &glyphs, account)
            .map_err(|e| LeaderPatternError::render(ordinal, e))?;
        let recorded_glyphs = account.phase(RenderPhase::Qualify, || {
            let account = account.at(RenderPhase::Qualify);
            qualify_preparation_placement(transform.into(), &prepared, &account)
                .map_err(|e| LeaderPatternError::render(ordinal, e))?;
            let mut bounds = account.retained(glyph_bounds.len())?;
            for (index, (local_bounds, (_, group))) in
                glyph_bounds.into_iter().zip(run.glyphs()).enumerate()
            {
                account.work(RenderWork::LeaderClusterVisits, 1)?;
                let mapped = map_preparation_bounds(transform.into(), local_bounds, &account)
                    .map_err(|e| LeaderPatternError::render(ordinal, e))?;
                total_bounds = union_bounds(total_bounds, mapped, &account)
                    .map_err(|e| LeaderPatternError::render(ordinal, e))?;
                let group_index = ensure_group(&mut groups, group, &account)?;
                let target = &mut groups[group_index];
                target.bounds = union_bounds(target.bounds, mapped, &account)
                    .map_err(|e| LeaderPatternError::render(ordinal, e))?;
                if target.members.len() == target.members.capacity() {
                    target.members.grow(target.members.capacity().max(1))?;
                }
                target.members.push(Member {
                    run: runs.len(),
                    glyph: index,
                })?;
                bounds.push(LeaderGlyph {
                    group,
                    bounds: mapped,
                })?;
            }
            Ok(bounds)
        })?;
        let paint = account.phase(RenderPhase::Output, || {
            account
                .at(RenderPhase::Output)
                .work(RenderWork::PaintVisits, 1)?;
            TextPaint::try_fill(
                color(run.run().brush())
                    .map_err(|e| LeaderPatternError::render(ordinal, e))?
                    .into(),
            )
            .map_err(|e| LeaderPatternError::render(ordinal, e))
        })?;
        runs.push(LeaderRun {
            ordinal,
            run: prepared,
            frame,
            transform,
            paint,
            glyphs: recorded_glyphs.into_preparation()?.into_retained(),
        })?;
        drop(instance);
        drop(coordinate_hold);
    }
    drop(glyphs);
    drop(resources);
    account.phase(RenderPhase::Output, || {
        let mut owned_groups = account.retained(groups.len())?;
        for group in groups {
            owned_groups.push(LeaderGroup {
                group: group.group,
                bounds: group.bounds,
                members: group.members.into_preparation()?.into_retained(),
            })?;
        }
        Ok(PreparedLeaderPattern {
            pattern,
            bounds: total_bounds,
            runs: runs.into_preparation()?.into_retained(),
            groups: owned_groups.into_preparation()?.into_retained(),
        })
    })
}

fn ensure_group(
    groups: &mut AccountedVec<GroupBuilder>,
    group: LeaderPaintGroup,
    account: &Account,
) -> Result<usize> {
    for (index, candidate) in groups.iter().enumerate() {
        account.work(RenderWork::ResourceProbes, 1)?;
        if candidate.group == group {
            return Ok(index);
        }
    }
    if groups.len() == groups.capacity() {
        groups.grow(groups.capacity().max(1))?;
    }
    let index = groups.len();
    groups.push(GroupBuilder {
        group,
        bounds: TextRunBounds::empty(),
        members: account.retained(0)?,
    })?;
    Ok(index)
}

fn frame_transform(frame: GlyphFrame, account: &Account) -> Result<Transform> {
    let origin = frame.origin();
    let x = frame.outline_x();
    let y = frame.outline_y();
    Transform::try_new_in([x.x, x.y, y.x, y.y, origin.x, origin.y], account)
}

fn union_bounds(a: TextRunBounds, b: TextRunBounds, account: &Account) -> Result<TextRunBounds> {
    let (Some(a), Some(b)) = (a.ink_rect(), b.ink_rect()) else {
        return Ok(if a.ink_rect().is_some() { a } else { b });
    };
    use crate::vello_engine::geometry::action;
    account.work(RenderWork::CoordinateVisits, 8)?;
    let (ax, ay, aw, ah) = (a.x(), a.y(), a.width(), a.height());
    let (bx, by, bw, bh) = (b.x(), b.y(), b.width(), b.height());
    let x0 = action(account, || ax.min(bx))?;
    let y0 = action(account, || ay.min(by))?;
    let ax1 = action(account, || ax + aw)?;
    let bx1 = action(account, || bx + bw)?;
    let ay1 = action(account, || ay + ah)?;
    let by1 = action(account, || by + bh)?;
    let x1 = action(account, || ax1.max(bx1))?;
    let y1 = action(account, || ay1.max(by1))?;
    let width = action(account, || x1 - x0)?;
    let height = action(account, || y1 - y0)?;
    TextRunBounds::try_ink(Rect::try_new(x0, y0, width, height)?)
}

#[cfg(test)]
mod tests;

use super::{Command, Image, Paint, Result};
use crate::{
    accounting::{Account, AccountedVec, RenderPhase, RenderWork},
    paint::PaintKind,
};

/// Measured structural facts for an authored scene, including nested commands.
/// Image hits and misses use exact content within this inspection only.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SceneSummary {
    /// Authored command records, including layer records.
    pub commands: u64,
    /// Authored fill records.
    pub fills: u64,
    /// Authored stroke records.
    pub strokes: u64,
    /// Shape and text-shadow records.
    pub shadows: u64,
    /// Image occurrences, including image paints and prepared bitmap paints.
    pub images: u64,
    /// Authored and prepared glyph occurrences.
    pub glyphs: u64,
    /// Authored nested layers.
    pub layers: u64,
    /// Repeated exact image-content observations.
    pub image_content_hits: u64,
    /// First exact image-content observations.
    pub image_content_misses: u64,
    /// Pixel bytes first observed during this inspection.
    pub first_observed_image_bytes: u64,
}

pub(super) fn inspect_commands(commands: &[Command], account: &Account) -> Result<SceneSummary> {
    account.phase(RenderPhase::SceneInspect, || {
        inspect_body(commands, account)
    })
}
fn inspect_body(commands: &[Command], account: &Account) -> Result<SceneSummary> {
    let mut images = account.temporary(0)?;
    walk(commands, account, &mut images)?;
    Ok(SceneSummary {
        commands: account.work_count(RenderWork::AuthoredCommands)?,
        fills: account.work_count(RenderWork::AuthoredFills)?,
        strokes: account.work_count(RenderWork::AuthoredStrokes)?,
        shadows: account.work_count(RenderWork::AuthoredShadows)?,
        images: account.work_count(RenderWork::AuthoredImages)?,
        glyphs: account.work_count(RenderWork::AuthoredGlyphs)?,
        layers: account.work_count(RenderWork::AuthoredLayers)?,
        image_content_hits: account.work_count(RenderWork::ImageContentHits)?,
        image_content_misses: account.work_count(RenderWork::ImageContentMisses)?,
        first_observed_image_bytes: account.work_count(RenderWork::ImageFirstObservedBytes)?,
    })
}

fn walk<'a>(
    mut commands: &'a [Command],
    account: &Account,
    images: &mut AccountedVec<&'a Image>,
) -> Result<()> {
    let mut pending = account.temporary(0)?;
    loop {
        let Some((command, siblings)) = commands.split_first() else {
            let Some(siblings) = pending.pop() else {
                return Ok(());
            };
            commands = siblings;
            continue;
        };
        commands = siblings;
        account.work(RenderWork::CommandVisits, 1)?;
        account.work(RenderWork::AuthoredCommands, 1)?;
        match command {
            Command::Fill { paint, .. } => {
                account.work(RenderWork::AuthoredFills, 1)?;
                paint_image(paint, account, images)?;
            }
            Command::Stroke { paint, .. } => {
                account.work(RenderWork::AuthoredStrokes, 1)?;
                paint_image(paint, account, images)?;
            }
            Command::Shadow { shadow, .. } => {
                account.work(RenderWork::AuthoredShadows, 1)?;
                paint_image(shadow.paint(), account, images)?;
            }
            Command::Image { image, .. } | Command::ImageRegion { image, .. } => {
                image_observation(image, account, images)?
            }
            Command::PreparedGlyphRun { run, paint, .. } => {
                for _ in run.glyphs() {
                    account.work(RenderWork::GlyphVisits, 1)?;
                    account.work(RenderWork::AuthoredGlyphs, 1)?;
                }
                paint_image(paint.fill(), account, images)?;
                for paint in run.paints() {
                    account.work(RenderWork::PaintVisits, 1)?;
                    if let crate::text::GlyphPaint::Bitmap { image, .. } = paint {
                        image_observation(image, account, images)?;
                    }
                }
            }
            Command::TextRun { glyphs, .. } => {
                for _ in glyphs {
                    account.work(RenderWork::GlyphVisits, 1)?;
                    account.work(RenderWork::AuthoredGlyphs, 1)?;
                }
            }
            Command::TextShadowRun {
                glyphs, shadows, ..
            } => {
                for _ in glyphs {
                    account.work(RenderWork::GlyphVisits, 1)?;
                    account.work(RenderWork::AuthoredGlyphs, 1)?;
                }
                for shadow in shadows.shadows() {
                    account.work(RenderWork::AuthoredShadows, 1)?;
                    paint_image(shadow.paint(), account, images)?;
                }
            }
            Command::Layer { children, .. } => {
                account.work(RenderWork::AuthoredLayers, 1)?;
                if !children.is_empty() {
                    // Only suspend a nonempty continuation. A single-child
                    // chain uses the current slice without scratch backing.
                    if !siblings.is_empty() {
                        if pending.len() == pending.capacity() {
                            pending.grow(pending.capacity().max(1))?;
                        }
                        pending.push(siblings)?;
                    }
                    commands = children;
                }
            }
        }
    }
}

fn paint_image<'a>(
    paint: &'a Paint,
    account: &Account,
    images: &mut AccountedVec<&'a Image>,
) -> Result<()> {
    account.work(RenderWork::PaintVisits, 1)?;
    if let PaintKind::Image(image) = paint.kind() {
        image_observation(image, account, images)?;
    }
    Ok(())
}

fn image_observation<'a>(
    image: &'a Image,
    account: &Account,
    images: &mut AccountedVec<&'a Image>,
) -> Result<()> {
    account.work(RenderWork::AuthoredImages, 1)?;
    let identity = image.content_identity();
    for previous in images.iter() {
        account.work(RenderWork::ResourceProbes, 1)?;
        if previous.content_identity().content_key() != identity.content_key() {
            continue;
        }
        let mut equal = true;
        for (left, right) in previous.bytes.iter().zip(image.bytes.iter()) {
            account.work(RenderWork::ImageIdentityByteComparisons, 1)?;
            if left != right {
                equal = false;
                break;
            }
        }
        if equal && previous.bytes.len() == image.bytes.len() {
            account.work(RenderWork::ImageContentHits, 1)?;
            return Ok(());
        }
    }
    images.push(image)?;
    account.work(RenderWork::ImageContentMisses, 1)?;
    account.work(RenderWork::ImageFirstObservedBytes, image.bytes.len())
}

#[cfg(test)]
mod tests {
    use crate::{
        Color, Image, ImageFit, Layer, Rect, RenderAccountingContext, RenderAccountingOptions,
        RenderLimits, RenderWork, Scene, Size,
    };

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn deeply_nested_layers_can_be_inspected_on_a_bounded_stack() {
        use super::Command;
        use crate::RenderResource;

        std::thread::Builder::new()
            .name("deep scene inspection".into())
            .stack_size(256 * 1024)
            .spawn(|| {
                const LAYERS: u64 = 8_192;
                let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
                for with_siblings in [false, true] {
                    let mut scene = Scene::new();
                    scene.fill(rect, Color::BLACK);
                    for _ in 0..LAYERS {
                        let child = scene;
                        scene = Scene::new();
                        scene.layer(Layer::new(), |target| *target = child);
                        if with_siblings {
                            scene.fill(rect, Color::BLACK);
                        }
                    }
                    let mut context = RenderAccountingContext::default();
                    let summary = scene.inspect(&mut context);

                    // Each layer is first in its sequence. Drop the leaf
                    // siblings before moving to the next child sequence.
                    let mut commands = std::mem::take(&mut scene.commands);
                    while let Some(command) = commands.pop() {
                        if let Command::Layer { mut children, .. } = command {
                            commands = std::mem::take(&mut children.0);
                        }
                    }

                    let fills = 1 + u64::from(with_siblings) * LAYERS;
                    let summary = summary.unwrap();
                    assert_eq!(summary.commands, LAYERS + fills);
                    assert_eq!(summary.layers, LAYERS);
                    assert_eq!(summary.fills, fills);
                    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
                    assert_eq!(usage.work(RenderWork::CommandVisits), LAYERS + fills);
                    let peak = usage.peak(RenderResource::ScratchCpuBytes);
                    if with_siblings {
                        assert!(peak >= LAYERS * std::mem::size_of::<&[Command]>() as u64);
                        assert!(usage.work(RenderWork::StorageRelocations) < 2 * LAYERS);
                    } else {
                        assert_eq!(peak, 0);
                    }
                    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn pending_siblings_are_admitted_before_inspecting_children() {
        use super::Command;
        use crate::{
            RenderAccountingError, RenderPhase, RenderQuantity, RenderRefusal, RenderRefusalReason,
            RenderResource,
        };

        let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
        let mut scene = Scene::new();
        scene.layer(Layer::new(), |child| {
            child.fill(rect, Color::BLACK);
        });
        scene.layer(Layer::new(), |_| {});
        let required = std::mem::size_of::<&[Command]>() as u64;
        let short = required - 1;
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(short)),
        );
        let error = scene.inspect(&mut context).unwrap_err();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(RenderRefusal {
                phase: RenderPhase::Attempt,
                quantity: RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
                current: 0,
                amount: required,
                reason: RenderRefusalReason::Limit { limit: short },
            }))
        );
        let report = context.last_attempt().unwrap();
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::AuthoredCommands), 1);
        assert_eq!(usage.work(RenderWork::AuthoredLayers), 1);
        assert_eq!(usage.work(RenderWork::AuthoredFills), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(report.cleanup_fault(), None);

        let limits = RenderLimits::new()
            .with_scratch_bytes(required)
            .with_work_limit(RenderWork::AuthoredCommands, 2)
            .unwrap();
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        let error = scene.inspect(&mut context).unwrap_err();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(RenderRefusal {
                phase: RenderPhase::Attempt,
                quantity: RenderQuantity::Work(RenderWork::AuthoredCommands),
                current: 2,
                amount: 1,
                reason: RenderRefusalReason::Limit { limit: 2 },
            }))
        );
        let report = context.last_attempt().unwrap();
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::AuthoredLayers), 1);
        assert_eq!(usage.work(RenderWork::AuthoredFills), 1);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), required);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(report.cleanup_fault(), None);

        context.set_options(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(required)),
        );
        let summary = scene.inspect(&mut context).unwrap();
        assert_eq!(summary.commands, 3);
        assert_eq!(summary.layers, 2);
        assert_eq!(summary.fills, 1);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), required);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    }

    #[test]
    fn nested_authored_structure_and_exact_content_are_measured() {
        let first = Image::from_rgba(Size::new(1.0, 1.0), vec![10, 20, 30, 255]).unwrap();
        let equal = Image::from_rgba(Size::new(1.0, 1.0), vec![10, 20, 30, 255]).unwrap();
        let mut scene = Scene::new();
        scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
        scene.layer(Layer::new(), |child| {
            child.image(first, Rect::new(0.0, 0.0, 1.0, 1.0), ImageFit::Fill);
            child.image(equal, Rect::new(1.0, 0.0, 1.0, 1.0), ImageFit::Fill);
        });
        let mut context = RenderAccountingContext::default();
        let summary = scene.inspect(&mut context).unwrap();
        assert_eq!(summary.commands, 4);
        assert_eq!(summary.fills, 1);
        assert_eq!(summary.layers, 1);
        assert_eq!(summary.images, 2);
        assert_eq!(summary.image_content_hits, 1);
        assert_eq!(summary.image_content_misses, 1);
        assert_eq!(summary.first_observed_image_bytes, 4);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::AuthoredCommands), 4);
        assert_eq!(usage.work(RenderWork::CommandsEmitted), 0);
        assert_eq!(usage.work(RenderWork::ImageIdentityByteComparisons), 4);
        // The sole distinct-content observation stores a borrowed reference,
        // not an owned retained identity with a source payload destructor.
        assert_eq!(usage.work(RenderWork::ImageIdentityDrops), 0);
        assert_eq!(
            usage.peak(crate::RenderResource::ScratchCpuBytes),
            std::mem::size_of::<&Image>() as u64
        );
        assert_eq!(usage.live(crate::RenderResource::ScratchCpuBytes), 0);
    }

    #[test]
    fn refusal_preserves_only_accepted_authored_prefix_and_context_retries() {
        let mut scene = Scene::new();
        scene.fill(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        scene.fill(Rect::new(1.0, 0.0, 1.0, 1.0), Color::BLACK);
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::AuthoredCommands, 1)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        assert!(scene.inspect(&mut context).is_err());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::AuthoredCommands),
            1
        );
        context.set_options(RenderAccountingOptions::default());
        assert_eq!(scene.inspect(&mut context).unwrap().commands, 2);
    }

    #[test]
    fn cached_hash_collision_still_compares_exact_image_content() {
        let first = Image::from_rgba(Size::new(1.0, 1.0), vec![10, 20, 30, 255])
            .unwrap()
            .with_content_hash_for_test(7);
        let second = Image::from_rgba(Size::new(1.0, 1.0), vec![10, 20, 31, 255])
            .unwrap()
            .with_content_hash_for_test(7);
        let mut scene = Scene::new();
        let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
        scene
            .image(first, rect, ImageFit::Fill)
            .image(second, rect, ImageFit::Fill);
        let mut context = RenderAccountingContext::default();
        let summary = scene.inspect(&mut context).unwrap();
        assert_eq!(summary.image_content_hits, 0);
        assert_eq!(summary.image_content_misses, 2);
        assert_eq!(summary.first_observed_image_bytes, 8);
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ImageIdentityByteComparisons),
            3
        );
    }
}

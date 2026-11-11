use super::{
    BorderImageArea, BorderImagePartitionRequest, BorderImageRect, BorderImageRepeat,
    BorderImageTiling, GeometryWork, OperationKind, in_geometry, resolve_geometry, run_stages,
    tiling, validate_partition,
};
use crate::{PaintError, PaintOptions, PaintPhase, PaintReport, WorkKind, accounting::charge};
use surgeist_layout::Size;

/// Mask coverage geometry for an admitted border image and one destination area.
#[derive(Debug, PartialEq)]
pub struct MaskBorderPlan {
    area: BorderImageRect,
    tiles: BorderImageTiling,
    opaque_center: Option<BorderImageRect>,
}

impl MaskBorderPlan {
    /// The complete destination area in CSS pixels, including any outset.
    pub const fn area(&self) -> BorderImageRect {
        self.area
    }

    /// Regions that sample the admitted image, including a filled center when present.
    pub const fn tiles(&self) -> &BorderImageTiling {
        &self.tiles
    }

    /// The nonempty destination center with full mask coverage when fill is absent.
    pub const fn opaque_center(&self) -> Option<BorderImageRect> {
        self.opaque_center
    }
}

/// One complete coverage plan or a typed refusal, with its Ledger report.
#[derive(Debug)]
pub struct MaskBorderOutcome {
    pub result: Result<MaskBorderPlan, PaintError>,
    pub report: Option<PaintReport>,
}

/// Resolves mask-border coverage for a concrete image and an admitted paint area.
/// Partitioning and tile planning share one budget and publish one result.
pub fn resolve_mask_border_in_area(
    area: BorderImageArea,
    request: BorderImagePartitionRequest,
    repeat: Size<BorderImageRepeat>,
    options: PaintOptions,
) -> MaskBorderOutcome {
    let (result, report) = run_stages(
        options,
        OperationKind::MaskBorder,
        [
            PaintPhase::BorderImageGeometry,
            PaintPhase::BorderImageTiling,
        ],
        Some(|work: &mut GeometryWork<'_, '_, '_>| validate_partition(work, request)),
        |meter, [partition, tiling]| {
            let geometry = in_geometry(meter, partition, |work| {
                resolve_geometry(work, request, |_| Ok(area))
            })?;
            in_geometry(meter, tiling, |work| {
                let tiles = tiling::resolve(work, &geometry, repeat)?;
                let opaque_center = if request.fill_center {
                    None
                } else {
                    work.coordinate(|| geometry.regions()[1][1].destination())?
                };
                if opaque_center.is_some() {
                    charge(work.phase, work.scope, WorkKind::OutputRecord, 1)?;
                }
                Ok(MaskBorderPlan {
                    area: geometry.area(),
                    tiles,
                    opaque_center,
                })
            })
        },
    );
    MaskBorderOutcome { result, report }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BorderImageAreaRequest, BorderImageOutset, BorderImageSlice, BorderImageSource,
        BorderImageWidth, PaintChange, PaintLimits, ProfileMode, ResourceKind,
        resolve_border_image_area,
    };
    use surgeist_layout::{Edges, Point};

    fn options(limits: PaintLimits) -> PaintOptions {
        PaintOptions::new(limits, ProfileMode::Summary { timing: false })
    }

    fn area(width: f64) -> BorderImageArea {
        resolve_border_image_area(
            BorderImageAreaRequest {
                border_origin: Point::new(0.0, 0.0),
                border_size: Size::new(width, 20.0),
                border_widths: Edges::all(2.0),
                outsets: Edges::all(BorderImageOutset::Length(0.0)),
            },
            options(PaintLimits::unlimited()),
        )
        .result
        .unwrap()
    }

    fn request() -> BorderImagePartitionRequest {
        BorderImagePartitionRequest {
            source: BorderImageSource {
                coordinate_size: Size::new(10.0, 10.0),
                natural_size: Size::new(Some(10.0), Some(10.0)),
            },
            slices: Edges::all(BorderImageSlice::SourceUnits(2.0)),
            widths: Edges::all(BorderImageWidth::BorderWidthMultiple(1.0)),
            fill_center: false,
        }
    }

    #[test]
    fn mask_border_omits_opaque_coverage_for_a_collapsed_center() {
        let area = area(4.0);
        let plan = resolve_mask_border_in_area(
            area,
            request(),
            Size::new(BorderImageRepeat::Stretch, BorderImageRepeat::Stretch),
            options(PaintLimits::unlimited()),
        )
        .result
        .unwrap();
        assert_eq!(plan.area(), area.rect());
        assert!(plan.tiles().regions()[0][0].is_some());
        assert!(plan.tiles().regions()[1][1].is_none());
        assert_eq!(plan.opaque_center(), None);
    }

    #[test]
    fn mask_border_accounts_for_partition_tiles_opaque_center_and_publication_once() {
        let area = area(20.0);
        let repeat = Size::new(BorderImageRepeat::Stretch, BorderImageRepeat::Stretch);
        let kind = WorkKind::OutputRecord;
        let retained = ResourceKind::RetainedResultBytes;
        let result_bytes = u64::try_from(size_of::<MaskBorderPlan>()).unwrap();
        for (cap, phase, peak) in [
            (17, PaintPhase::BorderImageTiling, 0),
            (18, PaintPhase::Publication, result_bytes),
        ] {
            let outcome = resolve_mask_border_in_area(
                area,
                request(),
                repeat,
                options(PaintLimits::unlimited().with_work_limit(kind, cap)),
            );
            let Err(PaintError::Budget(refusal)) = outcome.result else {
                panic!("expected the next record to refuse");
            };
            assert_eq!(refusal.origin, phase);
            assert_eq!(refusal.current, cap);
            assert_eq!(refusal.change, PaintChange::Work { kind, amount: 1 });
            let report = outcome.report.unwrap();
            assert_eq!(report.work(kind), cap);
            assert_eq!(report.peak(retained), peak);
            assert_eq!(report.live(retained), 0);
        }

        let outcome = resolve_mask_border_in_area(
            area,
            request(),
            repeat,
            options(PaintLimits::unlimited().with_work_limit(kind, 19)),
        );
        let plan = outcome.result.unwrap();
        let center = plan.opaque_center().unwrap();
        assert_eq!(center.min(), Point::new(2.0, 2.0));
        assert_eq!(center.max(), Point::new(18.0, 18.0));
        assert!(plan.tiles().regions()[1][1].is_none());
        let report = outcome.report.unwrap();
        assert_eq!(report.work(kind), 19);
        assert_eq!(report.live(retained), result_bytes);
        for (phase, records) in [
            (PaintPhase::BorderImageGeometry, 9),
            (PaintPhase::BorderImageTiling, 9),
            (PaintPhase::Publication, 1),
        ] {
            let phase = report.phases().find(|item| item.phase() == phase).unwrap();
            assert_eq!(phase.work(kind), records);
            assert_eq!(phase.invocations(), 1);
        }
    }
}

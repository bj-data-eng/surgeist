use super::*;
use crate::{
    RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderLimits,
    RenderProfile, RenderQuantity, RenderResource, TextRunBoundsKind,
};
use surgeist_ledger::Dimension;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn system(bytes: Vec<u8>) -> text::System {
    let mut system = text::System::default();
    system
        .register_fonts(bytes, surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    system
}
fn input(source: &str) -> text::LeaderPatternInput {
    text::LeaderPatternInput {
        pattern: text::LeaderPattern::Literal(source.into()),
        source_identity: text::SourceIdentity::new(None, text::SourceRevision::new(3)),
        style: text::Style {
            font: text::Font::new().family("Ahem"),
            size: 20.0,
            brush: text::Brush::color(0.0, 0.0, 0.0, 0.5),
            ..text::Style::default()
        },
        direction: text::ParagraphBaseDirection::LeftToRight,
        writing_axis: text::SourceWritingAxis::Horizontal,
        typography: text::AnalysisTypography::HorizontalComposition,
        orientation: text::LineOrientation::Horizontal,
        scale: 1.0,
        tab_reference: None,
        combination: None,
    }
}
fn pattern(source: &str) -> text::FinalizedLeaderPattern {
    system(AHEM.to_vec())
        .prepare_leader_pattern(input(source))
        .unwrap()
}
fn ink(x: f64, y: f64, w: f64, h: f64) -> TextRunBounds {
    TextRunBounds::try_ink(Rect::try_new(x, y, w, h).unwrap()).unwrap()
}

#[test]
fn authentic_ahem_pattern_retains_original_groups_source_font_and_literal_ink() {
    let mut producer = system(AHEM.to_vec());
    let pattern = producer.prepare_leader_pattern(input("A\r\nA")).unwrap();
    assert_eq!(
        pattern.processed_source().processed_text().code_units(),
        [65, 65]
    );
    let source_runs: Vec<_> = pattern.run_views().collect();
    let original = source_runs[0].run().effective_instance().unwrap();
    let identity = original.face().admitted_face().resource().identity();
    // A later font publication must not replace a retained old pattern's face.
    producer
        .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let stats = producer.stats();
    let mut accounting = RenderAccountingContext::default();
    let ready = prepare_leader_pattern(&pattern, &mut accounting).unwrap();
    assert!(std::ptr::eq(ready.pattern(), &pattern));
    assert_eq!(
        accounting.last_attempt().unwrap().kind(),
        RenderAttemptKind::LeaderPattern
    );
    assert_eq!(producer.stats(), stats);
    assert_eq!(ready.bounds(), ink(0.0, -16.0, 40.0, 20.0));
    let run = ready.runs().next().unwrap();
    assert_eq!(run.transform(), Transform::identity());
    assert_eq!(
        run.run().font().admitted_face().resource().identity(),
        identity
    );
    assert_eq!(run.run().instance().size(), 20.0);
    assert!(run.run().instance().coordinates().is_empty());
    assert_eq!(run.run().instance().synthesis(), GlyphSynthesis::none());
    assert_eq!(
        run.run().glyphs(),
        [
            TextGlyph::try_new(35, 0.0, 0.0, 20.0).unwrap(),
            TextGlyph::try_new(35, 20.0, 0.0, 20.0).unwrap()
        ]
    );
    assert_eq!(run.run().paints().len(), 1);
    let originals: Vec<_> = source_runs[0].glyphs().map(|(_, group)| group).collect();
    assert_eq!(
        run.glyphs().map(|glyph| glyph.group()).collect::<Vec<_>>(),
        originals
    );
    let groups: Vec<_> = ready.groups().collect();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].bounds(), ink(0.0, -16.0, 20.0, 20.0));
    assert_eq!(groups[1].bounds(), ink(20.0, -16.0, 20.0, 20.0));
    for (index, group) in groups.iter().enumerate() {
        let glyph = group.glyphs().next().unwrap();
        assert_eq!(glyph.group(), group.group());
        assert_eq!(glyph.ordinal(), index);
        assert_eq!(glyph.bounds(), group.bounds());
    }
    let crate::text::GlyphPaint::Fill {
        elements,
        color,
        clip,
    } = &run.run().paints()[0]
    else {
        panic!("ordinary Ahem glyphs must share a fill");
    };
    assert_eq!(elements, &(0..10));
    assert!(color.is_none() && clip.is_none());
    assert_eq!(
        run.glyphs().map(|glyph| glyph.bounds()).collect::<Vec<_>>(),
        [ink(0.0, -16.0, 20.0, 20.0), ink(20.0, -16.0, 20.0, 20.0)]
    );
    assert_eq!(
        accounting
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::GlyphInputs),
        2
    );
}

#[test]
fn spaces_and_empty_patterns_keep_proved_empty_distinct_from_positive_advance() {
    for source in ["", " "] {
        let pattern = pattern(source);
        let ready =
            prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
        assert_eq!(ready.bounds().kind(), TextRunBoundsKind::Empty);
        assert_eq!(
            pattern.metrics().advance,
            if source.is_empty() { 0.0 } else { 20.0 }
        );
        for run in ready.runs() {
            assert_eq!(run.run().bounds().kind(), TextRunBoundsKind::Empty);
            assert!(run.run().paints().is_empty());
            for glyph in run.glyphs() {
                assert_eq!(glyph.bounds().kind(), TextRunBoundsKind::Empty);
            }
        }
        assert!(
            ready
                .groups()
                .all(|group| group.bounds().kind() == TextRunBoundsKind::Empty)
        );
        assert_eq!(
            ready.groups().len(),
            pattern
                .cluster_views()
                .filter(|c| c.paint_group().is_some())
                .count()
        );
    }
}

#[test]
fn physical_clockwise_counterclockwise_and_rtl_frames_have_literal_envelopes() {
    for (orientation, expected, transform) in [
        (
            text::LineOrientation::SidewaysClockwise,
            ink(-4.0, 0.0, 20.0, 40.0),
            [0.0, 1.0, -1.0, 0.0, 0.0, 0.0],
        ),
        (
            text::LineOrientation::SidewaysCounterclockwise,
            ink(-16.0, -40.0, 20.0, 40.0),
            [0.0, -1.0, 1.0, 0.0, 0.0, 0.0],
        ),
    ] {
        let mut request = input("AA");
        request.orientation = orientation;
        request.writing_axis = text::SourceWritingAxis::Vertical;
        let pattern = system(AHEM.to_vec())
            .prepare_leader_pattern(request)
            .unwrap();
        let ready =
            prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
        assert_eq!(ready.bounds(), expected);
        assert_eq!(
            ready.runs().next().unwrap().transform().as_array(),
            transform
        );
    }
    let mut request = input("AA");
    request.direction = text::ParagraphBaseDirection::RightToLeft;
    let pattern = system(AHEM.to_vec())
        .prepare_leader_pattern(request)
        .unwrap();
    let ready = prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
    assert_eq!(ready.bounds(), ink(0.0, -16.0, 40.0, 20.0));
    assert_eq!(
        ready.runs().next().unwrap().frame().pen_direction(),
        text::Point::new(1.0, 0.0)
    );
}

fn assert_released(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    for resource in [
        RenderResource::ScratchCpuBytes,
        RenderResource::RetainedCpuBytes,
    ] {
        assert_eq!(report.attempt().unwrap().usage().live(resource), 0);
    }
}

#[test]
fn central_work_and_storage_caps_refuse_atomically_and_exact_budget_retries() {
    let pattern = pattern("AAA");
    let mut context = RenderAccountingContext::default();
    let ready = prepare_leader_pattern(&pattern, &mut context).unwrap();
    assert_eq!(ready.bounds(), ink(0.0, -16.0, 60.0, 20.0));
    drop(ready);
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    let retained = usage.peak(RenderResource::RetainedCpuBytes);
    let scratch = usage.peak(RenderResource::ScratchCpuBytes);
    let work: Vec<_> = RenderWork::ALL
        .iter()
        .map(|&w| (w, usage.work(w)))
        .filter(|(w, n)| *n > 0 && w.supported_for(RenderAttemptKind::LeaderPattern))
        .collect();
    let mut exact = RenderLimits::new()
        .with_retained_bytes(retained)
        .with_scratch_bytes(scratch);
    for &(work, amount) in &work {
        exact = exact.with_work_limit(work, amount).unwrap();
    }
    for (limits, quantity) in [
        (
            exact.with_retained_bytes(retained - 1),
            RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
        ),
        (
            exact.with_scratch_bytes(scratch - 1),
            RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
        ),
        (
            exact
                .with_work_limit(RenderWork::LeaderClusterVisits, 5)
                .unwrap(),
            RenderQuantity::Work(RenderWork::LeaderClusterVisits),
        ),
        (
            exact.with_work_limit(RenderWork::GlyphInputs, 2).unwrap(),
            RenderQuantity::Work(RenderWork::GlyphInputs),
        ),
    ] {
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        let error = prepare_leader_pattern(&pattern, &mut context).unwrap_err();
        let report = context.last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        let refusal = report.attempt().unwrap().refusal().unwrap();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        assert_eq!(refusal.quantity(), quantity);
        assert!(error.leader_pattern_diagnostic().is_none());
        assert_released(&context);
        context.set_options(RenderAccountingOptions::new().with_limits(exact));
        assert_eq!(
            prepare_leader_pattern(&pattern, &mut context)
                .unwrap()
                .bounds(),
            ink(0.0, -16.0, 60.0, 20.0)
        );
    }
}

#[test]
fn off_untimed_and_timed_profiles_preserve_work_and_truthful_origins() {
    let pattern = pattern("AA");
    let mut expected = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ] {
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_profile(profile));
        prepare_leader_pattern(&pattern, &mut context).unwrap();
        let attempt = context.last_attempt().unwrap().attempt().unwrap();
        let work: Vec<_> = RenderWork::ALL
            .iter()
            .map(|&w| attempt.usage().work(w))
            .collect();
        if let Some(expected) = &expected {
            assert_eq!(&work, expected);
        } else {
            expected = Some(work);
        }
        assert_eq!(
            attempt
                .phase_usage(RenderPhase::TextPreflight)
                .work(RenderWork::GlyphInputs),
            2
        );
        assert_eq!(
            attempt
                .phase_usage(RenderPhase::TextPreflight)
                .work(RenderWork::TextRuns),
            1
        );
        assert_eq!(
            attempt
                .phase_usage(RenderPhase::TextPreflight)
                .work(RenderWork::LeaderClusterVisits),
            2
        );
        assert!(
            attempt
                .phase_usage(RenderPhase::GlyphPrepare)
                .work(RenderWork::GlyphVisits)
                > 0
        );
        assert!(
            attempt
                .phase_usage(RenderPhase::Qualify)
                .work(RenderWork::LeaderClusterVisits)
                > 0
        );
    }
}

#[path = "fixtures.rs"]
mod fixtures;

#[test]
fn zero_advance_ink_and_negative_bearing_remain_representation_feedback() {
    let pattern = system(fixtures::zero_advance())
        .prepare_leader_pattern(input("A"))
        .unwrap();
    assert_eq!(pattern.metrics().advance, 0.0);
    let ready = prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
    assert_eq!(ready.bounds(), ink(0.0, -16.0, 20.0, 20.0));
    assert_eq!(
        ready.runs().next().unwrap().run().glyphs()[0].advance(),
        0.0
    );
    // A real A contour translated -100 design units inside X, with matching
    // compound bounds and side bearing; no reconstructed glyph is admitted.
    let pattern = system(fixtures::negative_bearing())
        .prepare_leader_pattern(input("X"))
        .unwrap();
    let ready = prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
    assert_eq!(ready.bounds(), ink(-2.0, -16.0, 20.0, 20.0));
    assert_eq!(ready.groups().next().unwrap().bounds(), ready.bounds());
}

#[test]
fn compressed_combined_cell_keeps_one_authentic_group_and_horizontal_inner_frame() {
    for (orientation, transform, bounds) in [
        (
            text::LineOrientation::SidewaysClockwise,
            [0.5, 0.0, 0.0, 1.0, -4.0, 16.0],
            ink(-4.0, 0.0, 20.0, 20.0),
        ),
        (
            text::LineOrientation::SidewaysCounterclockwise,
            [0.5, 0.0, 0.0, 1.0, -16.0, -4.0],
            ink(-16.0, -20.0, 20.0, 20.0),
        ),
    ] {
        let mut request = input("A\r\nA");
        request.writing_axis = text::SourceWritingAxis::Vertical;
        request.orientation = orientation;
        request.style.text_combine_upright = text::TextCombineUpright::All;
        request.combination = Some(text::LeaderCombinationContext {
            horizontal_inner_direction: text::ParagraphBaseDirection::LeftToRight,
            font_reference: text::ScopeFontReference {
                script: text::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
                axis: text::FontBaselineAxis::HorizontalY,
                baseline: text::FontBaseline::Alphabetic,
            },
            limits: text::ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216),
        });
        let pattern = system(AHEM.to_vec())
            .prepare_leader_pattern(request)
            .unwrap();
        assert_eq!(pattern.metrics().advance, 20.0);
        let ready =
            prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
        assert_eq!(ready.groups().len(), 1);
        let group = ready.groups().next().unwrap();
        assert_eq!(group.glyphs().len(), 2);
        assert_eq!(group.bounds(), bounds);
        let run = ready.runs().next().unwrap();
        assert_eq!(run.frame().pen_direction(), text::Point::new(1.0, 0.0));
        assert_eq!(run.transform().as_array(), transform);
        assert_eq!(run.run().glyphs()[1].x(), 20.0);
        assert_eq!(run.run().paints().len(), 1);
        assert!(run.glyphs().all(|glyph| glyph.group() == group.group()));
    }
}

#[test]
fn selected_empty_intrinsic_color_remains_a_barrier_between_foreground_spans() {
    let pattern = system(fixtures::empty_color())
        .prepare_leader_pattern(input("AXA"))
        .unwrap();
    let ready = prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
    let run = ready.runs().next().unwrap();
    assert_eq!(run.run().paints().len(), 2);
    assert_eq!(
        run.glyphs().nth(1).unwrap().bounds().kind(),
        TextRunBoundsKind::Empty
    );
    let ranges: Vec<_> = run
        .run()
        .paints()
        .iter()
        .map(|paint| match paint {
            crate::text::GlyphPaint::Fill {
                elements,
                color,
                clip,
            } => {
                assert!(color.is_none() && clip.is_none());
                elements.clone()
            }
            _ => panic!("no bitmap was authored"),
        })
        .collect();
    assert_eq!(ranges, [0..5, 5..10]);
    assert_eq!(
        run.glyphs().nth(2).unwrap().bounds(),
        ink(40.0, -16.0, 20.0, 20.0)
    );
    assert_eq!(ready.groups().len(), 3);
    assert_eq!(
        ready.groups().nth(1).unwrap().bounds().kind(),
        TextRunBoundsKind::Empty
    );
}

#[test]
fn late_unsupported_actual_run_preserves_typed_cause_without_a_receipt() {
    let pattern = system(fixtures::composite_x_ahem(Some(8192)))
        .prepare_leader_pattern(input("AאX"))
        .unwrap();
    let bad_run = pattern
        .run_views()
        .enumerate()
        .find(|(_, run)| run.glyphs().any(|(g, _)| g.id() == 58))
        .unwrap()
        .0;
    assert!(
        bad_run > 0,
        "the unsupported glyph must occur after a successfully admitted actual run"
    );
    let mut context = RenderAccountingContext::default();
    let error = prepare_leader_pattern(&pattern, &mut context).unwrap_err();
    let LeaderPatternError::Run {
        run,
        cause: LeaderPatternCause::Render(cause),
    } = error.leader_pattern_diagnostic().unwrap()
    else {
        panic!("the original Render failure must retain its actual run");
    };
    assert_eq!(*run, bad_run);
    assert_eq!(
        cause.glyph_preparation_diagnostic(),
        Some(&crate::GlyphPreparationError::UnsupportedOutline {
            capability: crate::GlyphCapability::CompositeGlyph
        })
    );
    assert_eq!(
        context.last_attempt().unwrap().status(),
        RenderAttemptStatus::Failed(crate::ErrorCode::LeaderPattern)
    );
    assert_released(&context);
    assert_eq!(
        pattern.source().text().code_units(),
        "AאX".encode_utf16().collect::<Vec<_>>()
    );
}

#[test]
fn glyphless_preserved_tabs_keep_their_original_empty_groups() {
    let mut request = input("A\t\tB");
    request.style.white_space = text::WhiteSpace::Preserve;
    request.tab_reference =
        Some(text::ParagraphTabReference::new(request.style.clone(), request.direction).unwrap());
    let pattern = system(AHEM.to_vec())
        .prepare_leader_pattern(request)
        .unwrap();
    let ready = prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
    assert_eq!(ready.groups().len(), 4);
    assert_eq!(
        ready.groups().map(|g| g.glyphs().len()).collect::<Vec<_>>(),
        [1, 0, 0, 1]
    );
    let originals: Vec<_> = pattern
        .cluster_views()
        .map(|c| c.paint_group().unwrap())
        .collect();
    assert_eq!(
        ready.groups().map(|g| g.group()).collect::<Vec<_>>(),
        originals
    );
    assert!(
        ready
            .groups()
            .skip(1)
            .take(2)
            .all(|g| g.bounds().kind() == TextRunBoundsKind::Empty)
    );
}

#[test]
fn combined_group_unions_every_actual_run_and_retains_both_font_resources() {
    let (primary, fallback) = fixtures::fallback_fonts();
    let mut producer = system(primary.clone());
    producer
        .register_fonts(fallback.clone(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let mut request = input("AX");
    request.style.font = text::Font::new().family("Ahem").family("Bhem");
    request.writing_axis = text::SourceWritingAxis::Vertical;
    request.orientation = text::LineOrientation::SidewaysClockwise;
    request.style.text_combine_upright = text::TextCombineUpright::All;
    request.combination = Some(text::LeaderCombinationContext {
        horizontal_inner_direction: text::ParagraphBaseDirection::LeftToRight,
        font_reference: text::ScopeFontReference {
            script: text::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            axis: text::FontBaselineAxis::HorizontalY,
            baseline: text::FontBaseline::Alphabetic,
        },
        limits: text::ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216),
    });
    let pattern = producer.prepare_leader_pattern(request).unwrap();
    assert_eq!(pattern.run_views().len(), 2);
    let ready = prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
    assert_eq!(ready.runs().len(), 2);
    assert_eq!(ready.groups().len(), 1);
    let group = ready.groups().next().unwrap();
    assert_eq!(group.glyphs().len(), 2);
    assert_eq!(group.bounds(), ink(-4.0, 0.0, 20.0, 20.0));
    let runs: Vec<_> = ready.runs().collect();
    assert_eq!(runs[0].run().font().bytes(), primary);
    assert_eq!(runs[1].run().font().bytes(), fallback);
    assert_eq!(
        runs[0].transform().as_array(),
        [0.5, 0.0, 0.0, 1.0, -4.0, 16.0]
    );
    assert_eq!(
        runs[1].transform().as_array(),
        [0.5, 0.0, 0.0, 1.0, 6.0, 16.0]
    );
    assert_eq!(
        group
            .glyphs()
            .map(|g| g.run().ordinal())
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn normalized_instances_and_synthesis_come_from_each_actual_text_preparation() {
    let mut producer = system(fixtures::variable_ahem());
    for (setting, bits, width) in [(0.0, 0, 1000.0), (0.5, 8192, 1050.0), (1.0, 16384, 1100.0)] {
        let mut request = input("X");
        request.style.size = 1000.0;
        request.style.font = text::Font::new()
            .family("Ahem")
            .variation(format!("'wght' {setting}"));
        let pattern = producer.prepare_leader_pattern(request).unwrap();
        let mut context = RenderAccountingContext::default();
        let ready = prepare_leader_pattern(&pattern, &mut context).unwrap();
        let run = ready.runs().next().unwrap();
        assert_eq!(run.run().instance().coordinates()[0].bits(), bits);
        assert_eq!(ready.bounds(), ink(0.0, -800.0, width, 1000.0));
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateInputs),
            1
        );
    }
    let mut request = input("A");
    request.style.font = text::Font::new()
        .family("Ahem")
        .weight(text::Weight::Bold)
        .style(text::Slant::Italic);
    let pattern = system(AHEM.to_vec())
        .prepare_leader_pattern(request)
        .unwrap();
    let original_run = pattern.run_views().next().unwrap();
    let expected = original_run.run().effective_instance().unwrap();
    let ready = prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
    let actual = ready.runs().next().unwrap().run().instance().synthesis();
    assert_eq!(actual.skew(), expected.x_shear());
    assert_eq!(actual.embolden(), expected.embolden());
}

#[test]
fn bitmap_callback_reports_admitted_rectangle_and_transparent_selection_empty() {
    for alpha in [0, 255] {
        let pattern = system(fixtures::bitmap(alpha))
            .prepare_leader_pattern(input("X"))
            .unwrap();
        let mut context = RenderAccountingContext::default();
        let ready = prepare_leader_pattern(&pattern, &mut context).unwrap();
        let expected = if alpha == 0 {
            TextRunBounds::empty()
        } else {
            ink(1.0, -1.0, 2.0, 3.0)
        };
        assert_eq!(ready.bounds(), expected);
        assert_eq!(ready.groups().next().unwrap().bounds(), expected);
        assert_eq!(
            ready
                .runs()
                .next()
                .unwrap()
                .glyphs()
                .next()
                .unwrap()
                .bounds(),
            expected
        );
        let attempt = context.last_attempt().unwrap().attempt().unwrap();
        assert!(
            attempt
                .phase_usage(RenderPhase::Png)
                .work(RenderWork::PngByteVisits)
                > 0
        );
        assert!(
            attempt
                .phase_usage(RenderPhase::Bitmap)
                .work(RenderWork::ImageTexelVisits)
                > 0
        );
    }
}

#[test]
fn one_resource_shared_by_multiple_actual_runs_is_retained_once() {
    use crate::{RenderChangeKind, RenderEvent, RenderPhaseFilter};
    let pattern = pattern("AאA");
    assert!(pattern.run_views().len() > 1);
    let font_bytes = pattern
        .run_views()
        .next()
        .unwrap()
        .run()
        .face()
        .admitted_face()
        .resource()
        .retained_bytes() as u64;
    let mut context = RenderAccountingContext::new(RenderAccountingOptions::new().with_profile(
        RenderProfile::Detailed {
            timing: false,
            event_limit: 8192,
            filter: RenderPhaseFilter::All,
        },
    ));
    let ready = prepare_leader_pattern(&pattern, &mut context).unwrap();
    assert_eq!(ready.runs().len(), pattern.run_views().len());
    let attempt = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(attempt.dropped_events(), 0);
    assert_eq!(attempt.events().filter(|event|matches!(event,RenderEvent::Change {
        quantity:RenderQuantity::Resource(RenderResource::RetainedCpuBytes),amount,kind:RenderChangeKind::Acquire,..
    } if *amount==font_bytes)).count(),1);
}

#[test]
fn real_gpu_overlapping_leader_outlines_keep_nonzero_spans_and_empty_barriers() {
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let precisions = renderer
        .default_device_capabilities_for_test()
        .effect_precisions();
    let format = if precisions.supports_high_precision() {
        crate::resource::WorkingFormat::HighPrecision
    } else {
        assert!(precisions.supports_reduced_precision());
        crate::resource::WorkingFormat::ReducedPrecision
    };
    let mut surface =
        pollster::block_on(renderer.create_headless(crate::Size::new(2.0, 2.0), 1.0)).unwrap();
    for (font, source, spacing, paints, alpha) in [
        (fixtures::zero_advance(), "AA", 0.0, 1, 128_u8),
        (fixtures::empty_color(), "AXA", -2.0, 2, 192_u8),
    ] {
        let mut request = input(source);
        request.style.size = 2.0;
        request.style.letter_spacing = spacing;
        let pattern = system(font).prepare_leader_pattern(request).unwrap();
        let ready =
            prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
        let run = ready.runs().next().unwrap();
        assert!(run.run().glyphs().iter().all(|g| g.x() == 0.0));
        assert_eq!(run.run().paints().len(), paints);
        let mut scene = crate::Scene::new();
        // Replay the actual canonical run once. The independent baseline maps
        // size2 Ahem's font-space top800/bottom-200 onto the tiny target.
        scene.prepared_glyph_run(
            run.run().clone(),
            Transform::translation(0.0, 1.6).unwrap(),
            run.paint().clone(),
        );
        for graph in [false, true] {
            if graph {
                pollster::block_on(renderer.render_forced_base_graph_for_test(
                    &mut surface,
                    &scene,
                    crate::Parameters::default(),
                    format,
                ))
                .unwrap();
            } else {
                pollster::block_on(renderer.render(
                    &mut surface,
                    &scene,
                    crate::Parameters::default(),
                ))
                .unwrap();
            }
            let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for pixel in image.rgba().as_chunks::<4>().0 {
                assert_eq!(&pixel[..3], &[0, 0, 0]);
                assert!(
                    pixel[3].abs_diff(alpha) <= 2,
                    "graph={graph},source={source}: canonical span alpha {alpha}, actual {pixel:?}"
                );
            }
        }
    }
}

#[test]
fn whole_upright_pattern_keeps_physical_vertical_pens_and_unrotated_ink() {
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12];
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    for n in [28_u32, 0, 1, 0x4e00, 0x4e00, 274] {
        cmap.extend(n.to_be_bytes());
    }
    let font = fixtures::replace_tables(vec![(*b"cmap", cmap)]);
    let mut request = input("一一");
    request.writing_axis = text::SourceWritingAxis::Vertical;
    request.orientation = text::LineOrientation::Vertical;
    request.typography = text::AnalysisTypography::Upright {
        computed_direction: text::ParagraphBaseDirection::RightToLeft,
    };
    let pattern = system(font).prepare_leader_pattern(request).unwrap();
    let ready = prepare_leader_pattern(&pattern, &mut RenderAccountingContext::default()).unwrap();
    let run = ready.runs().next().unwrap();
    assert_eq!(run.transform(), Transform::identity());
    assert_eq!(run.frame().pen_direction(), text::Point::new(0.0, 1.0));
    assert_eq!(
        run.run().glyphs(),
        [
            TextGlyph::try_new(274, -10.0, 16.0, 20.0).unwrap(),
            TextGlyph::try_new(274, -10.0, 36.0, 20.0).unwrap(),
        ]
    );
    // Ahem chi is x[0,1000],y[400,600]. Nominal vertical origin is
    // (500,800), and the second pen adds1000 only along physical y.
    assert_eq!(ready.bounds(), ink(-10.0, 4.0, 20.0, 24.0));
    assert_eq!(
        ready
            .groups()
            .map(|group| group.bounds())
            .collect::<Vec<_>>(),
        [ink(-10.0, 4.0, 20.0, 4.0), ink(-10.0, 24.0, 20.0, 4.0),]
    );
}

#[test]
fn intrinsic_path_and_pixel_resource_refusals_release_metadata_and_retry() {
    for (font, work, expected) in [
        (
            fixtures::color(255),
            RenderWork::PaintVisits,
            ink(0.0, -16.0, 20.0, 20.0),
        ),
        (
            fixtures::bitmap(255),
            RenderWork::PngByteVisits,
            ink(1.0, -1.0, 2.0, 3.0),
        ),
        (
            fixtures::bitmap(0),
            RenderWork::ImageTexelVisits,
            TextRunBounds::empty(),
        ),
    ] {
        let pattern = system(font).prepare_leader_pattern(input("X")).unwrap();
        let mut context = RenderAccountingContext::default();
        assert_eq!(
            prepare_leader_pattern(&pattern, &mut context)
                .unwrap()
                .bounds(),
            expected
        );
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        let retained = usage.peak(RenderResource::RetainedCpuBytes);
        let scratch = usage.peak(RenderResource::ScratchCpuBytes);
        let total = usage.work(work);
        assert!(total > 0);
        for limits in [
            RenderLimits::new().with_retained_bytes(retained - 1),
            RenderLimits::new().with_scratch_bytes(scratch - 1),
            RenderLimits::new()
                .with_work_limit(work, total - 1)
                .unwrap(),
        ] {
            context.set_options(RenderAccountingOptions::new().with_limits(limits));
            let error = prepare_leader_pattern(&pattern, &mut context).unwrap_err();
            let report = context.last_attempt().unwrap();
            let refusal = report.attempt().unwrap().refusal().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(
                error.accounting_diagnostic(),
                Some(RenderAccountingError::Refused(refusal))
            );
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::PreparedTransferredBytes),
                0
            );
            assert_released(&context);
            let exact = RenderLimits::new()
                .with_retained_bytes(retained)
                .with_scratch_bytes(scratch)
                .with_work_limit(work, total)
                .unwrap();
            context.set_options(RenderAccountingOptions::new().with_limits(exact));
            assert_eq!(
                prepare_leader_pattern(&pattern, &mut context)
                    .unwrap()
                    .bounds(),
                expected
            );
        }
    }
}

#[test]
fn proved_empty_envelope_has_no_native_placement_operands() {
    let transform = kurbo::Affine::new([1.0, 0.0, 0.0, 1.0, 1e100, 0.0]);
    let mut context = RenderAccountingContext::default();
    let mapped = context
        .run(
            RenderAttemptKind::LeaderPattern,
            RenderPhase::Qualify,
            |account| {
                account.phase(RenderPhase::Qualify, || {
                    map_preparation_bounds(transform, TextRunBounds::empty(), account)
                })
            },
        )
        .unwrap();
    assert_eq!(mapped, TextRunBounds::empty());
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CoordinateVisits),
        0
    );
    assert_released(&context);

    let error = context
        .run(
            RenderAttemptKind::LeaderPattern,
            RenderPhase::Qualify,
            |account| {
                account.phase(RenderPhase::Qualify, || {
                    map_preparation_bounds(transform, ink(0.0, -16.0, 20.0, 20.0), account)
                })
            },
        )
        .unwrap_err();
    assert!(
        matches!(error.glyph_preparation_diagnostic(),Some(crate::GlyphPreparationError::Numeric {
        phase:crate::GlyphNumericPhase::Placement,value,
    }) if *value==1e100)
    );
    assert_released(&context);
}

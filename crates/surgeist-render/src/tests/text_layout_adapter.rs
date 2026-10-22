//! Functional qualification of the real standalone Text consumer.
//! Literal geometry comes from retained Ahem tables (UPEM1000, ascent800,
//! descent-200, A/X advance1000), not another adapter invocation.
use surgeist_text as text;

use crate::{
    Color, Error, OutlineCapability, OutlineLimits, OutlineNumericPhase, OutlinePreparationError,
    OutlineResource, OutlineSynthesis, PreparedOutline, Rect, TextGlyph, TextLayoutCause,
    TextLayoutError, TextLayoutLimits, TextLayoutLocation, TextLayoutResource, Transform,
    prepare_text_layout, scene::Command,
};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
const ROOM: usize = 8 << 20;

fn system(bytes: Vec<u8>) -> text::System {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}

fn style() -> text::Style {
    text::Style {
        font: text::Font::new().family("Ahem"),
        size: 20.0,
        line_height: text::LineHeight::Absolute(20.0),
        ..text::Style::default()
    }
}

fn options() -> text::Options {
    text::Options {
        quantize: false,
        ..text::Options::default()
    }
}

fn limits() -> TextLayoutLimits {
    bounded(32, 1024, ROOM, 32, 32, 32)
}

fn bounded(
    glyphs: usize,
    paths: usize,
    retained: usize,
    runs: usize,
    coordinates: usize,
    decorations: usize,
) -> TextLayoutLimits {
    TextLayoutLimits::new(
        OutlineLimits::new(glyphs, paths, ROOM, retained),
        runs,
        coordinates,
        decorations,
    )
}

fn layout(source: impl Into<text::Text>) -> text::Layout {
    system(AHEM.to_vec())
        .layout(text::Source::new(source), style(), options())
        .unwrap()
}

fn split_source() -> text::Source {
    let mut source = text::Source::new("AX");
    source.span(
        text::Range::new(1, 2),
        text::Style {
            brush: text::Brush::color(1.0, 0.0, 0.0, 1.0),
            ..style()
        },
    );
    source
}

fn outline(command: &Command) -> (&PreparedOutline, Transform) {
    let Command::PreparedOutline {
        outline, transform, ..
    } = command
    else {
        panic!("expected an actual prepared-outline command, got {command:?}");
    };
    (outline, *transform)
}

fn refuse(layout: &text::Layout, placement: Transform, limits: TextLayoutLimits) -> Error {
    match prepare_text_layout(layout, placement, limits) {
        Ok(_) => panic!("the whole invocation must refuse without a prepared scene"),
        Err(error) => error,
    }
}

fn diagnostic(error: &Error) -> &TextLayoutError {
    assert_eq!(error.code(), crate::ErrorCode::TextLayout);
    assert!(error.invalid_value_diagnostic().is_none());
    assert!(error.outline_preparation_diagnostic().is_none());
    error
        .text_layout_diagnostic()
        .expect("typed adapter refusal")
}

fn assert_limit(error: &Error, resource: TextLayoutResource, required: usize, limit: usize) {
    assert!(
        matches!(diagnostic(error), TextLayoutError::Limit {
        resource: actual_resource, required: actual_required, limit: actual_limit,
    } if *actual_resource == resource && *actual_required == required && *actual_limit == limit),
        "{error:?}"
    );
}

fn item_render(error: &Error, expected: TextLayoutLocation) -> &Error {
    let TextLayoutError::Item {
        location,
        cause: TextLayoutCause::Render(cause),
    } = diagnostic(error)
    else {
        panic!("expected preserved Render cause at {expected:?}: {error:?}");
    };
    assert_eq!(*location, expected);
    cause
}

#[test]
fn actual_registered_layout_preserves_face_instance_glyphs_ink_and_single_placement() {
    let mut system = system(AHEM.to_vec());
    let input = system
        .layout(
            text::Source::new("AX"),
            text::Style {
                brush: text::Brush::color(1.0, 0.0, 0.0, 0.5),
                ..style()
            },
            options(),
        )
        .unwrap();
    let before = system.stats();
    let placement = Transform::translation(7.0, 9.0).unwrap();
    let prepared = prepare_text_layout(&input, placement, limits()).unwrap();
    assert!(std::ptr::eq(prepared.layout(), &input));
    assert_eq!(
        system.stats(),
        before,
        "the consumer must not lay out again"
    );
    assert_eq!(prepared.scene().commands.len(), 1);
    let (run, transform) = outline(&prepared.scene().commands[0]);
    assert_eq!(transform, Transform::translation(7.0, 25.0).unwrap());
    assert_eq!(run.font().bytes(), AHEM);
    assert_eq!(run.font().index(), 0);
    assert_eq!(run.instance().size(), 20.0);
    assert!(run.instance().coordinates().is_empty());
    assert_eq!(run.instance().synthesis(), OutlineSynthesis::none());
    assert_eq!(
        run.glyphs(),
        [
            TextGlyph::try_new(35, 0.0, 0.0, 20.0).unwrap(),
            TextGlyph::try_new(58, 20.0, 0.0, 20.0).unwrap(),
        ]
    );
    assert_eq!(
        run.bounds().ink_rect(),
        Some(Rect::try_new(0.0, -16.0, 40.0, 20.0).unwrap())
    );
    let Command::PreparedOutline { paint, .. } = &prepared.scene().commands[0] else {
        unreachable!()
    };
    assert_eq!(
        paint.fill(),
        &Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap().into()
    );
}

#[test]
fn actual_fractional_logical_pens_preserve_provider_advances() {
    let spacing = 2.0_f64.powi(-30);
    let input = system(AHEM.to_vec())
        .layout(
            text::Source::new("AXA"),
            text::Style {
                letter_spacing: spacing,
                ..style()
            },
            options(),
        )
        .unwrap();
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let (run, transform) = outline(&prepared.scene().commands[0]);
    assert_eq!(transform, Transform::translation(0.0, 16.0).unwrap());
    assert_eq!(
        run.glyphs()
            .iter()
            .map(|glyph| glyph.x())
            .collect::<Vec<_>>(),
        [0.0, 20.0 + spacing, 40.0 + 2.0 * spacing]
    );
    assert!(
        run.glyphs()
            .iter()
            .all(|glyph| glyph.y() == 0.0 && glyph.advance() == 20.0)
    );
}

#[test]
fn actual_run_origin_is_outer_and_placement_order_preserves_decorations() {
    let input = system(AHEM.to_vec())
        .layout(
            text::Source::new("X"),
            decorated_style(),
            text::Options {
                indent: text::Indent {
                    amount: 5.0,
                    ..text::Indent::default()
                },
                ..options()
            },
        )
        .unwrap();
    let placement = Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap();
    let prepared = prepare_text_layout(&input, placement, limits()).unwrap();
    let (run, transform) = outline(&prepared.scene().commands[0]);
    assert_eq!(
        run.glyphs(),
        [TextGlyph::try_new(58, 0.0, 0.0, 20.0).unwrap()]
    );
    assert_eq!(
        run.bounds().ink_rect(),
        Some(Rect::new(0.0, -16.0, 20.0, 20.0))
    );
    // P*T applies the run origin before scaling. T*P would incorrectly give (12, 25).
    assert_eq!(transform.as_array(), [2.0, 0.0, 0.0, 3.0, 17.0, 57.0]);
    let Command::Layer { layer, children } = &prepared.scene().commands[1] else {
        panic!("paragraph decorations retain the single placement layer");
    };
    assert_eq!(layer.transform(), placement);
    let Command::Fill { shape, .. } = &children[0] else {
        panic!("underline fill")
    };
    assert_eq!(shape, &crate::Shape::rect(Rect::new(5.0, 18.0, 20.0, 1.0)));
}

#[test]
fn actual_large_run_origin_never_enters_canonical_outline_points() {
    let origin = 16_777_217.0_f64;
    let input = system(AHEM.to_vec())
        .layout(
            text::Source::new("X"),
            style(),
            text::Options {
                indent: text::Indent {
                    amount: origin,
                    ..text::Indent::default()
                },
                ..options()
            },
        )
        .unwrap();
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let (run, transform) = outline(&prepared.scene().commands[0]);
    assert_eq!(transform.as_array(), [1.0, 0.0, 0.0, 1.0, origin, 16.0]);
    assert_eq!(run.glyphs()[0].x(), 0.0);
    assert_eq!(
        run.bounds().ink_rect(),
        Some(Rect::new(0.0, -16.0, 20.0, 20.0))
    );
}

#[test]
fn actual_registered_fallback_runs_keep_each_face_and_outer_origin() {
    // Primary Ahem has only A in its authored cmap; Bhem is the unchanged
    // outline/metric face with its actual family renamed. X must select Bhem.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12];
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    cmap.extend(28_u32.to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(1_u32.to_be_bytes());
    for word in [0x41_u32, 0x41, 35] {
        cmap.extend(word.to_be_bytes());
    }
    let primary = replace_tables(vec![(*b"cmap", cmap)]);
    let mut name = table(AHEM, *b"name").to_vec();
    for index in 0..=name.len() - 8 {
        if name[index..index + 8] == [0, b'A', 0, b'h', 0, b'e', 0, b'm'] {
            name[index + 1] = b'B';
        }
    }
    for index in 0..=name.len() - 4 {
        if &name[index..index + 4] == b"Ahem" {
            name[index] = b'B';
        }
    }
    let fallback = replace_tables(vec![(*b"name", name)]);
    let mut system = system(primary.clone());
    system.register_fonts(fallback.clone()).unwrap();
    let input = system
        .layout(
            text::Source::new("AX"),
            text::Style {
                font: text::Font::new().family("Ahem").family("Bhem"),
                ..style()
            },
            options(),
        )
        .unwrap();
    let runs: Vec<_> = input.run_views().collect();
    assert_eq!(runs.len(), 2);
    assert!(!runs[0].face().is_same_face(&runs[1].face()));
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let (first, first_transform) = outline(&prepared.scene().commands[0]);
    let (second, second_transform) = outline(&prepared.scene().commands[1]);
    assert_eq!(first.font().bytes(), primary);
    assert_eq!(second.font().bytes(), fallback);
    assert_eq!(
        first.glyphs(),
        [TextGlyph::try_new(35, 0.0, 0.0, 20.0).unwrap()]
    );
    assert_eq!(
        second.glyphs(),
        [TextGlyph::try_new(58, 0.0, 0.0, 20.0).unwrap()]
    );
    assert_eq!(first_transform, Transform::translation(0.0, 16.0).unwrap());
    assert_eq!(
        second_transform,
        Transform::translation(20.0, 16.0).unwrap()
    );
}

#[test]
fn real_multiline_alignment_and_indent_are_preserved_by_run_placement() {
    for (alignment, indent, expected) in [
        (
            text::Alignment::Center,
            0.0,
            [(40.0, 16.0), (30.0, 36.0), (50.0, 36.0)],
        ),
        (
            text::Alignment::Start,
            10.0,
            [(10.0, 16.0), (0.0, 36.0), (20.0, 36.0)],
        ),
    ] {
        let input = system(AHEM.to_vec())
            .layout(
                text::Source::new("X\nXX"),
                style(),
                text::Options {
                    width: Some(100.0),
                    alignment,
                    indent: text::Indent {
                        amount: indent,
                        ..text::Indent::default()
                    },
                    ..options()
                },
            )
            .unwrap();
        assert_eq!(input.lines().len(), 2);
        let placement = Transform::translation(7.0, 9.0).unwrap();
        let prepared = prepare_text_layout(&input, placement, limits()).unwrap();
        let mut actual = Vec::new();
        for command in &prepared.scene().commands {
            let (run, transform) = outline(command);
            let [_, _, _, _, x, y] = transform.as_array();
            actual.extend(
                run.glyphs()
                    .iter()
                    .filter(|g| g.id() == 58)
                    .map(|g| (x - 7.0 + g.x(), y - 9.0 + g.y())),
            );
        }
        assert_eq!(actual, expected);
    }
}

#[test]
fn returned_borrow_retains_exact_utf16_layout_interactions() {
    let input = layout("A😀X");
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let same = prepared.layout();
    assert!(std::ptr::eq(same, &input));
    assert_eq!(
        same.source().text().code_units(),
        [0x41, 0xd83d, 0xde00, 0x58]
    );
    for (affinity, expected_x) in [
        (text::Affinity::Before, 20.0),
        (text::Affinity::After, 40.0),
    ] {
        let cursor = text::Cursor::new(2, affinity);
        assert_eq!(cursor.index(), 2);
        assert_eq!(same.cursor(cursor).unwrap().rect().origin.x, expected_x);
        assert!(
            same.selection(text::Selection::collapsed(cursor))
                .unwrap()
                .rects()
                .is_empty()
        );
    }
    for (start, end) in [(1, 2), (2, 3)] {
        let selection = text::Selection::new(
            text::Cursor::new(start, text::Affinity::After),
            text::Cursor::new(end, text::Affinity::Before),
        );
        assert_eq!(selection.range(), text::Range::new(start, end));
        let geometry = same.selection(selection).unwrap();
        assert_eq!(geometry.rects().len(), 1);
        assert_eq!(geometry.rects()[0].rect().origin.x, 20.0);
        assert_eq!(geometry.rects()[0].rect().size.width, 20.0);
    }
}

fn decorated_style() -> text::Style {
    text::Style {
        underline: text::Decoration::solid()
            .with_offset(text::DecorationOffset::try_absolute(-2.0).unwrap())
            .with_thickness(text::DecorationThickness::try_absolute(1.0).unwrap()),
        strikethrough: text::Decoration::solid()
            .with_offset(text::DecorationOffset::try_absolute(6.0).unwrap())
            .with_thickness(text::DecorationThickness::try_absolute(2.0).unwrap())
            .with_brush(text::DecorationBrush::Color(text::Brush::color(
                1.0, 0.0, 0.0, 1.0,
            ))),
        ..style()
    }
}

#[test]
fn actual_decorations_follow_glyphs_and_have_one_shared_placement_layer() {
    let input = system(AHEM.to_vec())
        .layout(text::Source::new("AX"), decorated_style(), options())
        .unwrap();
    let expected = [
        Rect::try_new(0.0, 18.0, 40.0, 1.0).unwrap(),
        // The offset is the center: baseline 16 - center 6 - half-thickness 1.
        Rect::try_new(0.0, 9.0, 40.0, 2.0).unwrap(),
    ];
    assert_eq!(input.decoration_views().count(), 2);
    for placement in [
        Transform::identity(),
        Transform::translation(7.0, 9.0).unwrap(),
    ] {
        let prepared = prepare_text_layout(&input, placement, limits()).unwrap();
        let commands = &prepared.scene().commands;
        let (_, transform) = outline(&commands[0]);
        assert_eq!(
            transform,
            Transform::translation(0.0, 16.0)
                .unwrap()
                .then(placement)
                .unwrap()
        );
        let decorations = if placement == Transform::identity() {
            assert_eq!(commands.len(), 3);
            &commands[1..]
        } else {
            assert_eq!(commands.len(), 2);
            let Command::Layer { layer, children } = &commands[1] else {
                panic!("nonidentity decoration placement must use one final layer");
            };
            assert_eq!(layer.transform(), placement);
            children.as_slice()
        };
        assert_eq!(decorations.len(), 2);
        for (index, (command, rect)) in decorations.iter().zip(expected).enumerate() {
            let Command::Fill { shape, paint } = command else {
                panic!("the actual decoration rectangle must remain a fill");
            };
            assert_eq!(shape, &crate::Shape::rect(rect));
            let color = if index == 0 {
                Color::BLACK
            } else {
                Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()
            };
            assert_eq!(paint, &color.into());
        }
    }
    let identity_charge = minimum_retained(&input, Transform::identity());
    let layer_charge = minimum_retained(&input, Transform::translation(7.0, 9.0).unwrap());
    assert!(layer_charge >= identity_charge + std::mem::size_of::<Command>());
}

#[test]
fn actual_zero_area_decoration_is_noop_but_positive_extent_cannot_disappear() {
    // Both glyphs have no outline. The actual U+200B advance is zero; the
    // actual space advance is20. Only the latter decoration has positive area.
    let input = |source| {
        system(AHEM.to_vec())
            .layout(text::Source::new(source), decorated_style(), options())
            .unwrap()
    };
    let zero = input("\u{200b}");
    let positive = input(" ");
    let zero_decorations: Vec<_> = zero.decoration_views().collect();
    assert_eq!(zero_decorations.len(), 2);
    assert!(zero_decorations.iter().all(|d| d.rect().size.width == 0.0));
    let positive_decorations: Vec<_> = positive.decoration_views().collect();
    assert_eq!(positive_decorations.len(), 2);
    assert!(
        positive_decorations
            .iter()
            .all(|d| d.rect().size.width == 20.0)
    );
    let placement = Transform::translation(1e30, 1e30).unwrap();
    assert!(prepare_text_layout(&zero, placement, limits()).is_ok());
    let error = refuse(&positive, placement, limits());
    let cause = item_render(&error, TextLayoutLocation::Decoration { decoration: 0 });
    assert!(matches!(
        cause.outline_preparation_diagnostic(),
        Some(OutlinePreparationError::Numeric {
            phase: OutlineNumericPhase::Placement,
            ..
        })
    ));
    assert!(prepare_text_layout(&positive, Transform::identity(), limits()).is_ok());
}

#[test]
fn aggregate_limits_cover_actual_multiple_runs_and_raised_retry() {
    let input = system(AHEM.to_vec())
        .layout(split_source(), style(), options())
        .unwrap();
    assert_eq!(input.run_views().count(), 2);
    let exact = bounded(2, 10, ROOM, 2, 0, 0);
    let prepared = prepare_text_layout(&input, Transform::identity(), exact).unwrap();
    assert_eq!(prepared.scene().commands.len(), 2);
    assert_eq!(
        prepared
            .scene()
            .commands
            .iter()
            .map(|c| outline(c).0.path_elements().count())
            .sum::<usize>(),
        10
    );
    let old_source = input.source().clone();
    for (limit, resource, required, supplied) in [
        (
            bounded(1, 10, ROOM, 2, 0, 0),
            TextLayoutResource::Glyphs,
            2,
            1,
        ),
        (
            bounded(2, 10, ROOM, 1, 0, 0),
            TextLayoutResource::Runs,
            2,
            1,
        ),
    ] {
        let error = refuse(&input, Transform::identity(), limit);
        assert_limit(&error, resource, required, supplied);
        assert_eq!(input.source().text(), old_source.text());
        assert!(prepare_text_layout(&input, Transform::identity(), exact).is_ok());
    }
    let error = refuse(&input, Transform::identity(), bounded(2, 9, ROOM, 2, 0, 0));
    let cause = item_render(&error, TextLayoutLocation::Run { line: 0, run: 1 });
    assert!(matches!(
        cause.outline_preparation_diagnostic(),
        Some(OutlinePreparationError::Limit {
            resource: OutlineResource::PathElements,
            required: 5,
            limit: 4,
        })
    ));
    let error = refuse(
        &input,
        Transform::identity(),
        TextLayoutLimits::new(OutlineLimits::new(2, 10, 0, ROOM), 2, 0, 0),
    );
    assert!(matches!(
        diagnostic(&error),
        TextLayoutError::Limit {
            resource: TextLayoutResource::ScratchBytes,
            ..
        }
    ));
    let decorated = system(AHEM.to_vec())
        .layout(text::Source::new("AX"), decorated_style(), options())
        .unwrap();
    let error = refuse(
        &decorated,
        Transform::identity(),
        bounded(2, 10, ROOM, 1, 0, 1),
    );
    assert_limit(&error, TextLayoutResource::Decorations, 2, 1);
    assert!(
        prepare_text_layout(
            &decorated,
            Transform::identity(),
            bounded(2, 10, ROOM, 1, 0, 2)
        )
        .is_ok()
    );
}

#[test]
fn actual_registered_static_composite_preserves_layout_run_and_literal_ink() {
    let input = system(composite_x_ahem(None))
        .layout(split_source(), style(), options())
        .unwrap();
    let runs: Vec<_> = input.run_views().collect();
    assert_eq!(runs[1].glyphs().map(|g| g.id()).collect::<Vec<_>>(), [58]);
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    assert!(std::ptr::eq(prepared.layout(), &input));
    let commands = &prepared.scene().commands;
    let (first, _) = outline(&commands[0]);
    let (composite, _) = outline(&commands[1]);
    assert_eq!(composite.glyphs()[0].id(), 58);
    assert_eq!(
        first.bounds().ink_rect(),
        Some(Rect::new(0.0, -16.0, 20.0, 20.0))
    );
    assert_eq!(
        composite.glyphs(),
        [TextGlyph::try_new(58, 0.0, 0.0, 20.0).unwrap()]
    );
    assert!(
        matches!(composite.path_elements().next(),Some(crate::PathElement::MoveTo(p)) if p.x()==0.0 && p.y()==-16.0)
    );
    assert_eq!(
        composite.bounds().ink_rect(),
        Some(Rect::new(0.0, -16.0, 20.0, 20.0))
    );
    assert_eq!(input.source().text().code_units(), [0x41, 0x58]);
    assert!(
        prepare_text_layout(&input, Transform::identity(), bounded(2, 9, ROOM, 2, 0, 0)).is_err()
    );
    let retry = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    assert_eq!(retry.scene().commands.len(), 2);
}

#[test]
fn actual_late_nonidentity_composite_refusal_exposes_no_batch_and_keeps_input_for_retry() {
    let input = system(composite_x_ahem(Some(8192)))
        .layout(split_source(), style(), options())
        .unwrap();
    let runs: Vec<_> = input.run_views().collect();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].glyphs().map(|g| g.id()).collect::<Vec<_>>(), [35]);
    assert_eq!(runs[1].glyphs().map(|g| g.id()).collect::<Vec<_>>(), [58]);
    let error = refuse(&input, Transform::identity(), limits());
    let cause = item_render(&error, TextLayoutLocation::Run { line: 0, run: 1 });
    assert_eq!(
        cause.outline_preparation_diagnostic(),
        Some(&OutlinePreparationError::UnsupportedOutline {
            capability: OutlineCapability::CompositeGlyph,
        })
    );
    assert_eq!(input.source().text().code_units(), [0x41, 0x58]);
    let good = system(AHEM.to_vec())
        .layout(split_source(), style(), options())
        .unwrap();
    assert!(prepare_text_layout(&good, Transform::identity(), limits()).is_ok());
}

#[test]
fn actual_italic_synthesis_is_executed_from_the_same_effective_instance() {
    let mut source = text::Source::new("AX");
    source.span(
        text::Range::new(1, 2),
        text::Style {
            font: text::Font::new().family("Ahem").style(text::Slant::Italic),
            ..style()
        },
    );
    let input = system(AHEM.to_vec())
        .layout(source, style(), options())
        .unwrap();
    let runs: Vec<_> = input.run_views().collect();
    assert_eq!(runs.len(), 2);
    assert!(runs[0].effective_instance().is_ok());
    let actual = runs[1].effective_instance().unwrap();
    assert!(actual.x_shear() > 0.0);
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let (italic, transform) = outline(&prepared.scene().commands[1]);
    assert_eq!(
        italic.instance().synthesis(),
        OutlineSynthesis::try_new(actual.x_shear(), 0.0).unwrap()
    );
    assert_eq!(transform, Transform::translation(20.0, 16.0).unwrap());
    // Literal Ahem top 800/bottom -200, scaled to size 20, sheared before y reflection.
    let left = f64::from((-4.0 * f64::from(actual.x_shear())) as f32);
    let right = f64::from((20.0 + 16.0 * f64::from(actual.x_shear())) as f32);
    assert_eq!(
        italic.bounds().ink_rect(),
        Some(Rect::new(left, -16.0, right - left, 20.0))
    );
    assert_eq!(input.source().text().code_units(), [0x41, 0x58]);
}

#[test]
fn actual_same_face_runs_copy_once_and_layout_owns_input_after_system_drop() {
    let input = {
        let mut bytes = Vec::with_capacity(AHEM.len() + ROOM);
        bytes.extend_from_slice(AHEM);
        let mut system = system(bytes);
        system.layout(split_source(), style(), options()).unwrap()
    };
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let first = outline(&prepared.scene().commands[0]).0;
    let second = outline(&prepared.scene().commands[1]).0;
    assert_eq!(
        first.font().bytes().as_ptr(),
        second.font().bytes().as_ptr()
    );
    assert_ne!(
        first.font().bytes().as_ptr(),
        input.run_views().next().unwrap().face().bytes().as_ptr()
    );
    assert_eq!(first.font().bytes(), AHEM);
    assert_eq!(second.font().bytes(), AHEM);
    let base = system(AHEM.to_vec())
        .layout(split_source(), style(), options())
        .unwrap();
    let base_charge = minimum_retained(&base, Transform::identity());
    assert_eq!(minimum_retained(&input, Transform::identity()), base_charge);
    const PADDING: usize = 257;
    let mut padded = AHEM.to_vec();
    padded.resize(padded.len() + PADDING, 0);
    let padded = system(padded)
        .layout(split_source(), style(), options())
        .unwrap();
    assert_eq!(
        minimum_retained(&padded, Transform::identity()),
        base_charge + PADDING
    );
    // One fewer retained byte refuses; a fresh full-budget invocation succeeds.
    refuse(
        &base,
        Transform::identity(),
        bounded(32, 1024, base_charge - 1, 32, 32, 32),
    );
    assert!(prepare_text_layout(&base, Transform::identity(), limits()).is_ok());
    let detached_scene = prepared.scene().clone();
    drop(prepared);
    drop(input);
    let (detached, _) = outline(&detached_scene.commands[1]);
    assert_eq!(detached.font().bytes(), AHEM);
    assert_eq!(detached.path_elements().count(), 5);
}

fn minimum_retained(input: &text::Layout, placement: Transform) -> usize {
    let (mut low, mut high) = (0, ROOM);
    assert!(prepare_text_layout(input, placement, bounded(32, 1024, high, 32, 32, 32)).is_ok());
    while low < high {
        let mid = low + (high - low) / 2;
        if prepare_text_layout(input, placement, bounded(32, 1024, mid, 32, 32, 32)).is_ok() {
            high = mid;
        } else {
            // This search characterizes an actual capacity threshold. Only
            // independently known padding/owner-size differences are oracles.
            let error = refuse(input, placement, bounded(32, 1024, mid, 32, 32, 32));
            let retained = matches!(
                diagnostic(&error),
                TextLayoutError::Limit {
                    resource: TextLayoutResource::RetainedBytes,
                    ..
                }
            ) || matches!(diagnostic(&error), TextLayoutError::Item { cause: TextLayoutCause::Render(cause), .. }
                    if matches!(cause.outline_preparation_diagnostic(), Some(OutlinePreparationError::Limit { resource: OutlineResource::RetainedBytes, .. })));
            assert!(
                retained,
                "threshold probe must refuse retained storage: {error:?}"
            );
            low = mid + 1;
        }
    }
    low
}

#[test]
fn empty_and_actual_space_do_not_acquire_fabricated_ink() {
    let empty = layout("");
    let prepared = prepare_text_layout(
        &empty,
        Transform::identity(),
        TextLayoutLimits::new(OutlineLimits::new(0, 0, ROOM, ROOM), 0, 0, 0),
    )
    .unwrap();
    assert!(prepared.scene().commands.is_empty());
    let decorated = system(AHEM.to_vec())
        .layout(text::Source::new(""), decorated_style(), options())
        .unwrap();
    assert!(decorated.run_views().next().is_some());
    assert!(
        decorated
            .run_views()
            .all(|run| run.glyphs().next().is_none())
    );
    let prepared = prepare_text_layout(
        &decorated,
        Transform::identity(),
        bounded(0, 0, ROOM, 0, 0, 2),
    )
    .unwrap();
    assert_eq!(prepared.scene().commands.len(), 2);
    for (command, expected) in prepared.scene().commands.iter().zip([
        Rect::new(0.0, 18.0, 0.0, 1.0),
        Rect::new(0.0, 9.0, 0.0, 2.0),
    ]) {
        let Command::Fill { shape, .. } = command else {
            panic!("metric-only rows retain decorations without a glyph paint command");
        };
        assert_eq!(shape, &crate::Shape::rect(expected));
    }
    let space = layout(" ");
    let prepared = prepare_text_layout(&space, Transform::identity(), limits()).unwrap();
    assert_eq!(prepared.scene().commands.len(), 1);
    let (run, _) = outline(&prepared.scene().commands[0]);
    assert_eq!(
        run.glyphs(),
        [TextGlyph::try_new(3, 0.0, 0.0, 20.0).unwrap()]
    );
    assert_eq!(run.bounds().kind(), crate::TextRunBoundsKind::Empty);
    assert_eq!(run.path_elements().count(), 0);
}

#[test]
fn genuine_variable_layout_keeps_full_default_and_nondefault_coordinate_bits() {
    for (setting, bits, width) in [(0.0, 0, 1000.0), (0.5, 8192, 1050.0), (1.0, 16384, 1100.0)] {
        let input = system(variable_ahem())
            .layout(
                text::Source::new("X"),
                text::Style {
                    size: 1000.0,
                    line_height: text::LineHeight::Absolute(1000.0),
                    font: text::Font::new()
                        .family("Ahem")
                        .variation(format!("'wght' {setting}")),
                    ..style()
                },
                options(),
            )
            .unwrap();
        let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
        let (run, _) = outline(&prepared.scene().commands[0]);
        assert_eq!(run.instance().coordinates().len(), 1);
        assert_eq!(run.instance().coordinates()[0].bits(), bits);
        assert_eq!(
            run.bounds().ink_rect(),
            Some(Rect::try_new(0.0, -800.0, width, 1000.0).unwrap())
        );
        if bits == 0 {
            let error = refuse(&input, Transform::identity(), bounded(1, 5, ROOM, 1, 0, 0));
            assert_limit(&error, TextLayoutResource::CoordinateValues, 1, 0);
            assert!(
                prepare_text_layout(&input, Transform::identity(), bounded(1, 5, ROOM, 1, 1, 0))
                    .is_ok()
            );
        }
    }
}

#[test]
fn adapter_scene_real_gpu_direct_and_base_graph_use_the_same_layout_resources() {
    use super::support::{default_graph_working_format_for_test, pixel_rgba};
    use crate::{Options, Parameters, RenderRoute, Renderer, Size};

    // At size 2, Ahem X covers local (0,-1.6)..(2,0.4), then its actual
    // baseline is composed through the run transform. The explicit underline
    // offset-0.4 puts the blue decoration at y2..3.
    // An actual one-unit indent yields two red rows and one blue row,
    // with x0/x3 and the final row left transparent on a 4x4 surface.
    let input = system(AHEM.to_vec())
        .layout(
            text::Source::new("X"),
            text::Style {
                size: 2.0,
                line_height: text::LineHeight::Absolute(2.0),
                brush: text::Brush::color(1.0, 0.0, 0.0, 1.0),
                underline: text::Decoration::solid()
                    .with_offset(text::DecorationOffset::try_absolute(-0.4).unwrap())
                    .with_thickness(text::DecorationThickness::try_absolute(1.0).unwrap())
                    .with_brush(text::DecorationBrush::Color(text::Brush::color(
                        0.0, 0.0, 1.0, 1.0,
                    ))),
                ..style()
            },
            text::Options {
                width: Some(4.0),
                indent: text::Indent {
                    amount: 1.0,
                    ..text::Indent::default()
                },
                ..options()
            },
        )
        .unwrap();
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    assert!(std::ptr::eq(prepared.layout(), &input));
    let (run, _) = outline(&prepared.scene().commands[0]);
    assert_eq!(run.font().bytes(), AHEM);
    assert_eq!(run.instance().size(), 2.0);
    assert_eq!(
        run.glyphs(),
        [TextGlyph::try_new(58, 0.0, 0.0, 2.0).unwrap()]
    );
    // Independently apply the published once-to-f32 path conversion to the
    // fixture's font-space top800/bottom-200, before the outer run baseline.
    // No bound or position returned by the adapter supplies this expectation.
    let scale = 2.0_f64 / 1000.0;
    let top = f64::from((-800.0 * scale) as f32);
    let bottom = f64::from((200.0 * scale) as f32);
    assert_eq!(
        run.bounds().ink_rect(),
        Some(Rect::try_new(0.0, top, 2.0, bottom - top).unwrap())
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let working_format = default_graph_working_format_for_test(&mut renderer);
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(4.0, 4.0), 1.0)).unwrap();
    for graph in [false, true] {
        let stats = if graph {
            pollster::block_on(renderer.render_forced_base_graph_for_test(
                &mut surface,
                prepared.scene(),
                Parameters::default(),
                working_format,
            ))
            .unwrap()
            .stats
        } else {
            pollster::block_on(renderer.render(
                &mut surface,
                prepared.scene(),
                Parameters::default(),
            ))
            .unwrap()
        };
        assert_eq!(
            stats.route,
            Some(if graph {
                RenderRoute::GpuGraph
            } else {
                RenderRoute::DirectVello
            })
        );
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for y in 0..4 {
            for x in 0..4 {
                let actual = pixel_rgba(&image, x, y);
                if (1..3).contains(&x) && y < 3 {
                    let expected: [u8; 4] = if y < 2 {
                        [255, 0, 0, 255]
                    } else {
                        [0, 0, 255, 255]
                    };
                    assert!(
                        actual
                            .iter()
                            .zip(expected)
                            .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
                        "graph={graph} pixel({x},{y})={actual:?}, expected {expected:?}"
                    );
                } else {
                    assert!(
                        actual[3] <= 2,
                        "graph={graph} pixel({x},{y}) must be transparent: {actual:?}"
                    );
                }
            }
        }
    }
}

// Authored Ahem derivatives reuse the existing prepared_outline.rs fixture
// construction. Their literal tables, not a provider output, define the oracle.
fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| 12 + i * 16)
        .find(|&record| bytes[record..record + 4] == tag)
        .unwrap();
    let offset = u32_at(bytes, record + 8) as usize;
    let len = u32_at(bytes, record + 12) as usize;
    &bytes[offset..offset + len]
}
fn replace_tables(replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let mut tables: Vec<_> = (0..usize::from(u16_at(AHEM, 4)))
        .map(|i| {
            let tag: [u8; 4] = AHEM[12 + i * 16..16 + i * 16].try_into().unwrap();
            (tag, table(AHEM, tag).to_vec())
        })
        .collect();
    for (tag, data) in replacements {
        if let Some(old) = tables.iter_mut().find(|old| old.0 == tag) {
            old.1 = data;
        } else {
            tables.push((tag, data));
        }
    }
    tables.sort_by_key(|entry| entry.0);
    let mut result = vec![0; 12 + tables.len() * 16];
    result[..4].copy_from_slice(&AHEM[..4]);
    result[4..6].copy_from_slice(&u16::try_from(tables.len()).unwrap().to_be_bytes());
    let power =
        tables.len().next_power_of_two() / if tables.len().is_power_of_two() { 1 } else { 2 };
    result[6..8].copy_from_slice(&u16::try_from(power * 16).unwrap().to_be_bytes());
    result[8..10].copy_from_slice(&u16::try_from(power.ilog2()).unwrap().to_be_bytes());
    result[10..12].copy_from_slice(
        &u16::try_from((tables.len() - power) * 16)
            .unwrap()
            .to_be_bytes(),
    );
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        while !result.len().is_multiple_of(4) {
            result.push(0);
        }
        let record = 12 + i * 16;
        let offset = u32::try_from(result.len()).unwrap();
        result[record..record + 4].copy_from_slice(&tag);
        result[record + 8..record + 12].copy_from_slice(&offset.to_be_bytes());
        result[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        result.extend(data);
    }
    result
}
fn composite_x_ahem(scale: Option<i16>) -> Vec<u8> {
    let mut composite = Vec::new();
    for word in [
        -1_i16,
        0,
        -200,
        1000,
        800,
        if scale.is_some() { 11 } else { 3 },
        35,
        0,
        0,
    ] {
        composite.extend(word.to_be_bytes());
    }
    if let Some(scale) = scale {
        composite.extend(scale.to_be_bytes());
    }
    let loca = table(AHEM, *b"loca");
    let glyf = table(AHEM, *b"glyf");
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let (mut new_loca, mut new_glyf) = (Vec::new(), Vec::new());
    for id in 0..count {
        new_loca.extend(u16::try_from(new_glyf.len() / 2).unwrap().to_be_bytes());
        if id == 58 {
            new_glyf.extend_from_slice(&composite);
        } else {
            new_glyf.extend_from_slice(
                &glyf[usize::from(u16_at(loca, id * 2)) * 2
                    ..usize::from(u16_at(loca, (id + 1) * 2)) * 2],
            );
        }
        if !new_glyf.len().is_multiple_of(2) {
            new_glyf.push(0);
        }
    }
    new_loca.extend(u16::try_from(new_glyf.len() / 2).unwrap().to_be_bytes());
    replace_tables(vec![(*b"loca", new_loca), (*b"glyf", new_glyf)])
}
fn variable_ahem() -> Vec<u8> {
    let mut fvar = Vec::new();
    fvar.extend(0x00010000_u32.to_be_bytes());
    for word in [16_u16, 2, 1, 20, 0, 8] {
        fvar.extend(word.to_be_bytes());
    }
    fvar.extend(*b"wght");
    for word in [0_i32, 0, 65536] {
        fvar.extend(word.to_be_bytes());
    }
    fvar.extend(0_u16.to_be_bytes());
    fvar.extend(256_u16.to_be_bytes());
    let tuple = [
        0, 1, 0, 10, 0, 11, 0xA0, 0, 0x40, 0, 0, 7, 0, 100, 100, 0, 0, 0, 0, 0, 0x87,
    ];
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let offset = u32::try_from(20 + (count + 1) * 4).unwrap();
    let mut gvar = Vec::new();
    gvar.extend(0x00010000_u32.to_be_bytes());
    for word in [1_u16, 0] {
        gvar.extend(word.to_be_bytes());
    }
    gvar.extend(offset.to_be_bytes());
    gvar.extend(u16::try_from(count).unwrap().to_be_bytes());
    gvar.extend(1_u16.to_be_bytes());
    gvar.extend(offset.to_be_bytes());
    for glyph in 0..=count {
        gvar.extend(
            if glyph <= 58 {
                0_u32
            } else {
                u32::try_from(tuple.len()).unwrap()
            }
            .to_be_bytes(),
        );
    }
    gvar.extend(tuple);
    replace_tables(vec![(*b"fvar", fvar), (*b"gvar", gvar)])
}

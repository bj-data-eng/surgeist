//! Display text and `source()` chains of the placed Text layout and leader pattern errors.
//!
//! The wrapped Text error prints its own text, which Text owns; these tests compare the wrapper's text
//! exactly and the cause only by identity with the wrapped value.
#![cfg(feature = "render-text")]

use core::error::Error as StdError;
use surgeist_render::{
    Error, text_layout::LeaderPatternCause, text_layout::LeaderPatternError,
    text_layout::TextLayoutCause, text_layout::TextLayoutError, text_layout::TextLayoutLocation,
    text_layout::TextLayoutResource,
};

fn text_error() -> surgeist_text::Error {
    surgeist_text::Error::new(
        surgeist_text::ErrorCode::NoUsableFontFace,
        "no face for the run",
    )
}

fn render_error() -> Error {
    Error::invalid_value("radius", -1, "must be non-negative")
}

#[test]
fn the_text_layout_errors_are_standard_errors() {
    fn assert_error<E: StdError + core::fmt::Debug + 'static>() {}
    assert_error::<TextLayoutError>();
    assert_error::<TextLayoutCause>();
    assert_error::<LeaderPatternError>();
    assert_error::<LeaderPatternCause>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_text_layout_errors_are_send_and_sync_off_wasm() {
    fn assert_send_sync<E: StdError + Send + Sync + 'static>() {}
    assert_send_sync::<TextLayoutError>();
    assert_send_sync::<TextLayoutCause>();
    assert_send_sync::<LeaderPatternError>();
    assert_send_sync::<LeaderPatternCause>();
}

#[test]
fn text_layout_limit_allocation_and_overflow_messages_name_the_resource() {
    use TextLayoutResource as R;
    for (resource, label) in [
        (R::Runs, "runs"),
        (R::CoordinateValues, "coordinate values"),
        (R::Decorations, "decorations"),
        (R::Glyphs, "glyphs"),
        (R::PathElements, "path elements"),
        (R::ScratchBytes, "scratch cpu bytes"),
        (R::RetainedBytes, "retained cpu bytes"),
    ] {
        let limit = TextLayoutError::Limit {
            resource,
            required: 9,
            limit: 4,
        };
        assert_eq!(
            limit.to_string(),
            format!("placed layout needs 9 {label}, above the limit of 4")
        );
        assert!(limit.source().is_none());
        let allocation = TextLayoutError::Allocation { resource };
        assert_eq!(allocation.to_string(), format!("allocating {label} failed"));
        assert!(allocation.source().is_none());
    }
    let overflow = TextLayoutError::CapacityOverflow;
    assert_eq!(overflow.to_string(), "a count calculation overflowed");
    assert!(overflow.source().is_none());
}

#[test]
fn a_layout_item_error_names_the_item_and_returns_its_cause() {
    let run = TextLayoutError::Item {
        location: TextLayoutLocation::Run { line: 1, run: 2 },
        cause: TextLayoutCause::Text(Box::new(text_error())),
    };
    assert_eq!(
        run.to_string(),
        "layout item at run 2 on line 1 was refused"
    );
    let cause = run.source().unwrap();
    assert_eq!(
        cause.to_string(),
        "text could not qualify the face, instance or synthesis"
    );
    assert_eq!(
        cause.source().unwrap().to_string(),
        text_error().to_string()
    );

    let decoration = TextLayoutError::Item {
        location: TextLayoutLocation::Decoration { decoration: 5 },
        cause: TextLayoutCause::Render(Box::new(render_error())),
    };
    assert_eq!(
        decoration.to_string(),
        "layout item at decoration 5 was refused"
    );
    let cause = decoration.source().unwrap();
    assert_eq!(cause.to_string(), "render could not prepare the item");
    assert_eq!(
        cause.source().unwrap().to_string(),
        render_error().to_string()
    );
}

#[test]
fn a_leader_run_error_names_the_run_and_returns_its_cause() {
    let text = LeaderPatternError::Run {
        run: 3,
        cause: LeaderPatternCause::Text(Box::new(text_error())),
    };
    assert_eq!(text.to_string(), "leader run 3 was refused");
    let cause = text.source().unwrap();
    assert_eq!(
        cause.to_string(),
        "text could not qualify the retained instance or glyph frame"
    );
    assert_eq!(
        cause.source().unwrap().to_string(),
        text_error().to_string()
    );

    let render = LeaderPatternError::Run {
        run: 0,
        cause: LeaderPatternCause::Render(Box::new(render_error())),
    };
    let cause = render.source().unwrap();
    assert_eq!(cause.to_string(), "render could not prepare the run");
    assert_eq!(
        cause.source().unwrap().to_string(),
        render_error().to_string()
    );
}

#[test]
fn a_new_variant_fails_to_compile_here_until_the_tests_above_cover_it() {
    // These matches have no wildcard. Each value below is also the subject of a test above.
    let layout = [
        TextLayoutError::Limit {
            resource: TextLayoutResource::Runs,
            required: 1,
            limit: 0,
        },
        TextLayoutError::CapacityOverflow,
        TextLayoutError::Allocation {
            resource: TextLayoutResource::Runs,
        },
        TextLayoutError::Item {
            location: TextLayoutLocation::Run { line: 0, run: 0 },
            cause: TextLayoutCause::Text(Box::new(text_error())),
        },
    ];
    let mut seen = [false; 4];
    for error in &layout {
        seen[match error {
            TextLayoutError::Limit { .. } => 0,
            TextLayoutError::CapacityOverflow => 1,
            TextLayoutError::Allocation { .. } => 2,
            TextLayoutError::Item { .. } => 3,
        }] = true;
        assert!(!error.to_string().is_empty());
    }
    assert!(seen.iter().all(|covered| *covered));
    let mut causes = [false; 2];
    for cause in [
        TextLayoutCause::Text(Box::new(text_error())),
        TextLayoutCause::Render(Box::new(render_error())),
    ] {
        causes[match cause {
            TextLayoutCause::Text(_) => 0,
            TextLayoutCause::Render(_) => 1,
        }] = true;
    }
    assert!(causes.iter().all(|covered| *covered));
    let mut leader = [false; 2];
    for cause in [
        LeaderPatternCause::Text(Box::new(text_error())),
        LeaderPatternCause::Render(Box::new(render_error())),
    ] {
        leader[match cause {
            LeaderPatternCause::Text(_) => 0,
            LeaderPatternCause::Render(_) => 1,
        }] = true;
    }
    assert!(leader.iter().all(|covered| *covered));
    let run = LeaderPatternError::Run {
        run: 0,
        cause: LeaderPatternCause::Text(Box::new(text_error())),
    };
    let LeaderPatternError::Run { .. } = run;
}

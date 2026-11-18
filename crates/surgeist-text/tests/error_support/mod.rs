//! Shared checks for the per-type message tables of the `error_text_*` tests.
//!
//! Each table row is written by hand from the documented meaning of one
//! variant. The checks here only compare a value against its row; they never
//! derive an expectation from the implementation.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Debug;

/// Checks one type's table.
///
/// * `rows` holds one `(value, expected text)` pair per variant, in declaration
///   order. `index` is an exhaustive `match` in the caller, so a new variant does
///   not compile until it has an index and therefore a row.
/// * Each text is exact, one lowercase clause without a trailing period or
///   braces, differs from the `Debug` form and does not contain the variant
///   identifier, and is distinct from every other row.
/// * `source()` is present exactly for the variants `wraps` names.
///
/// The bounds also prove at compile time that the type is a
/// `core::error::Error + Send + Sync + 'static`.
pub fn check_table<E>(
    variants: usize,
    rows: Vec<(E, &'static str)>,
    index: fn(&E) -> usize,
    wraps: fn(&E) -> bool,
) where
    E: Error + Send + Sync + 'static + Debug,
{
    assert_eq!(rows.len(), variants, "one row per variant");
    let mut seen = BTreeSet::new();
    for (position, (value, expected)) in rows.iter().enumerate() {
        assert_eq!(index(value), position, "row order for {value:?}");
        let text = value.to_string();
        assert_eq!(&text, expected, "message for {value:?}");
        assert!(!text.ends_with('.'), "no trailing period: {text}");
        assert!(
            text.chars().next().is_some_and(char::is_lowercase),
            "starts in lowercase: {text}"
        );
        assert!(
            !text.contains('{') && !text.contains('}'),
            "no braces: {text}"
        );
        let debug = format!("{value:?}");
        assert_ne!(text, debug, "message is the Debug form");
        let identifier: String = debug.chars().take_while(|c| c.is_alphanumeric()).collect();
        assert!(
            !text.contains(&identifier),
            "message repeats the identifier {identifier}: {text}"
        );
        assert!(seen.insert(text.clone()), "two rows share the text {text}");
        assert_eq!(
            value.source().is_some(),
            wraps(value),
            "source presence for {value:?}"
        );
    }
}

/// `source()` returns exactly the wrapped error, of the stated type, and the
/// wrapper's text neither repeats nor replaces it.
pub fn check_wrapper<E, S>(wrapper: &E, wrapped: &S)
where
    E: Error + 'static,
    S: Error + PartialEq + Debug + 'static,
{
    let source = wrapper.source().expect("a wrapper returns its payload");
    let source = source
        .downcast_ref::<S>()
        .expect("the source is the payload type");
    assert_eq!(source, wrapped);
    let outer = wrapper.to_string();
    let inner = wrapped.to_string();
    assert!(
        !outer.contains(&inner),
        "the wrapper repeats its source: {outer}"
    );
    assert_ne!(outer, inner, "the wrapper replaces its source");
}

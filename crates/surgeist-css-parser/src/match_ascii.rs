//! ASCII case-insensitive matching of a string against literal arms.

/// Matches a string against literal arms, ignoring ASCII case.
///
/// The first argument is the scrutinee: any expression that coerces to `&str`.
/// It is evaluated once. Each arm is one or more string literals separated by
/// `|`, an optional `if` guard, `=>`, and a body; the arms end with a `_`
/// fallback. Arms are tried in order, as in a `match`.
///
/// ```
/// use surgeist_css_parser::match_ignore_ascii_case;
///
/// fn side(ident: &str, horizontal: Option<u8>) -> u32 {
///     match_ignore_ascii_case! { ident,
///         "left" if horizontal.is_none() => 1,
///         "right" | "end" => 2,
///         _ => 0,
///     }
/// }
///
/// assert_eq!(side("LeFt", None), 1);
/// assert_eq!(side("left", Some(1)), 0);
/// assert_eq!(side("END", None), 2);
/// assert_eq!(side("sideways", None), 0);
/// ```
///
/// # Arm literals
///
/// Only the scrutinee is case-folded. Every arm literal must therefore be
/// written in lowercase ASCII: a literal containing an ASCII uppercase letter
/// is a compile-time error (an evaluation panic, E0080), because it could
/// never match.
///
/// ```compile_fail,E0080
/// use surgeist_css_parser::match_ignore_ascii_case;
///
/// let _ = match_ignore_ascii_case! { "left",
///     "Left" => 1,
///     _ => 0,
/// };
/// ```
///
/// A literal may contain non-ASCII characters; they are compared exactly,
/// since ASCII case folding leaves them alone.
///
/// # Cost
///
/// The scrutinee is lowercased into a stack buffer as long as the longest arm
/// literal; a longer scrutinee cannot match any arm and takes the fallback
/// without being examined. The macro never allocates.
///
/// # Hygiene
///
/// The expansion declares no named item (no `const`, `fn` or `static`), and
/// its local bindings are hygienic, so a guard or a body sees only the
/// caller's names, including caller items called `LONGEST` or `LOWERCASE`.
#[macro_export]
macro_rules! match_ignore_ascii_case {
    ( $input:expr,
      $( $( $literal:literal )|+ $( if $guard:expr )? => $body:expr, )*
      _ => $fallback:expr $(,)? ) => {{
        const {
            $( $(
                let bytes = $literal.as_bytes();
                let mut index = 0;
                while index < bytes.len() {
                    assert!(
                        !bytes[index].is_ascii_uppercase(),
                        "match_ignore_ascii_case! arm literals must be lowercase ASCII",
                    );
                    index += 1;
                }
            )+ )*
        }
        let input: &str = $input;
        // A plain block, not a `const` block: an inline constant sizing an
        // array inside a closure of a generic function is rejected as one that
        // may depend on the enclosing generics (`const_evaluatable_unchecked`).
        let mut buffer = [0u8; {
            let lengths: &[usize] = &[ $( $( $literal.len(), )+ )* ];
            let mut longest = 0;
            let mut index = 0;
            while index < lengths.len() {
                if lengths[index] > longest {
                    longest = lengths[index];
                }
                index += 1;
            }
            longest
        }];
        let lowered: ::core::option::Option<&str> = if input.len() <= buffer.len() {
            let lowered = &mut buffer[..input.len()];
            lowered.copy_from_slice(input.as_bytes());
            lowered.make_ascii_lowercase();
            ::core::str::from_utf8(lowered).ok()
        } else {
            ::core::option::Option::None
        };
        match lowered {
            $( ::core::option::Option::Some($( $literal )|+) $( if $guard )? => $body, )*
            _ => $fallback,
        }
    }};
}

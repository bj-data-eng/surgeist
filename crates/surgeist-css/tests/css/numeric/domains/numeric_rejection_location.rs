//! One responsible-token rule for every rejected numeric value (crate
//! reference, "Source coordinates and diagnostics"): the token that cannot
//! continue its production is responsible, including a closing `)`; when the
//! value ends first, the last authored token is; whitespace and comments are
//! never responsible. Each numeric type (number, integer, percentage, length,
//! length-percentage, angle, time, frequency, with their ranges), its wrappers,
//! its token dispatch sites and math roots must report the same token.
//!
//! Every case marks the expected responsible token with `[[...]]`; the marker,
//! not the parser, is the oracle. Each case also runs with extra whitespace and
//! a comment before the token, and, for a value cut short, after it: trivia must
//! move neither the position nor the token.

use surgeist_css::*;

const PAD: &str = "  /*c*/  ";

struct Case {
    source: String,
    offset: usize,
    token: String,
}

fn case(marked: &str, before: &str, after: &str) -> Case {
    let start = marked.find("[[").expect("opening marker");
    let end = marked.find("]]").expect("closing marker");
    let token = &marked[start + 2..end];
    let head = &marked[..start];
    let tail = &marked[end + 2..];
    let source = format!("{head}{before}{token}{tail}{after}");
    Case {
        source,
        offset: head.len() + before.len(),
        token: token.to_string(),
    }
}

#[track_caller]
fn assert_responsible(marked: &str) {
    let ends_value = marked.ends_with("]]");
    let mut variants = vec![case(marked, "", ""), case(marked, PAD, "")];
    if ends_value {
        variants.push(case(marked, "", PAD));
    }
    for Case {
        source,
        offset,
        token,
    } in variants
    {
        let report = CssParser::default().parse_block_contents(&source);
        let diagnostic = report
            .diagnostics()
            .first()
            .unwrap_or_else(|| panic!("a diagnostic: {source:?}"));
        let error = diagnostic.error();
        assert_eq!(
            error.code(),
            CssErrorCode::InvalidPropertyValue,
            "{source:?}: {error:?}"
        );
        assert_eq!(
            error.position().byte_offset().value(),
            offset,
            "{source:?}: responsible token `{token}`: {error:?}"
        );
        assert_eq!(
            error.encountered().map(CssTokenSummary::authored),
            Some(token.as_str()),
            "{source:?}: {error:?}"
        );
        assert!(
            report.syntax().as_slice().is_empty(),
            "{source:?} must be dropped"
        );
    }
}

#[test]
fn a_token_of_the_wrong_type_or_unit_is_responsible_in_every_numeric_type() {
    for marked in [
        // <length> and <length [0,∞]>, with wrappers.
        "outline-offset:[[1deg]]",
        "outline-offset:[[foo]]",
        "outline-offset:[[1]]",
        "letter-spacing:[[1s]]",
        "scroll-margin-top:[[foo]]",
        "scroll-margin:1px [[foo]]",
        "scroll-margin:1px 1px [[1]]",
        "padding-top:[[-1px]]",
        "padding:1px [[-1px]]",
        "border-spacing:1px [[foo]]",
        // <length-percentage>, with wrappers.
        "shape-margin:[[-1px]]",
        "margin:1px [[foo]]",
        "background-size:1px [[-1px]]",
        "background-position:1px [[1]]",
        "transform-origin:1px 1px [[1]]",
        "translate:1px 1px [[1]]",
        "box-shadow:1px 1px [[-1px]]",
        "text-shadow:1px 1px [[1]]",
        // <number>, <number [0,∞]> and <integer>.
        "flex-grow:[[1px]]",
        "flex:1 1 [[1]]",
        "opacity:[[1px]]",
        "scale:1 [[foo]]",
        "line-height:[[-1]]",
        "order:[[1.5]]",
        "orphans:[[-1]]",
        "z-index:[[1deg]]",
        "z-index:[[1.5]]",
        "column-count:[[0]]",
        "tab-size:1 [[foo]]",
        "text-combine-upright:digits [[5]]",
        "text-combine-upright:digits [[foo]]",
        "text-combine-upright:digits [[2px]]",
        // <percentage> and <percentage [0,∞]>.
        "voice-rate:[[-1%]]",
        "voice-rate:fast [[foo]]",
        "font-stretch:[[-1%]]",
        // <angle>.
        "rotate:[[1px]]",
        "rotate:45deg [[foo]]",
        "image-orientation:[[1]]",
        "offset-rotate:auto [[1]]",
        "glyph-orientation-vertical:[[45deg]]",
        "background-image:conic-gradient(from [[10]], red, blue)",
        "background-image:conic-gradient(red [[10px]], blue)",
        "background-image:conic-gradient(red 10deg [[20]], blue)",
        // <time>.
        "transition-duration:[[-1s]]",
        "transition-duration:1s, [[foo]]",
        "animation-delay:1s, [[1]]",
        "animation-duration:[[1]]",
        "pause-before:[[-1s]]",
        "interest-delay:1s [[1]]",
        // <frequency>.
        "voice-pitch:absolute [[-1Hz]]",
        "voice-pitch:absolute [[1px]]",
        // Token dispatch sites: track breadths, gradient stops, font settings.
        "grid-template-columns:1px [[foo]]",
        "grid-template-columns:1px [[-1px]]",
        "grid-template-columns:1px [[1]]",
        "grid-template-columns:repeat(2, [[-1px]])",
        "grid-template-columns:minmax(1px, [[1]])",
        "grid-template-columns:fit-content([[-1px]])",
        "grid-auto-rows:1px [[-1px]]",
        "grid-auto-columns:[[1]]",
        "background-image:linear-gradient(red, blue [[-1]])",
        "background-image:linear-gradient(red [[1]], blue)",
        "font-variation-settings:\"wght\" [[1px]]",
        "font-feature-settings:\"liga\" [[1.5]]",
        "counter-increment:a [[1.5]]",
        "hyphenate-limit-chars:1 1 [[1px]]",
        "aspect-ratio:1 / [[-1]]",
        "aspect-ratio:1 / [[1px]]",
        // Functions: the argument that cannot continue the production.
        "transform:translate(1px, [[foo]])",
        "transform:scale(1, [[1px]])",
        "transform:rotate([[1]])",
        "transform:rotate3d(1, 1, [[1px]], 1deg)",
        "transform:perspective([[-1px]])",
        "filter:blur([[-1px]])",
        "filter:drop-shadow(1px 1px [[-1px]])",
        "animation-timing-function:steps([[0]])",
        "animation-timing-function:cubic-bezier(0, 0, [[2]], 0)",
        "animation-timing-function:linear(0 50% [[1px]])",
        "clip-path:circle([[-1px]])",
        "clip-path:inset(1px round [[-1px]])",
        "offset-path:ray([[1px]])",
        "width:min(1px, [[foo]])",
        "width:clamp(1px, [[foo]], 2px)",
        "width:fit-content([[-1px]])",
    ] {
        assert_responsible(marked);
    }
}

#[test]
fn math_names_the_operand_that_cannot_continue_or_the_whole_function_that_does_not_type_check() {
    for marked in [
        // A closing `)` that cannot continue the production is responsible.
        "width:calc(1px +[[)]]",
        "width:calc([[)]]",
        "width:clamp(1px, 2px[[)]]",
        "line-height:calc(1 +[[)]]",
        "transform:matrix(1, 2, 3[[)]]",
        "clip:rect(1px, 2px, 3px[[)]]",
        "transform:rotate3d(1, 0, 0[[)]]",
        // A sum names its incompatible operand (Values 4 §10.9).
        "width:calc(1px + [[1s]])",
        "width:calc(1px + [[1]])",
        "width:calc(1 + [[1px]])",
        "line-height:calc(1px + [[1s]])",
        "transition-duration:calc(1px + [[1deg]])",
        "rotate:calc(1px + [[1s]])",
        // Another operand of the wrong shape.
        "width:calc(1px [[2px]])",
        "width:calc(1px + 1% + [[foo]])",
        "width:calc((1px + [[foo]]))",
        "width:calc(1px + calc([[foo]]))",
        "width:round(up, [[foo]])",
        // A grammatical calculation whose type does not check is responsible
        // as a whole.
        "width:[[calc(]]1px * 1px)",
        "width:[[calc(]]2 * 3)",
        "width:[[sin(]]1)",
        "opacity:[[calc(]]1px)",
        "z-index:[[calc(]]1px)",
        "transition-duration:[[calc(]]1px)",
        "rotate:[[calc(]]1s)",
    ] {
        assert_responsible(marked);
    }
}

#[test]
fn a_value_cut_short_names_its_last_authored_token_in_every_numeric_type() {
    for marked in [
        "aspect-ratio:1 [[/]]",
        "border-radius:1px [[/]]",
        "border-image:url(a) 1 [[/]]",
        "cursor:url(a.png) [[1]]",
        "box-shadow:[[1px]]",
        "text-shadow:[[1px]]",
        "voice-pitch:[[absolute]]",
        "rotate:[[x]]",
        "rotate:1 2 [[3]]",
        "transform:translate(1px[[,]]",
        "transform:rotate3d(1, 0, 0[[,]]",
        "transform:matrix(1[[,]]",
        "grid-template-columns:repeat(2[[,]]",
        "grid-template-columns:minmax(1px[[,]]",
        "animation-timing-function:cubic-bezier(0, 0[[,]]",
        "animation-timing-function:steps(1[[,]]",
        "clip-path:circle(1px [[at]]",
        "clip-path:inset(1px [[round]]",
        "clip-path:polygon(1px 1px[[,]]",
        "width:calc(1px [[+]]",
        "width:min(1px[[,]]",
        "width:clamp(1px, 2px[[,]]",
        "line-height:calc(1 [[+]]",
        "z-index:calc(1 [[+]]",
        "transition-duration:calc(1s [[+]]",
        // A function with nothing inside is itself the last authored token.
        "width:[[calc(]]",
        "width:[[min(]]",
        "opacity:[[calc(]]",
        "transform:[[rotate(]]",
        "transform:[[perspective(]]",
        "grid-template-columns:[[repeat(]]",
        "grid-template-columns:[[minmax(]]",
        "animation-timing-function:[[steps(]]",
        "text-combine-upright:digits [[calc(]]",
    ] {
        assert_responsible(marked);
    }
}

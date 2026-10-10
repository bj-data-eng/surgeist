#![forbid(unsafe_code)]

//! The public 256-level component limit must fit an ordinary 2 MiB stack.
//! Run through the repository's bounded process wrapper: an abort stays in the
//! child, and the wrapper's deadline covers its inherited process group.

#[path = "support/isolated_stack.rs"]
mod isolated_stack;

use surgeist_css::{
    CssComponentValueErrorKind, CssComponentValueRef, CssValueOrigin, parse_component_values,
};

fn isolated(test: &str, operation: fn()) {
    isolated_stack::run(
        test,
        "SURGEIST_CSS_COMPONENT_STACK_LIMIT_CHILD",
        "component stack assertions and resource destruction completed",
        Some("component-stack-contract"),
        operation,
    );
}

#[test]
fn depth_256_parses_serializes_and_drops_on_an_ordinary_stack() {
    isolated(
        "depth_256_parses_serializes_and_drops_on_an_ordinary_stack",
        || {
            let source = format!("{}x{}", "(".repeat(256), ")".repeat(256));
            let values = parse_component_values(&source).expect("256 levels are admitted");
            assert_eq!(values.component_count(), 257);
            assert_eq!(values.nesting_depth(), 256);
            let cloned = values.clone();
            assert!(values == cloned);
            let serialized = values.serialize().expect("serialize the admitted value");
            assert_eq!(serialized.as_css(), source);
            drop(serialized);
            drop(cloned);
            drop(values);
            drop(source);
        },
    );
}

#[test]
fn depth_257_reports_the_first_excess_opening_on_an_ordinary_stack() {
    isolated(
        "depth_257_reports_the_first_excess_opening_on_an_ordinary_stack",
        || {
            let source = format!("{}x{}", "(".repeat(257), ")".repeat(257));
            let error = parse_component_values(&source).expect_err("257 levels exceed the limit");
            assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
            let CssValueOrigin::Parsed(origin) = error.origin() else {
                panic!("the excess opening must retain its source origin");
            };
            assert_eq!(origin.source().as_str(), source);
            assert_eq!(origin.span().start().byte_offset().value(), 256);
            assert_eq!(origin.span().end().byte_offset().value(), 257);
            drop(error);
            drop(source);
        },
    );
}

#[test]
fn mixed_blocks_preserve_eof_closures_and_reject_mismatched_delimiters() {
    let source = "f([{x";
    let values = parse_component_values(source).expect("EOF closes each open block");
    assert_eq!(values.component_count(), 4);
    assert_eq!(values.nesting_depth(), 3);
    assert_eq!(values.serialize().unwrap().as_css(), "f([{x}])");
    let mut children = &values;
    for (start, end) in [(0, 2), (2, 3), (3, 4)] {
        let (closing, nested) = match children.items()[0].view() {
            CssComponentValueRef::Function(value) => (value.closing_origin(), value.values()),
            CssComponentValueRef::Block(value) => (value.closing_origin(), value.values()),
            _ => panic!("expected the next enclosing function or block"),
        };
        let CssValueOrigin::ImplicitClosure { opening, at } = closing else {
            panic!("missing authored delimiters must have implicit origins");
        };
        assert_eq!(opening.span().start().byte_offset().value(), start);
        assert_eq!(opening.span().end().byte_offset().value(), end);
        assert_eq!(at.span().start().byte_offset().value(), source.len());
        assert_eq!(at.span().end().byte_offset().value(), source.len());
        assert!(opening.source().same_snapshot(at.source()));
        children = nested;
    }

    let error = parse_component_values("f([x)").expect_err("a parenthesis cannot close a bracket");
    assert_eq!(
        error.kind(),
        CssComponentValueErrorKind::UnmatchedClosingDelimiter
    );
    let CssValueOrigin::Parsed(origin) = error.origin() else {
        panic!("the mismatched delimiter must retain its source origin");
    };
    assert_eq!(origin.span().start().byte_offset().value(), 4);
    assert_eq!(origin.span().end().byte_offset().value(), 5);
}

#[test]
fn mixed_depth_256_preserves_escaped_tokens_and_implicit_closures() {
    isolated(
        "mixed_depth_256_preserves_escaped_tokens_and_implicit_closures",
        || {
            let openings = "f([{(".repeat(64);
            let leaf = r#"\) "[)]" url(\)) /* }]) */"#;
            let source = format!("{openings}{leaf}");
            let values = parse_component_values(&source).expect("256 mixed levels are admitted");
            assert_eq!(values.nesting_depth(), 256);
            let expected = format!("{source}{}", ")}])".repeat(64));
            assert_eq!(values.serialize().unwrap().as_css(), expected);
            let mut children = &values;
            for _ in 0..256 {
                let (closing, nested) = match children.items()[0].view() {
                    CssComponentValueRef::Function(value) => {
                        (value.closing_origin(), value.values())
                    }
                    CssComponentValueRef::Block(value) => (value.closing_origin(), value.values()),
                    _ => panic!("expected a mixed enclosing component"),
                };
                let CssValueOrigin::ImplicitClosure { at, .. } = closing else {
                    panic!("all enclosing delimiters close at EOF");
                };
                assert_eq!(at.span().start().byte_offset().value(), source.len());
                assert_eq!(at.span().end().byte_offset().value(), source.len());
                children = nested;
            }
            assert_eq!(children.component_count(), 7);
            drop(values);
        },
    );
}

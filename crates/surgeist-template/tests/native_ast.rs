#![forbid(unsafe_code)]

use surgeist_template::{
    AstConstructionError, AttrValue, Attribute, ComponentRegistry, ElementName, Expr, IfBranch,
    Literal, NativeElementRegistry, NativeElementSpec, Node, SourcePos, SourceSpan, SpanError,
    TemplateDocument, ValidatedNode, ValidationErrorKind, VariableName, validate_template,
};

fn position(line: usize, column: usize, byte: usize) -> SourcePos {
    SourcePos::try_new(line, column, byte).unwrap()
}

fn span() -> SourceSpan {
    SourceSpan::try_new(position(2, 3, 4), position(3, 1, 8)).unwrap()
}

#[test]
fn native_positions_require_nonzero_coordinates_without_inventing_source_membership() {
    for (line, column) in [(0, 1), (1, 0), (0, 0)] {
        assert_eq!(
            SourcePos::try_new(line, column, 7),
            Err(SpanError::InvalidPosition { line, column })
        );
    }
    let supplied = position(7, 9, usize::MAX);
    assert_eq!(
        (supplied.line(), supplied.column(), supplied.byte()),
        (7, 9, usize::MAX)
    );
}

#[test]
fn native_spans_reject_reversed_ranges_and_inconsistent_zero_lengths() {
    let start = position(2, 5, 14);
    for end in [position(3, 1, 12), position(1, 9, 18), position(2, 4, 18)] {
        assert_eq!(
            SourceSpan::try_new(start, end),
            Err(SpanError::EndBeforeStart { start, end })
        );
    }
    for end in [position(2, 6, 14), position(3, 1, 14)] {
        assert_eq!(
            SourceSpan::try_new(start, end),
            Err(SpanError::InconsistentZeroLengthSpan { start, end })
        );
    }
    let empty = SourceSpan::try_new(start, start).unwrap();
    assert_eq!(empty.len_bytes(), 0);
    assert_eq!((empty.start(), empty.end()), (start, start));
    let ordered = span();
    assert_eq!(ordered.len_bytes(), 4);
    assert_eq!(
        (ordered.start(), ordered.end()),
        (position(2, 3, 4), position(3, 1, 8))
    );
}

#[test]
fn native_conditionals_require_initial_branch_and_retain_empty_bodies() {
    assert_eq!(
        Node::if_block(vec![], vec![Node::text("fallback", span())], span()),
        Err(AstConstructionError::EmptyIfBranches)
    );
    let conditional = Node::if_block(
        vec![IfBranch::new(Expr::Literal(Literal::Bool(false)), vec![])],
        vec![Node::text("fallback", span())],
        span(),
    )
    .unwrap();
    let Node::If(value) = &conditional else {
        panic!("conditional")
    };
    assert_eq!(value.branches().len(), 1);
    assert!(value.branches()[0].children().is_empty());
    assert_eq!(
        value.branches()[0].condition(),
        &Expr::Literal(Literal::Bool(false))
    );
    assert_eq!(value.else_children().len(), 1);
    assert_eq!(value.span(), span());
    let document = TemplateDocument::from_nodes(vec![conditional]);
    assert!(
        validate_template(
            &document,
            &NativeElementRegistry::default(),
            &ComponentRegistry::default()
        )
        .is_ok()
    );
}

#[test]
fn native_iteration_retains_symbolic_collection_binding_and_fallback() {
    let iteration = Node::foreach(
        Expr::Literal(Literal::Null),
        VariableName::try_new("item").unwrap(),
        vec![],
        vec![Node::text("empty", span())],
        span(),
    );
    let Node::ForEach(value) = iteration else {
        panic!("iteration")
    };
    assert_eq!(value.collection(), &Expr::Literal(Literal::Null));
    assert_eq!(value.item_name().as_str(), "item");
    assert!(value.children().is_empty());
    let [Node::Text(fallback)] = value.else_children() else {
        panic!("literal fallback")
    };
    assert_eq!(fallback.value(), "empty");
    assert_eq!(value.span(), span());
}

#[test]
fn native_float_literals_retain_every_bit_through_authored_and_validated_phases() {
    for bits in [
        0,
        0x8000_0000_0000_0000,
        0x3ff8_0000_0000_0000,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_0042,
        0xfff0_0000_0000_0001,
    ] {
        let document = TemplateDocument::from_nodes(vec![Node::interpolation(
            Expr::Literal(Literal::Float(f64::from_bits(bits))),
            span(),
        )]);
        let [Node::Interpolation(authored)] = document.nodes() else {
            panic!("authored interpolation")
        };
        let Expr::Literal(Literal::Float(value)) = authored.expr() else {
            panic!("authored float")
        };
        assert_eq!(value.to_bits(), bits);
        let validated = validate_template(
            &document,
            &NativeElementRegistry::default(),
            &ComponentRegistry::default(),
        )
        .unwrap();
        let [ValidatedNode::Interpolation(retained)] = validated.nodes() else {
            panic!("validated interpolation")
        };
        let Expr::Literal(Literal::Float(value)) = retained.expr() else {
            panic!("validated float")
        };
        assert_eq!(value.to_bits(), bits);
        assert_eq!(retained.span(), span());
    }
}

#[test]
fn native_construction_does_not_bypass_registry_validation() {
    let make = |attributes| {
        TemplateDocument::from_nodes(vec![Node::element(
            ElementName::native("div").unwrap(),
            attributes,
            vec![],
            span(),
        )])
    };
    let document = make(vec![]);
    let error = validate_template(
        &document,
        &NativeElementRegistry::default(),
        &ComponentRegistry::default(),
    )
    .unwrap_err();
    assert!(
        matches!(error.kind(), ValidationErrorKind::UnknownNativeElement { name } if name == "div")
    );
    assert_eq!(error.span(), span());
    let native = NativeElementRegistry::try_from_specs(vec![
        NativeElementSpec::try_new("div", vec![]).unwrap(),
    ])
    .unwrap();
    assert!(validate_template(&document, &native, &ComponentRegistry::default()).is_ok());
    let document = make(vec![
        Attribute::try_new("absent", AttrValue::Bool, span()).unwrap(),
    ]);
    assert!(
        matches!(validate_template(&document, &native, &ComponentRegistry::default()).unwrap_err().kind(), ValidationErrorKind::InvalidAttribute { attribute, .. } if attribute == "absent")
    );
    assert!(TemplateDocument::from_nodes(vec![]).nodes().is_empty());
}

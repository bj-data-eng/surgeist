use surgeist::template::{
    AttrPart, AttrValue, AttributeKind, AttributeRule, AttributeSpec, BinaryOp, ComponentRegistry,
    ComponentSpec, ElementName, Expr, Literal, NativeElementRegistry, NativeElementSpec, Node,
    PathSegment, SourceSpan, TemplateDocument, UnaryOp, parse_template, validate_template,
};

pub const SOURCE: &str = concat!(
    r#"<Panel disabled title="Hi {$user.name}" value={$match}>"#,
    "\nΩ \"\\\t\r\0\n",
    "{$user.items[2].name + 3 * -4}",
    r#"{if !$visible || false}<div class="raw\path">yes</div>{elseif null}nil{else}no{/if}"#,
    "{foreach $items as $item}{$item}{foreachelse}empty{/foreach}</Panel>",
);

pub fn registries() -> (NativeElementRegistry, ComponentRegistry) {
    let spec = |name, kind| AttributeSpec::try_new(name, AttributeRule::one(kind)).unwrap();
    (
        NativeElementRegistry::try_from_specs(vec![
            NativeElementSpec::try_new("div", vec![spec("class", AttributeKind::Static)]).unwrap(),
        ])
        .unwrap(),
        ComponentRegistry::try_from_specs(vec![
            ComponentSpec::try_new(
                "Panel",
                vec![
                    spec("disabled", AttributeKind::Bool),
                    spec("title", AttributeKind::Interpolated),
                    spec("value", AttributeKind::Expression),
                ],
            )
            .unwrap(),
        ])
        .unwrap(),
    )
}

fn variable(expr: &Expr, root: &str) {
    let Expr::Variable(path) = expr else {
        panic!("expected symbolic variable {root}: {expr:?}");
    };
    assert_eq!(path.root(), root);
    assert!(path.segments().is_empty());
}

fn text(node: &Node, expected: &str) {
    let Node::Text(actual) = node else {
        panic!("expected literal text {expected:?}: {node:?}");
    };
    assert_eq!(actual.value(), expected);
}

fn span(span: SourceSpan, start: (usize, usize, usize), end: (usize, usize, usize)) {
    let actual_start = span.start();
    let actual_end = span.end();
    assert_eq!(
        (
            actual_start.line(),
            actual_start.column(),
            actual_start.byte()
        ),
        start
    );
    assert_eq!(
        (actual_end.line(), actual_end.column(), actual_end.byte()),
        end
    );
}

pub fn verify(document: &TemplateDocument) {
    assert_eq!(document.nodes().len(), 1);
    let Node::Element(panel) = &document.nodes()[0] else {
        panic!("expected Panel element");
    };
    assert!(matches!(panel.name(), ElementName::Component(name) if name.as_str() == "Panel"));
    assert_eq!(
        panel
            .attributes()
            .iter()
            .map(|a| a.name())
            .collect::<Vec<_>>(),
        ["disabled", "title", "value"]
    );
    assert_eq!(panel.attributes()[0].value(), &AttrValue::Bool);
    let AttrValue::Interpolated(parts) = panel.attributes()[1].value() else {
        panic!("title remains symbolic interpolated content");
    };
    assert_eq!(parts.len(), 2);
    assert_eq!(&parts[0], &AttrPart::Text("Hi ".to_owned()));
    let AttrPart::Expr(Expr::Variable(path)) = &parts[1] else {
        panic!("title must retain user.name");
    };
    assert_eq!(path.root(), "user");
    assert!(matches!(path.segments(), [PathSegment::Field(name)] if name.as_str() == "name"));
    let AttrValue::Expression(value) = panel.attributes()[2].value() else {
        panic!("value remains symbolic expression content");
    };
    variable(value, "match");

    assert_eq!(panel.children().len(), 4);
    text(&panel.children()[0], "\nΩ \"\\\t\r\0\n");
    let Node::Text(initial_text) = &panel.children()[0] else {
        unreachable!()
    };
    span(initial_text.span(), (1, 56, 55), (3, 1, 65));
    let Node::Interpolation(interpolation) = &panel.children()[1] else {
        panic!("expected symbolic arithmetic interpolation");
    };
    let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right,
    } = interpolation.expr()
    else {
        panic!("expected addition");
    };
    let Expr::Variable(path) = &**left else {
        panic!("expected indexed user path")
    };
    assert_eq!(path.root(), "user");
    assert!(
        matches!(path.segments(), [PathSegment::Field(items), PathSegment::Index(index), PathSegment::Field(name)] if items.as_str() == "items" && index.get() == 2 && name.as_str() == "name")
    );
    let Expr::Binary {
        op: BinaryOp::Mul,
        left,
        right,
    } = &**right
    else {
        panic!("expected multiplication precedence")
    };
    assert_eq!(**left, Expr::Literal(Literal::Int(3)));
    assert_eq!(
        **right,
        Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::Literal(Literal::Int(4)))
        }
    );

    let Node::If(conditional) = &panel.children()[2] else {
        panic!("expected conditional")
    };
    assert_eq!(conditional.branches().len(), 2);
    let Expr::Binary {
        op: BinaryOp::Or,
        left,
        right,
    } = conditional.branches()[0].condition()
    else {
        panic!("expected symbolic disjunction")
    };
    let Expr::Unary {
        op: UnaryOp::Not,
        expr,
    } = &**left
    else {
        panic!("expected symbolic negation")
    };
    variable(expr, "visible");
    assert_eq!(**right, Expr::Literal(Literal::Bool(false)));
    assert_eq!(conditional.branches()[0].children().len(), 1);
    let Node::Element(div) = &conditional.branches()[0].children()[0] else {
        panic!("expected native div")
    };
    assert!(matches!(div.name(), ElementName::Native(name) if name.as_str() == "div"));
    assert_eq!(div.attributes().len(), 1);
    assert_eq!(div.attributes()[0].name(), "class");
    assert_eq!(
        div.attributes()[0].value(),
        &AttrValue::Static("raw\\path".to_owned())
    );
    assert_eq!(div.children().len(), 1);
    text(&div.children()[0], "yes");
    assert_eq!(
        conditional.branches()[1].condition(),
        &Expr::Literal(Literal::Null)
    );
    assert_eq!(conditional.branches()[1].children().len(), 1);
    text(&conditional.branches()[1].children()[0], "nil");
    assert_eq!(conditional.else_children().len(), 1);
    text(&conditional.else_children()[0], "no");

    let Node::ForEach(iteration) = &panel.children()[3] else {
        panic!("expected iteration")
    };
    variable(iteration.collection(), "items");
    assert_eq!(iteration.item_name().as_str(), "item");
    assert_eq!(iteration.children().len(), 1);
    let Node::Interpolation(item) = &iteration.children()[0] else {
        panic!("expected symbolic item")
    };
    variable(item.expr(), "item");
    assert_eq!(iteration.else_children().len(), 1);
    text(&iteration.else_children()[0], "empty");

    let (native, components) = registries();
    let revalidated = validate_template(document, &native, &components)
        .expect("generated authored AST revalidates");
    let parsed = parse_template(SOURCE).expect("independent strict source");
    let expected = validate_template(&parsed, &native, &components).unwrap();
    assert_eq!(
        revalidated, expected,
        "all symbolic values and spans survive output consumption"
    );
}

use surgeist_js_interner::Interner;

use crate::{
    Expression, Position, Span, Statement,
    expression::{
        Call, Identifier, NewTarget, TaggedTemplate, This,
        literal::{ArrayLiteral, ObjectLiteral, PropertyDefinition},
    },
    operations::{ContainsSymbol, contains, contains_invalid_object_literal},
    statement::With,
};

fn empty_span() -> Span {
    Span::new(Position::new(1, 1), Position::new(1, 1))
}

#[test]
fn invalid_object_walk_admits_before_the_match_and_stops_there() {
    let name = Interner::new().get_or_intern("a");
    let property = PropertyDefinition::CoverInitializedName(
        Identifier::new(name, empty_span()),
        This::new(empty_span()).into(),
    );
    let node: Expression = ObjectLiteral::new([property.clone(), property], empty_span()).into();
    for limit in [0, 2, 3] {
        let mut admitted = 0;
        let result = contains_invalid_object_literal(&node, || {
            if admitted == limit {
                return Err("refused");
            }
            admitted += 1;
            Ok(())
        });
        assert_eq!(admitted, limit);
        assert_eq!(result, if limit == 3 { Ok(true) } else { Err("refused") });
    }
}

#[test]
fn invalid_object_walk_charges_each_empty_array_slot() {
    let node = ArrayLiteral::new([None, None], false, empty_span());
    let mut admitted = 0;
    let result = contains_invalid_object_literal(&node, || {
        if admitted == 2 {
            return Err("refused");
        }
        admitted += 1;
        Ok(())
    });
    assert_eq!(result, Err("refused"));
    assert_eq!(admitted, 2);
}

#[test]
fn invalid_object_walk_charges_a_missing_cooked_template_segment() {
    let raw = Interner::new().get_or_intern("\\u");
    let node = TaggedTemplate::new(
        This::new(empty_span()).into(),
        Box::new([raw]),
        Box::new([None]),
        Box::default(),
        0,
        empty_span(),
    );
    let mut admitted = 0;
    let result = contains_invalid_object_literal(&node, || {
        if admitted == 4 {
            return Err("refused");
        }
        admitted += 1;
        Ok(())
    });
    assert_eq!(result, Err("refused"));
    assert_eq!(admitted, 4);
}

#[test]
fn check_contains_this_in_with_statement_expression() {
    let node = With::new(This::new(empty_span()).into(), Statement::Empty);
    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_with_statement_expression() {
    let node = With::new(NewTarget::new(empty_span()).into(), Statement::Empty);
    assert!(contains(&node, ContainsSymbol::NewTarget));
}

#[test]
fn check_contains_new_target_in_call_function_position() {
    let node = Call::new(
        NewTarget::new(empty_span()).into(),
        Box::default(),
        empty_span(),
    );
    assert!(contains(&node, ContainsSymbol::NewTarget));
}

#[test]
fn check_contains_this_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name = Identifier::new(interner.get_or_intern("func"), Span::new((1, 1), (1, 5)));
    let node = Call::new(
        function_name.into(),
        vec![This::new(empty_span()).into()].into_boxed_slice(),
        empty_span(),
    );

    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name = Identifier::new(interner.get_or_intern("func"), Span::new((1, 1), (1, 5)));
    let node = Call::new(
        function_name.into(),
        vec![NewTarget::new(empty_span()).into()].into_boxed_slice(),
        empty_span(),
    );

    assert!(contains(&node, ContainsSymbol::NewTarget));
}

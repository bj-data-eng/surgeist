//! Exact semantic compiler keys with producer-owned payload admission.

use super::{
    Literal,
    control::{CompilerControl, CompilerWork, primitive_error, string_error},
    index::CompilerKey,
};
use crate::{
    JsResult,
    value::primitive::{try_bigint_eq, try_bigint_hash},
};
use rustc_hash::FxHasher;
use std::{
    hash::{Hash, Hasher},
    mem::{Discriminant, size_of},
};
use surgeist_js_ast::{
    expression::Identifier,
    scope::{BindingLocator, BindingLocatorScope, IdentifierReference},
};
use surgeist_js_interner::Sym;

impl CompilerKey for Literal {
    fn try_hash(&self, control: &mut CompilerControl<'_>) -> JsResult<u64> {
        control.admit(CompilerWork::Read {
            bytes: size_of::<Discriminant<Self>>(),
        })?;
        let mut state = FxHasher::default();
        match self {
            Self::String(value) => {
                state.write_u8(0);
                value
                    .as_str()
                    .try_hash(&mut state, &mut |work| {
                        control.admit(CompilerWork::String(work))
                    })
                    .map_err(string_error)?;
            }
            Self::BigInt(value) => {
                state.write_u8(1);
                try_bigint_hash(value, &mut state, &mut |work| {
                    control.admit(CompilerWork::Primitive(work))
                })
                .map_err(primitive_error)?;
            }
        }
        Ok(state.finish())
    }

    fn try_eq(&self, other: &Self, control: &mut CompilerControl<'_>) -> JsResult<bool> {
        control.admit(CompilerWork::Read {
            bytes: 2 * size_of::<Discriminant<Self>>(),
        })?;
        match (self, other) {
            (Self::String(left), Self::String(right)) => left
                .as_str()
                .try_eq(right.as_str(), &mut |work| {
                    control.admit(CompilerWork::String(work))
                })
                .map_err(string_error),
            (Self::BigInt(left), Self::BigInt(right)) => try_bigint_eq(left, right, &mut |work| {
                control.admit(CompilerWork::Primitive(work))
            })
            .map_err(primitive_error),
            _ => Ok(false),
        }
    }
}

impl CompilerKey for Sym {
    fn try_hash(&self, control: &mut CompilerControl<'_>) -> JsResult<u64> {
        control.admit(CompilerWork::Read {
            bytes: size_of::<Self>(),
        })?;
        let mut state = FxHasher::default();
        self.hash(&mut state);
        Ok(state.finish())
    }

    fn try_eq(&self, other: &Self, control: &mut CompilerControl<'_>) -> JsResult<bool> {
        control.admit(CompilerWork::Read {
            bytes: 2 * size_of::<Self>(),
        })?;
        Ok(self == other)
    }
}

impl CompilerKey for Identifier {
    fn try_hash(&self, control: &mut CompilerControl<'_>) -> JsResult<u64> {
        control.admit(CompilerWork::Read {
            bytes: size_of::<Self>(),
        })?;
        let mut state = FxHasher::default();
        self.hash(&mut state);
        Ok(state.finish())
    }

    fn try_eq(&self, other: &Self, control: &mut CompilerControl<'_>) -> JsResult<bool> {
        control.admit(CompilerWork::Read {
            bytes: 2 * size_of::<Self>(),
        })?;
        Ok(self == other)
    }
}

fn hash_scope(scope: BindingLocatorScope, state: &mut FxHasher) {
    match scope {
        BindingLocatorScope::GlobalObject => state.write_u8(0),
        BindingLocatorScope::GlobalDeclarative => state.write_u8(1),
        BindingLocatorScope::Stack(index) => {
            state.write_u8(2);
            state.write_u32(index);
        }
    }
}

impl CompilerKey for BindingLocator {
    fn try_hash(&self, control: &mut CompilerControl<'_>) -> JsResult<u64> {
        let mut state = FxHasher::default();
        control.admit(CompilerWork::Read {
            bytes: size_of::<u32>(),
        })?;
        hash_scope(self.scope(), &mut state);
        control.admit(CompilerWork::Read {
            bytes: size_of::<u32>(),
        })?;
        state.write_u32(self.binding_index());
        control.admit(CompilerWork::Read {
            bytes: size_of::<u32>(),
        })?;
        state.write_u32(self.unique_scope_id());
        self.name()
            .as_str()
            .try_hash(&mut state, &mut |work| {
                control.admit(CompilerWork::String(work))
            })
            .map_err(string_error)?;
        Ok(state.finish())
    }

    fn try_eq(&self, other: &Self, control: &mut CompilerControl<'_>) -> JsResult<bool> {
        control.admit(CompilerWork::Read {
            bytes: 2 * size_of::<u32>(),
        })?;
        if self.scope() != other.scope() {
            return Ok(false);
        }
        control.admit(CompilerWork::Read {
            bytes: 2 * size_of::<u32>(),
        })?;
        if self.binding_index() != other.binding_index() {
            return Ok(false);
        }
        control.admit(CompilerWork::Read {
            bytes: 2 * size_of::<u32>(),
        })?;
        if self.unique_scope_id() != other.unique_scope_id() {
            return Ok(false);
        }
        self.name()
            .as_str()
            .try_eq(other.name().as_str(), &mut |work| {
                control.admit(CompilerWork::String(work))
            })
            .map_err(string_error)
    }
}

impl CompilerKey for IdentifierReference {
    fn try_hash(&self, control: &mut CompilerControl<'_>) -> JsResult<u64> {
        let mut state = FxHasher::default();
        control.admit(CompilerWork::Read {
            bytes: size_of::<bool>(),
        })?;
        self.is_lexical().hash(&mut state);
        control.admit(CompilerWork::Read {
            bytes: size_of::<bool>(),
        })?;
        self.escapes().hash(&mut state);
        control.admit(CompilerWork::Copy {
            bytes: size_of::<BindingLocator>(),
        })?;
        // locator() clones fixed metadata and an Rc handle, not name content.
        let locator = self.locator();
        state.write_u64(locator.try_hash(control)?);
        Ok(state.finish())
    }

    fn try_eq(&self, other: &Self, control: &mut CompilerControl<'_>) -> JsResult<bool> {
        control.admit(CompilerWork::Read {
            bytes: 2 * size_of::<bool>(),
        })?;
        if self.is_lexical() != other.is_lexical() {
            return Ok(false);
        }
        control.admit(CompilerWork::Read {
            bytes: 2 * size_of::<bool>(),
        })?;
        if self.escapes() != other.escapes() {
            return Ok(false);
        }
        control.admit(CompilerWork::Copy {
            bytes: 2 * size_of::<BindingLocator>(),
        })?;
        self.locator().try_eq(&other.locator(), control)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        JsBigInt, JsError, JsString,
        bytecompiler::control::{CompilerDepth, size_overflow},
        js_string,
    };
    use surgeist_js_ast::scope::Scope;

    fn hash<K: CompilerKey>(key: &K) -> u64 {
        let depth = CompilerDepth::new();
        key.try_hash(&mut CompilerControl::new(None, &depth))
            .unwrap()
    }

    fn equal<K: CompilerKey + PartialEq>(left: &K, right: &K, expected: bool) {
        let depth = CompilerDepth::new();
        let mut control = CompilerControl::new(None, &depth);
        assert_eq!(left.try_eq(right, &mut control).unwrap(), expected);
        assert_eq!(left == right, expected);
        if expected {
            assert_eq!(hash(left), hash(right));
        }
    }

    fn every_refusal(mut operation: impl FnMut(&mut CompilerControl<'_>) -> JsResult<()>) {
        let depth = CompilerDepth::new();
        let mut count = 0;
        let mut count_work = |_| {
            count += 1;
            Ok(())
        };
        operation(&mut CompilerControl::new(Some(&mut count_work), &depth)).unwrap();
        assert!(count > 1);
        let marker: JsError = size_overflow();
        for limit in 0..count {
            let mut accepted = 0;
            let mut refused = false;
            let mut admit = |_| {
                assert!(!refused, "key work continued after first refusal");
                if accepted == limit {
                    refused = true;
                    Err(marker.clone())
                } else {
                    accepted += 1;
                    Ok(())
                }
            };
            let error = operation(&mut CompilerControl::new(Some(&mut admit), &depth)).unwrap_err();
            assert_eq!(error, marker);
            assert_eq!(accepted, limit);
        }
    }

    #[test]
    fn literal_strings_preserve_encoding_independent_code_units_and_surrogates() {
        let latin = Literal::String(js_string!("caf\u{e9}"));
        let utf16 = Literal::String(JsString::from(&[99, 97, 102, 233][..]));
        equal(&latin, &utf16, true);
        let unpaired = Literal::String(JsString::from(&[0xd800, 65, 0xdc00][..]));
        let same = Literal::String(JsString::from(&[0xd800, 65, 0xdc00][..]));
        equal(&unpaired, &same, true);
        equal(
            &unpaired,
            &Literal::String(JsString::from(&[0xd800, 66, 0xdc00][..])),
            false,
        );
        equal(
            &Literal::String(js_string!("42")),
            &Literal::BigInt(JsBigInt::new(42)),
            false,
        );
    }

    #[test]
    fn bigint_literals_preserve_sign_and_canonical_magnitude_equality() {
        let literal = |value| Literal::BigInt(JsBigInt::from_string(value).unwrap());
        equal(&literal("42"), &literal("00042"), true);
        equal(&literal("0"), &literal("-0"), true);
        equal(&literal("42"), &literal("-42"), false);
        equal(
            &literal("18446744073709551617"),
            &literal("18446744073709551618"),
            false,
        );
        equal(
            &literal("18446744073709551617"),
            &literal("18446744073709551617"),
            true,
        );
    }

    #[test]
    fn long_literal_producers_preserve_the_first_late_refusal() {
        let string = Literal::String(JsString::from(&vec![0xd800; 1025][..]));
        every_refusal(|control| string.try_hash(control).map(|_| ()));
        every_refusal(|control| string.try_eq(&string, control).map(|_| ()));
        let magnitude = (crate::bigint::RawBigInt::from(1_u32) << 16384) + 7_u32;
        let bigint = Literal::BigInt(JsBigInt::new(magnitude));
        every_refusal(|control| bigint.try_hash(control).map(|_| ()));
        every_refusal(|control| bigint.try_eq(&bigint, control).map(|_| ()));
    }

    #[test]
    fn every_locator_field_participates_in_semantic_equality() {
        let root = Scope::new_global();
        let scope = Scope::new(root.clone(), false);
        let base = scope.create_mutable_binding(js_string!("key"), false);
        equal(&base, &base.clone(), true);
        let mut other_name = scope.create_mutable_binding(js_string!("other"), false);
        other_name.set_binding_index(base.binding_index());
        equal(&base, &other_name, false);
        let mut other_scope = base.clone();
        other_scope.set_scope(BindingLocatorScope::GlobalObject);
        equal(&base, &other_scope, false);
        let mut other_binding = base.clone();
        other_binding.set_binding_index(base.binding_index() + 1);
        equal(&base, &other_binding, false);
        let sibling = Scope::new(root, false);
        let other_identity = sibling.create_mutable_binding(js_string!("key"), false);
        assert_eq!(base.scope(), other_identity.scope());
        assert_eq!(base.binding_index(), other_identity.binding_index());
        assert_ne!(base.unique_scope_id(), other_identity.unique_scope_id());
        equal(&base, &other_identity, false);
        every_refusal(|control| base.try_hash(control).map(|_| ()));
        every_refusal(|control| base.try_eq(&base, control).map(|_| ()));
    }

    #[test]
    fn identifier_keys_preserve_lexical_and_escape_bits_independently() {
        let name = js_string!("key");
        let lexical_scope = Scope::new_global();
        let _ = lexical_scope.create_mutable_binding(name.clone(), false);
        let lexical = lexical_scope.get_identifier_reference(name.clone());
        let var_scope = Scope::new_global();
        let _ = var_scope.create_mutable_binding(name.clone(), true);
        let variable = var_scope.get_identifier_reference(name.clone());
        assert_eq!(lexical.locator(), variable.locator());
        assert!(!lexical.local() && !variable.local());
        assert_ne!(lexical.is_lexical(), variable.is_lexical());
        equal(&lexical, &variable, false);

        let scope = Scope::new(Scope::new_global(), true);
        let _ = scope.create_mutable_binding(name.clone(), false);
        let local = scope.get_identifier_reference(name.clone());
        scope.access_binding(&name, true);
        let escaped = scope.get_identifier_reference(name.clone());
        assert_eq!(local.locator(), escaped.locator());
        assert!(!local.escapes() && escaped.escapes());
        equal(&local, &escaped, false);
        equal(&escaped, &escaped.clone(), true);
        let global = Scope::new_global().get_identifier_reference(name);
        assert!(global.is_global_object() && global.escapes() && !global.local());
        equal(&global, &global.clone(), true);
        equal(&global, &escaped, false);
        every_refusal(|control| escaped.try_hash(control).map(|_| ()));
        every_refusal(|control| escaped.try_eq(&escaped, control).map(|_| ()));
    }

    #[test]
    fn symbols_use_exact_fixed_identity() {
        equal(&Sym::ARGUMENTS, &Sym::ARGUMENTS, true);
        equal(&Sym::ARGUMENTS, &Sym::EVAL, false);
    }

    #[test]
    fn identifier_keys_preserve_both_symbol_and_source_span() {
        use surgeist_js_ast::{Position, Span};
        let span = Span::new(Position::new(1, 1), Position::new(1, 4));
        let other_span = Span::new(Position::new(2, 1), Position::new(2, 4));
        let identifier = Identifier::new(Sym::ARGUMENTS, span);
        equal(&identifier, &Identifier::new(Sym::ARGUMENTS, span), true);
        equal(
            &identifier,
            &Identifier::new(Sym::ARGUMENTS, other_span),
            false,
        );
        equal(&identifier, &Identifier::new(Sym::EVAL, span), false);
    }
}

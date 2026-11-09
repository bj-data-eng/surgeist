//! Compiler-owned parameter-to-arguments metadata preparation.

use super::{
    ToJsString,
    control::{CompilerControl, CompilerWork, payload_bytes, size_overflow},
    index::CompilerIndex,
};
use crate::JsResult;
use surgeist_js_ast::{
    function::FormalParameterList,
    operations::try_bound_names,
    scope::{ScopeExtension, ScopeHandle},
};
use surgeist_js_interner::{Interner, Sym};
use thin_vec::ThinVec;

/// Maps only each name's last parameter occurrence to its environment binding.
/// Earlier occurrences remain ordinary, unmapped argument properties.
pub(super) fn binding_indices(
    formals: &FormalParameterList,
    interner: &Interner,
    scope: &ScopeHandle,
    extension: &mut ScopeExtension<'_>,
    control: &mut CompilerControl<'_>,
) -> JsResult<ThinVec<Option<u32>>> {
    let names = control.with_name_admission(|admission| try_bound_names(formals, admission))?;
    let length = names.len();
    u32::try_from(length).map_err(|_| size_overflow())?;
    let mut bindings = CompilerIndex::<Sym, (u32, usize)>::default();
    for (property_index, name) in names.into_iter().enumerate() {
        control.admit(CompilerWork::MetadataVisit)?;
        let string = name.try_to_js_string(interner, control)?;
        let binding_index = control.with_scope_admission(|admission| {
            extension.with_access(admission, |access| {
                Ok::<_, crate::JsError>(
                    access
                        .get_binding(scope, &string)?
                        .expect("parameter binding must exist")
                        .binding_index(),
                )
            })
        })?;
        // Every occurrence resolves against the same validated scope. Replacing
        // a duplicate retains that binding and moves only its argument property.
        bindings.insert(name, (binding_index, property_index), control)?;
    }

    if length == 0 {
        return Ok(ThinVec::new());
    }
    // ThinVec provides an infallible capacity API. This admits the requested
    // logical payload before that native allocation boundary; it does not
    // promise physical allocation measurements or complete OOM recovery.
    control.admit(CompilerWork::Capacity {
        bytes: payload_bytes::<Option<u32>>(length)?,
    })?;
    let mut indices = ThinVec::with_capacity(length);
    while indices.len() < length {
        let chunk = (length - indices.len()).min(256);
        control.admit(CompilerWork::Copy {
            bytes: payload_bytes::<Option<u32>>(chunk)?,
        })?;
        indices.resize(indices.len() + chunk, None);
    }
    bindings.visit_values(control, |(binding_index, property_index), control| {
        control.admit(CompilerWork::MetadataWrite)?;
        indices[*property_index] = Some(*binding_index);
        Ok(())
    })?;
    Ok(indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JsError, JsString, JsValue, bytecompiler::control::CompilerDepth};
    use surgeist_js_ast::{
        Span,
        declaration::Variable,
        expression::Identifier,
        function::FormalParameter,
        scope::{PreparedScopes, Scope, ScopeWork},
    };

    fn fixture(
        parameters: &[&str],
        exports: &[&str],
    ) -> (Interner, FormalParameterList, PreparedScopes, ScopeHandle) {
        let mut interner = Interner::default();
        let formals = FormalParameterList::from_parameters(
            parameters
                .iter()
                .map(|name| {
                    FormalParameter::new(
                        Variable::from_identifier(
                            Identifier::new(
                                interner.get_or_intern(*name),
                                Span::new((1, 1), (1, 2)),
                            ),
                            None,
                        ),
                        false,
                    )
                })
                .collect(),
        );
        let root = Scope::new_global();
        let mut scopes = PreparedScopes::new(&root);
        let names = exports
            .iter()
            .map(|name| JsString::from(*name))
            .collect::<Vec<_>>();
        let scope = scopes
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), JsError>>,
                |access| {
                    let parent = access.root_handle()?;
                    let (scope, _) = access.create_module_scope(&parent, &names)?;
                    Ok::<_, JsError>(scope)
                },
            )
            .expect("prepare parameter bindings");
        (interner, formals, scopes, scope)
    }

    #[test]
    fn distinct_parameters_map_in_order_and_duplicates_map_only_the_last_occurrence() {
        for (parameters, expected) in [
            (vec!["a", "b", "c"], vec![Some(1), Some(2), Some(3)]),
            (vec!["a", "a", "b", "a"], vec![None, None, Some(2), Some(1)]),
            (vec![], vec![]),
        ] {
            let (interner, formals, mut scopes, scope) =
                fixture(&parameters, &["arguments", "a", "b", "c"]);
            let mut extension = scopes
                .begin_extension::<JsError>(None)
                .expect("compiler savepoint");
            let depth = CompilerDepth::new();
            let actual = binding_indices(
                &formals,
                &interner,
                &scope,
                &mut extension,
                &mut CompilerControl::new(None, &depth),
            )
            .expect("mapped metadata");
            assert_eq!(actual.as_slice(), expected);
        }
    }

    #[test]
    fn bound_names_string_conversion_and_scope_work_reach_the_shared_control_once() {
        let (interner, formals, mut scopes, scope) =
            fixture(&["a", "b", "a"], &["arguments", "a", "b"]);
        let mut expected_names = Vec::new();
        try_bound_names(
            &formals,
            Some(&mut |work| {
                expected_names.push(work);
                Ok::<_, JsError>(())
            }),
        )
        .expect("AST producer sequence");
        let mut names = Vec::new();
        let mut strings = 0;
        let mut scope_work = 0;
        let mut parameters = 0;
        let mut scatter = 0;
        let mut admission = |work| {
            match work {
                CompilerWork::Name(work) => names.push(work),
                CompilerWork::String(_) => strings += 1,
                CompilerWork::Scope(_) => scope_work += 1,
                CompilerWork::MetadataVisit => parameters += 1,
                CompilerWork::MetadataWrite => scatter += 1,
                _ => {}
            }
            Ok(())
        };
        let depth = CompilerDepth::new();
        let mut extension = scopes.begin_extension::<JsError>(None).expect("savepoint");
        let actual = binding_indices(
            &formals,
            &interner,
            &scope,
            &mut extension,
            &mut CompilerControl::new(Some(&mut admission), &depth),
        )
        .expect("admitted metadata");
        assert_eq!(actual.as_slice(), [None, Some(2), Some(1)]);
        assert_eq!(names, expected_names);
        assert!(strings > 0);
        assert!(scope_work > 0);
        assert_eq!(parameters, 3);
        assert_eq!(scatter, 2);
    }

    #[test]
    fn exact_work_succeeds_and_every_one_short_refusal_preserves_the_original_error() {
        let (interner, formals, mut scopes, scope) =
            fixture(&["a", "b", "a"], &["arguments", "a", "b"]);
        let depth = CompilerDepth::new();
        let mut events = 0;
        let mut count = |_| {
            events += 1;
            Ok(())
        };
        {
            let mut extension = scopes.begin_extension::<JsError>(None).expect("savepoint");
            assert_eq!(
                binding_indices(
                    &formals,
                    &interner,
                    &scope,
                    &mut extension,
                    &mut CompilerControl::new(Some(&mut count), &depth)
                )
                .expect("calibration")
                .as_slice(),
                [None, Some(2), Some(1)]
            );
        }
        for limit in 0..=events {
            let marker = JsError::from_opaque(JsValue::new(313)).unwrap();
            let mut accepted = 0;
            let mut stopped = false;
            let mut admission = |_| {
                assert!(!stopped, "metadata work continued after refusal");
                if accepted == limit {
                    stopped = true;
                    Err(marker.clone())
                } else {
                    accepted += 1;
                    Ok(())
                }
            };
            let mut extension = scopes
                .begin_extension::<JsError>(None)
                .expect("retry savepoint");
            let result = binding_indices(
                &formals,
                &interner,
                &scope,
                &mut extension,
                &mut CompilerControl::new(Some(&mut admission), &depth),
            );
            if limit == events {
                assert_eq!(
                    result.expect("exact work").as_slice(),
                    [None, Some(2), Some(1)]
                );
            } else {
                assert_eq!(result.expect_err("one short at an actual request"), marker);
            }
            assert_eq!(accepted, limit);
        }
    }

    #[test]
    fn large_parameter_output_fill_can_refuse_between_bounded_chunks() {
        let parameters = vec!["a"; 513];
        let (interner, formals, mut scopes, scope) = fixture(&parameters, &["arguments", "a"]);
        let mut extension = scopes.begin_extension::<JsError>(None).expect("savepoint");
        let marker = JsError::from_opaque(JsValue::new(314)).unwrap();
        let chunk_bytes = payload_bytes::<Option<u32>>(256).expect("bounded chunk");
        let mut chunks = 0;
        let mut stopped = false;
        let mut admission = |work| {
            assert!(!stopped, "fill continued after refusal");
            if let CompilerWork::Copy { bytes } = work {
                assert!(bytes <= chunk_bytes);
                if bytes == chunk_bytes {
                    chunks += 1;
                    if chunks == 2 {
                        stopped = true;
                        return Err(marker.clone());
                    }
                }
            }
            Ok(())
        };
        let depth = CompilerDepth::new();
        let result = binding_indices(
            &formals,
            &interner,
            &scope,
            &mut extension,
            &mut CompilerControl::new(Some(&mut admission), &depth),
        );
        assert_eq!(result.expect_err("second fill chunk refused"), marker);
        assert_eq!(chunks, 2);
    }
}

//! This module implements the conversions from and into [`serde_json::Value`].

use super::JsValue;
use crate::ContextAccess;
use crate::{
    Context, JsResult, JsVariant,
    builtins::Array,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
};
use serde_json::{Map, Value};
use std::collections::HashSet;
use surgeist_js_gc::heap::ObjectId;

impl JsValue {
    /// Converts a [`serde_json::Value`] to a `JsValue`.
    ///
    /// # Example
    ///
    /// ```
    /// use surgeist_js::{Context, JsValue};
    ///
    /// let data = r#"
    ///     {
    ///         "name": "John Doe",
    ///         "age": 43,
    ///         "phones": [
    ///             "+44 1234567",
    ///             "+44 2345678"
    ///         ]
    ///      }"#;
    ///
    /// let json: serde_json::Value = serde_json::from_str(data).unwrap();
    ///
    /// let mut context = Context::default();
    /// let value = JsValue::from_json(&json, context.access()).unwrap();
    /// #
    /// # assert_eq!(Some(json), value.to_json(context.access()).unwrap());
    /// ```
    pub fn from_json(json: &Value, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        /// Biggest possible integer, as i64.
        const MAX_INT: i64 = i32::MAX as i64;

        /// Smallest possible integer, as i64.
        const MIN_INT: i64 = i32::MIN as i64;

        match json {
            Value::Null => Ok(Self::null()),
            Value::Bool(b) => Ok(Self::new(*b)),
            Value::Number(num) => num
                .as_i64()
                .filter(|n| (MIN_INT..=MAX_INT).contains(n))
                .map(|i| Self::new(i as i32))
                .or_else(|| num.as_f64().map(Self::new))
                .ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message(format!("could not convert JSON number {num} to JsValue"))
                        .into()
                }),
            Value::String(string) => Ok(Self::from(js_string!(string.as_str()))),
            Value::Array(vec) => {
                let mut arr = Vec::with_capacity(vec.len());
                for val in vec {
                    arr.push(Self::from_json(val, context.access())?);
                }
                Ok(Array::create_array_from_list(arr, context).into())
            }
            Value::Object(obj) => {
                let js_obj = JsObject::with_object_proto(context.intrinsics());
                for (key, value) in obj {
                    let property = PropertyDescriptor::builder()
                        .value(Self::from_json(value, context.access())?)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true);
                    js_obj
                        .borrow_mut()?
                        .insert(js_string!(key.clone()), property);
                }

                Ok(js_obj.into())
            }
        }
    }

    /// Converts the `JsValue` to a [`serde_json::Value`].
    ///
    /// If the `JsValue` is `Undefined`, this method will return `None`.
    /// Otherwise it will return the corresponding `serde_json::Value`.
    ///
    /// # Example
    ///
    /// ```
    /// use surgeist_js::{Context, JsValue};
    ///
    /// let data = r#"
    ///     {
    ///         "name": "John Doe",
    ///         "age": 43,
    ///         "phones": [
    ///             "+44 1234567",
    ///             "+44 2345678"
    ///         ]
    ///      }"#;
    ///
    /// let json: serde_json::Value = serde_json::from_str(data).unwrap();
    ///
    /// let mut context = Context::default();
    /// let value = JsValue::from_json(&json, context.access()).unwrap();
    ///
    /// let back_to_json = value.to_json(context.access()).unwrap();
    /// #
    /// # assert_eq!(Some(json), back_to_json);
    /// ```
    pub fn to_json(&self, context: ContextAccess<'_>) -> JsResult<Option<Value>> {
        let context = context.into_owner();
        let mut seen_objects = HashSet::new();
        self.to_json_inner(context, &mut seen_objects)
    }

    fn to_json_inner(
        &self,
        context: &mut Context,
        seen_objects: &mut HashSet<ObjectId>,
    ) -> JsResult<Option<Value>> {
        match self.variant() {
            JsVariant::Null => Ok(Some(Value::Null)),
            JsVariant::Undefined => Ok(None),
            JsVariant::Boolean(b) => Ok(Some(Value::from(b))),
            JsVariant::String(string) => Ok(Some(string.to_std_string_escaped().into())),
            JsVariant::Float64(rat) => Ok(Some(Value::from(rat))),
            JsVariant::Integer32(int) => Ok(Some(Value::from(int))),
            JsVariant::BigInt(_bigint) => Err(JsNativeError::typ()
                .with_message("cannot convert bigint to JSON")
                .into()),
            JsVariant::Object(obj) => {
                let id = obj.identity();
                if !seen_objects.insert(id) {
                    return Err(JsNativeError::typ()
                        .with_message("cyclic object value")
                        .into());
                }
                let mut value_by_prop_key = |property_key, context: &mut Context| {
                    obj.borrow()?
                        .properties()
                        .get(&property_key)
                        .and_then(|x| {
                            x.value()
                                .map(|val| val.to_json_inner(context, seen_objects))
                        })
                        .unwrap_or(Ok(Some(Value::Null)))
                };

                if obj.is_array() {
                    let len = obj.length_of_array_like(context)?;
                    let mut arr = Vec::with_capacity(len as usize);

                    for k in 0..len as u32 {
                        let val = value_by_prop_key(k.into(), context)?;
                        match val {
                            Some(val) => arr.push(val),

                            // Undefined in array. Substitute with null as Value doesn't support Undefined.
                            None => arr.push(Value::Null),
                        }
                    }
                    seen_objects.remove(&id);
                    Ok(Some(Value::Array(arr)))
                } else {
                    let mut map = Map::new();

                    for index in obj.borrow()?.properties().index_property_keys() {
                        let key = index.to_string();
                        let value = value_by_prop_key(index.into(), context)?;
                        if let Some(value) = value {
                            map.insert(key, value);
                        }
                    }

                    for property_key in obj.borrow()?.properties().shape().keys() {
                        let key = match &property_key {
                            PropertyKey::String(string) => string.to_std_string_escaped(),
                            PropertyKey::Index(i) => i.get().to_string(),
                            PropertyKey::Symbol(_sym) => {
                                return Err(JsNativeError::typ()
                                    .with_message("cannot convert Symbol to JSON")
                                    .into());
                            }
                        };
                        let value = value_by_prop_key(property_key, context)?;
                        if let Some(value) = value {
                            map.insert(key, value);
                        }
                    }
                    seen_objects.remove(&id);
                    Ok(Some(Value::Object(map)))
                }
            }
            JsVariant::Symbol(_sym) => Err(JsNativeError::typ()
                .with_message("cannot convert Symbol to JSON")
                .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use serde_json::json;
    use surgeist_js_macros::js_str;

    use crate::{
        Context, JsObject, JsValue, TestAction, js_string, object::JsArray, run_test_actions,
    };

    #[test]
    fn json_conversions() {
        const DATA: &str = indoc! {r#"
            {
                "name": "John Doe",
                "age": 43,
                "minor": false,
                "adult": true,
                "extra": {
                    "address": null
                },
                "phones": [
                    "+44 1234567",
                    -45,
                    {},
                    true
                ],
                "7.3": "random text",
                "100": 1000,
                "24": 42
            }
        "#};

        run_test_actions([TestAction::inspect_context(|ctx| {
            let json: serde_json::Value = serde_json::from_str(DATA).unwrap();
            assert!(json.is_object());

            let value = JsValue::from_json(&json, ctx.access()).unwrap();
            let obj = value.as_object().unwrap();
            assert_eq!(
                obj.get(js_str!("name"), ctx.access()).unwrap(),
                js_str!("John Doe").into()
            );
            assert_eq!(
                obj.get(js_str!("age"), ctx.access()).unwrap(),
                43_i32.into()
            );
            assert_eq!(
                obj.get(js_str!("minor"), ctx.access()).unwrap(),
                false.into()
            );
            assert_eq!(
                obj.get(js_str!("adult"), ctx.access()).unwrap(),
                true.into()
            );

            assert_eq!(
                obj.get(js_str!("7.3"), ctx.access()).unwrap(),
                js_string!("random text").into()
            );
            assert_eq!(obj.get(js_str!("100"), ctx.access()).unwrap(), 1000.into());
            assert_eq!(obj.get(js_str!("24"), ctx.access()).unwrap(), 42.into());

            {
                let extra = obj.get(js_str!("extra"), ctx.access()).unwrap();
                let extra = extra.as_object().unwrap();
                assert!(
                    extra
                        .get(js_str!("address"), ctx.access())
                        .unwrap()
                        .is_null()
                );
            }
            {
                let phones = obj.get(js_str!("phones"), ctx.access()).unwrap();
                let phones = phones.as_object().unwrap();

                let arr = JsArray::from_object(phones.clone()).unwrap();
                assert_eq!(
                    arr.at(0, ctx.access()).unwrap(),
                    js_str!("+44 1234567").into()
                );
                assert_eq!(arr.at(1, ctx.access()).unwrap(), JsValue::from(-45_i32));
                assert!(arr.at(2, ctx.access()).unwrap().is_object());
                assert_eq!(arr.at(3, ctx.access()).unwrap(), true.into());
            }

            assert_eq!(Some(json), value.to_json(ctx.access()).unwrap());
        })]);
    }

    #[test]
    fn integer_ops_to_json() {
        run_test_actions([
            TestAction::assert_with_op("1000000 + 500", |v, ctx| {
                v.to_json(ctx.access()).unwrap() == Some(json!(1_000_500))
            }),
            TestAction::assert_with_op("1000000 - 500", |v, ctx| {
                v.to_json(ctx.access()).unwrap() == Some(json!(999_500))
            }),
            TestAction::assert_with_op("1000000 * 500", |v, ctx| {
                v.to_json(ctx.access()).unwrap() == Some(json!(500_000_000))
            }),
            TestAction::assert_with_op("1000000 / 500", |v, ctx| {
                v.to_json(ctx.access()).unwrap() == Some(json!(2_000))
            }),
            TestAction::assert_with_op("233894 % 500", |v, ctx| {
                v.to_json(ctx.access()).unwrap() == Some(json!(394))
            }),
            TestAction::assert_with_op("36 ** 5", |v, ctx| {
                v.to_json(ctx.access()).unwrap() == Some(json!(60_466_176))
            }),
        ]);
    }

    #[test]
    fn to_json_cyclic() {
        let mut context = Context::default();
        let obj = JsObject::with_null_proto(context.runtime());
        obj.create_data_property(js_string!("a"), obj.clone(), context.access())
            .expect("should create data property");

        assert!(
            JsValue::from(obj)
                .to_json(context.access())
                .unwrap_err()
                .to_string()
                .starts_with("TypeError: cyclic object value"),
        );
    }

    #[test]
    fn to_json_undefined() {
        let mut context = Context::default();
        let undefined_value = JsValue::undefined();
        assert!(undefined_value.to_json(context.access()).unwrap().is_none());
    }

    #[test]
    fn to_json_undefined_in_structure() {
        let mut context = Context::default();
        let object_with_undefined = {
            // Defining the following structure:
            // {
            //     "outer_a": 1,
            //     "outer_b": undefined,
            //     "outer_c": [2, undefined, 3, { "inner_a": undefined }]
            // }

            let inner = JsObject::with_null_proto(context.runtime());
            inner
                .create_data_property(
                    js_string!("inner_a"),
                    JsValue::undefined(),
                    context.access(),
                )
                .expect("should add property");

            let array =
                JsArray::new(context.access()).expect("creating array in test must not fail");
            array.push(2, context.access()).expect("should push");
            array
                .push(JsValue::undefined(), context.access())
                .expect("should push");
            array.push(3, context.access()).expect("should push");
            array.push(inner, context.access()).expect("should push");

            let outer = JsObject::with_null_proto(context.runtime());
            outer
                .create_data_property(js_string!("outer_a"), JsValue::new(1), context.access())
                .expect("should add property");
            outer
                .create_data_property(
                    js_string!("outer_b"),
                    JsValue::undefined(),
                    context.access(),
                )
                .expect("should add property");
            outer
                .create_data_property(js_string!("outer_c"), array, context.access())
                .expect("should add property");

            JsValue::from(outer)
        };

        assert_eq!(
            Some(json!({
                "outer_a": 1,
                "outer_c": [2, null, 3, { }]
            })),
            object_with_undefined.to_json(context.access()).unwrap()
        );
    }
}

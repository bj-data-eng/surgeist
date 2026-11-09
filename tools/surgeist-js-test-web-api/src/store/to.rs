//! Reconstruct a JavaScript value from an owned graph.

use super::{NodeId, StoredGraph, StoredValue, StringStore, unsupported_type};
use rustc_hash::FxHashMap;
use surgeist_js::builtins::typed_array::TypedArrayKind;
use surgeist_js::object::builtins::{
    JsArray, JsArrayBuffer, JsDataView, JsDate, JsMap, JsRegExp, JsSet, JsSharedArrayBuffer,
    js_typed_array_from_kind,
};
use surgeist_js::{ContextAccess, JsBigInt, JsObject, JsResult, JsString, JsValue, js_error};

pub(super) fn decode(graph: &StoredGraph, mut context: ContextAccess<'_>) -> JsResult<JsValue> {
    Decoder {
        graph,
        seen: FxHashMap::default(),
    }
    .value(graph.root, context.reborrow())
}

struct Decoder<'a> {
    graph: &'a StoredGraph,
    seen: FxHashMap<NodeId, JsObject>,
}

impl Decoder<'_> {
    fn value(&mut self, id: NodeId, mut context: ContextAccess<'_>) -> JsResult<JsValue> {
        if let Some(object) = self.seen.get(&id) {
            return Ok(JsValue::from(object.clone()));
        }

        match &self.graph.nodes[id.0] {
            StoredValue::Null => Ok(JsValue::null()),
            StoredValue::Undefined => Ok(JsValue::undefined()),
            StoredValue::Boolean(value) => Ok(JsValue::from(*value)),
            StoredValue::Float(value) => Ok(JsValue::from(*value)),
            StoredValue::String(value) => Ok(JsValue::from(value.to_js_string())),
            StoredValue::BigInt(value) => Ok(JsValue::from(JsBigInt::new(value.clone()))),
            StoredValue::Object(fields) => self.object(id, fields, context.reborrow()),
            StoredValue::Map(entries) => {
                let map = JsMap::new(context.reborrow());
                self.seen.insert(id, map.clone().into());
                for &(key, value) in entries {
                    let key = self.value(key, context.reborrow())?;
                    let value = self.value(value, context.reborrow())?;
                    map.set(key, value, context.reborrow())?;
                }
                Ok(JsValue::from(map))
            }
            StoredValue::Set(entries) => {
                let set = JsSet::new(context.reborrow());
                self.seen.insert(id, set.clone().into());
                for &entry in entries {
                    let value = self.value(entry, context.reborrow())?;
                    set.add(value, context.reborrow())?;
                }
                Ok(JsValue::from(set))
            }
            StoredValue::Array(items) => self.array(id, items, context.reborrow()),
            StoredValue::Date(timestamp) => {
                let date = JsDate::new(context.reborrow());
                date.set_time(*timestamp, context.reborrow())?;
                self.seen.insert(id, date.clone().into());
                Ok(JsValue::from(date))
            }
            StoredValue::Error { .. } => Err(js_error!("Not yet implemented.")?),
            StoredValue::RegExp { source, flags } => {
                let regexp = JsRegExp::new(
                    JsString::from(source.as_str()),
                    JsString::from(flags.as_str()),
                    context.reborrow(),
                )?;
                self.seen.insert(id, regexp.clone().into());
                Ok(JsValue::from(regexp))
            }
            StoredValue::ArrayBuffer(data) => {
                let buffer = JsArrayBuffer::from_byte_block(data.clone(), context.reborrow())?;
                self.seen.insert(id, buffer.clone().into());
                Ok(JsValue::from(buffer))
            }
            StoredValue::SharedArrayBuffer(inner) => {
                let buffer = JsSharedArrayBuffer::from_buffer(inner.clone(), context.reborrow());
                self.seen.insert(id, buffer.clone().into());
                Ok(JsValue::from(buffer))
            }
            StoredValue::DataView {
                buffer,
                byte_length,
                byte_offset,
            } => {
                let buffer = self.array_buffer(*buffer, context.reborrow())?;
                let view = JsDataView::from_js_array_buffer(
                    buffer,
                    Some(*byte_offset),
                    Some(*byte_length),
                    context.reborrow(),
                )?;
                self.seen.insert(id, view.clone().into());
                Ok(JsValue::from(view))
            }
            StoredValue::TypedArray { kind, buffer } => {
                self.typed_array(id, *kind, *buffer, context.reborrow())
            }
        }
    }

    fn object(
        &mut self,
        id: NodeId,
        fields: &[(StringStore, NodeId)],
        mut context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let object = JsObject::with_object_proto(context.intrinsics());
        self.seen.insert(id, object.clone());

        for (key, id) in fields {
            let key = key.to_js_string();
            let value = self.value(*id, context.reborrow())?;
            object.set(key, value, true, context.reborrow())?;
        }
        Ok(JsValue::from(object))
    }

    fn array(
        &mut self,
        id: NodeId,
        items: &[Option<NodeId>],
        mut context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let array = JsArray::new(context.reborrow())?;
        self.seen.insert(id, array.clone().into());

        for (index, item) in items.iter().enumerate() {
            if let Some(id) = item {
                let value = self.value(*id, context.reborrow())?;
                array.set(index, value, true, context.reborrow())?;
            }
        }
        Ok(JsValue::from(array))
    }

    fn array_buffer(
        &mut self,
        id: NodeId,
        mut context: ContextAccess<'_>,
    ) -> JsResult<JsArrayBuffer> {
        let value = self.value(id, context.reborrow())?;
        JsArrayBuffer::from_object(value.as_object().ok_or_else(unsupported_type)?)
    }

    fn typed_array(
        &mut self,
        id: NodeId,
        kind: TypedArrayKind,
        buffer: NodeId,
        mut context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let buffer = self.array_buffer(buffer, context.reborrow())?;
        let array = js_typed_array_from_kind(kind, buffer, context.reborrow())?;
        if let Some(object) = array.as_object() {
            self.seen.insert(id, object);
        }
        Ok(array)
    }
}

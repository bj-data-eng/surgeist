//! Serialize a JavaScript value into an owned graph.

use super::{NodeId, StoredGraph, StoredValue, unsupported_type};
use rustc_hash::FxHashMap;
use surgeist_js::builtins::array_buffer::ArrayBuffer;
use surgeist_js::builtins::error::Error;
use surgeist_js::gc::heap::ObjectId;
use surgeist_js::object::builtins::{
    JsArray, JsArrayBuffer, JsDataView, JsDate, JsMap, JsRegExp, JsSet, JsSharedArrayBuffer,
    JsTypedArray,
};
use surgeist_js::property::PropertyKey;
use surgeist_js::{Context, JsError, JsObject, JsResult, JsString, JsValue, JsVariant, js_error};

/// Return true if an object is transferable.
pub(super) fn is_transferable(object: &JsObject) -> bool {
    // The only transferable object supported for now is ArrayBuffer.
    object.is::<ArrayBuffer>()
}

pub(super) fn encode(
    value: &JsValue,
    transfer: &FxHashMap<ObjectId, JsObject>,
    context: &mut Context,
) -> JsResult<StoredGraph> {
    let mut encoder = Encoder {
        transfer,
        seen: FxHashMap::default(),
        nodes: Vec::new(),
    };
    let root = encoder.value(value, context)?;
    let nodes = encoder
        .nodes
        .into_iter()
        .collect::<Option<_>>()
        .expect("successful traversal fills every reserved node");
    Ok(StoredGraph { root, nodes })
}

struct Encoder<'a> {
    transfer: &'a FxHashMap<ObjectId, JsObject>,
    // Keep visited objects rooted while JavaScript getters can run.
    seen: FxHashMap<ObjectId, (JsObject, NodeId)>,
    nodes: Vec<Option<StoredValue>>,
}

impl Encoder<'_> {
    fn push(&mut self, node: Option<StoredValue>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    fn value(&mut self, value: &JsValue, context: &mut Context) -> JsResult<NodeId> {
        let node = match value.variant() {
            JsVariant::Null => StoredValue::Null,
            JsVariant::Undefined => StoredValue::Undefined,
            JsVariant::Boolean(value) => StoredValue::Boolean(value),
            JsVariant::String(value) => StoredValue::String(value.into()),
            JsVariant::Float64(value) => StoredValue::Float(value),
            JsVariant::Integer32(value) => StoredValue::Float(f64::from(value)),
            JsVariant::BigInt(value) => StoredValue::BigInt(value.as_inner().clone()),
            JsVariant::Object(object) => return self.object(&object, context),
            JsVariant::Symbol(_) => return Err(unsupported_type()),
        };
        Ok(self.push(Some(node)))
    }

    fn object(&mut self, object: &JsObject, context: &mut Context) -> JsResult<NodeId> {
        let identity = object.identity();
        if let Some((_, id)) = self.seen.get(&identity) {
            return Ok(*id);
        }
        if self.transfer.contains_key(&identity) {
            let buffer =
                JsArrayBuffer::from_object(object.clone()).map_err(|_| unsupported_type())?;
            let bytes = buffer
                .detach_if_attached(&JsValue::undefined())?
                .ok_or_else(unsupported_type)?;
            return Ok(self.push(Some(StoredValue::ArrayBuffer(bytes))));
        }

        // Publish an ID before visiting children so back edges need no owning pointers.
        let id = self.push(None);
        self.seen.insert(identity, (object.clone(), id));
        self.nodes[id.0] = Some(self.object_contents(object, context)?);
        Ok(id)
    }

    fn object_contents(
        &mut self,
        object: &JsObject,
        context: &mut Context,
    ) -> JsResult<StoredValue> {
        if let Ok(array) = JsArray::from_object(object.clone()) {
            return self.array(&array, context);
        } else if let Ok(map) = JsMap::from_object(object.clone()) {
            let mut entries = Vec::new();
            map.for_each_native(|key, value| {
                entries.push((self.value(&key, context)?, self.value(&value, context)?));
                Ok(())
            })?;
            return Ok(StoredValue::Map(entries));
        } else if let Ok(set) = JsSet::from_object(object.clone()) {
            let mut entries = Vec::new();
            set.for_each_native(|value| {
                entries.push(self.value(&value, context)?);
                Ok(())
            })?;
            return Ok(StoredValue::Set(entries));
        } else if let Ok(buffer) = JsArrayBuffer::from_object(object.clone()) {
            let data = buffer.data().ok_or_else(unsupported_type)?;
            return Ok(StoredValue::ArrayBuffer(data.to_vec()));
        } else if let Ok(buffer) = JsSharedArrayBuffer::from_object(object.clone()) {
            return Ok(StoredValue::SharedArrayBuffer(buffer.inner()));
        } else if let Ok(array) = JsTypedArray::from_object(object.clone()) {
            let kind = array.kind().ok_or_else(unsupported_type)?;
            let buffer = array.buffer(context)?;
            return Ok(StoredValue::TypedArray {
                kind,
                buffer: self.value(&buffer, context)?,
            });
        } else if let Ok(date) = JsDate::from_object(object.clone()) {
            let timestamp = date
                .get_time(context)?
                .as_number()
                .ok_or_else(unsupported_type)?;
            return Ok(StoredValue::Date(timestamp));
        } else if object.clone().downcast::<Error>().is_ok() {
            return Err(js_error!(TypeError: "Errors are not supported yet."));
        } else if let Ok(regexp) = JsRegExp::from_object(object.clone()) {
            return Ok(StoredValue::RegExp {
                source: regexp.source(context)?,
                flags: regexp.flags(context)?,
            });
        } else if JsDataView::from_object(object.clone()).is_ok() {
            return Err(js_error!(TypeError: "Data views are not supported yet."));
        } else if object.is_callable() {
            return Err(unsupported_type());
        }

        // Keep own properties. Reconstruction supplies the destination realm's prototype.
        let keys = object.own_property_keys(context)?;
        let mut fields = Vec::with_capacity(keys.len());
        for key in keys {
            let value = object.get(key.clone(), context)?;
            let key = match key {
                PropertyKey::String(string) => string.into(),
                PropertyKey::Symbol(_) => return Err(unsupported_type()),
                PropertyKey::Index(index) => JsString::from(format!("{}", index.get())).into(),
            };
            fields.push((key, self.value(&value, context)?));
        }
        Ok(StoredValue::Object(fields))
    }

    fn array(&mut self, array: &JsArray, context: &mut Context) -> JsResult<StoredValue> {
        let length = usize::try_from(array.length(context)?).map_err(JsError::from_rust)?;
        let mut items = Vec::with_capacity(length);
        for index in 0..length {
            let value = array
                .borrow()
                .properties()
                .get(&index.into())
                .and_then(|descriptor| descriptor.value().cloned());
            let item = value.map(|value| self.value(&value, context)).transpose()?;
            items.push(item);
        }
        Ok(StoredValue::Array(items))
    }
}

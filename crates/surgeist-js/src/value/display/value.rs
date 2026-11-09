use std::collections::HashSet;

use crate::{
    JsError, JsString, JsValue, JsVariant,
    builtins::{
        Array, Promise,
        error::Error,
        function::{
            OrdinaryFunction,
            arguments::{MappedArguments, UnmappedArguments},
        },
        map::ordered_map::OrderedMap,
        promise::PromiseState,
        set::ordered_set::OrderedSet,
        typed_array::TypedArray,
        weak_map::NativeWeakMap,
        weak_set::NativeWeakSet,
    },
    js_string,
    property::{PropertyDescriptor, PropertyKey},
};

/// Maximum nesting depth before objects/arrays are collapsed
pub(super) const COMPACT_DEPTH_LIMIT: u32 = 2;

pub(super) fn log_value_to(
    f: &mut super::RenderSink<'_>,
    x: &JsValue,
    print_internals: bool,
    print_children: bool,
) -> crate::JsResult<()> {
    match x.variant() {
        // We don't want to print private (compiler) or prototype properties
        JsVariant::Object(v) => {
            // Can use the private "type" field of an Object to match on
            // which type of Object it represents for special printing
            if let Some(s) = v.downcast_ref::<JsString>()? {
                write!(f, "String {{ {:?} }}", s.to_std_string_escaped())
            } else if let Some(b) = v.downcast_ref::<bool>()? {
                write!(f, "Boolean {{ {b} }}")
            } else if let Some(r) = v.downcast_ref::<f64>()? {
                f.write_str("Number { ")?;
                super::primitives::format_rational(*r, f)?;
                f.write_str(" }")
            } else if v.is::<Array>() {
                super::array::log_array_to(f, &v, print_internals, print_children)
            } else if v.is::<UnmappedArguments>() || v.is::<MappedArguments>() {
                super::arguments::log_arguments_to(f, &v, print_internals, print_children)
            } else if v.is::<OrderedMap>() {
                super::map::log_map_to(f, &v, print_internals, print_children)
            } else if v.is::<OrderedSet>() {
                super::set::log_set_to(f, &v, print_internals, print_children)
            } else if v.is::<NativeWeakMap>() {
                f.write_str("WeakMap { <items unknown> }")
            } else if v.is::<NativeWeakSet>() {
                f.write_str("WeakSet { <items unknown> }")
            } else if v.is::<Error>() {
                let name = v
                    .get_property(&js_string!("name").into())?
                    .as_ref()
                    .and_then(PropertyDescriptor::value)
                    .map(error_property_text)
                    .transpose()?
                    .unwrap_or_else(|| "<error>".into());
                let message = v
                    .get_property(&js_string!("message").into())?
                    .as_ref()
                    .and_then(PropertyDescriptor::value)
                    .map(error_property_text)
                    .transpose()?
                    .unwrap_or_default();
                if name.is_empty() {
                    f.write_str(&message)?;
                } else if message.is_empty() {
                    f.write_str(&name)?;
                } else {
                    write!(f, "{name}: {message}")?;
                }
                let data = v
                    .downcast_ref::<Error>()?
                    .ok_or(crate::error::ObjectAccessError::WrongNativeBrand)?;

                if let Some(entry) = data.stack.0.position() {
                    write!(f, "{}", entry.display(false))?;
                }
                Ok(())
            } else if let Some(promise) = v.downcast_ref::<Promise>()? {
                f.write_str("Promise { ")?;
                match promise.state() {
                    PromiseState::Pending => f.write_str("<pending>")?,
                    PromiseState::Fulfilled(val) => val.display().render_to(f.0)?,
                    PromiseState::Rejected(reason) => {
                        f.write_str("<rejected> ")?;
                        JsError::from_opaque(reason.clone())?.render_to(f.0)?;
                    }
                }
                f.write_str(" }")
            } else if v.is::<TypedArray>() {
                super::typed_array::log_typed_array(f, &v, print_children, print_internals)
            } else if let Some(date) = v.downcast_ref::<crate::builtins::date::Date>()? {
                match date.to_iso_display() {
                    Some(iso) => f.write_str(&iso),
                    None => f.write_str("Invalid Date"),
                }
            } else if let Some(regexp) = v.downcast_ref::<crate::builtins::regexp::RegExp>()? {
                let source =
                    escape_regexp_source(&regexp.original_source().to_std_string_escaped());
                let flags = regexp.original_flags().to_std_string_escaped();
                write!(f, "/{source}/{flags}")
            } else if v.is_callable() {
                let name = v
                    .get_property(&PropertyKey::from(js_string!("name")))?
                    .and_then(|d| Some(d.value()?.as_string()?.to_std_string_escaped()));
                let is_class = v
                    .downcast_ref::<OrdinaryFunction>()?
                    .is_some_and(|f| f.code.is_class_constructor());

                if is_class {
                    match name {
                        Some(name) if !name.is_empty() => write!(f, "[class {name}]"),
                        _ => f.write_str("[class (anonymous)]"),
                    }
                } else {
                    match name {
                        Some(name) if !name.is_empty() => write!(f, "[Function: {name}]"),
                        _ => f.write_str("[Function (anonymous)]"),
                    }
                }
            } else {
                super::object::log_object_to_internal(f, x, &mut HashSet::new(), 4, print_internals)
            }
        }
        JsVariant::Null => write!(f, "null"),
        JsVariant::Undefined => write!(f, "undefined"),
        JsVariant::Boolean(v) => write!(f, "{v}"),
        JsVariant::Symbol(symbol) => {
            write!(f, "{}", symbol.descriptive_string().to_std_string_escaped())
        }
        JsVariant::String(v) => write!(f, "{:?}", v.to_std_string_escaped()),
        JsVariant::Float64(v) => super::primitives::format_rational(v, f),
        JsVariant::Integer32(v) => write!(f, "{v}"),
        JsVariant::BigInt(num) => write!(f, "{num}n"),
    }
}

fn error_property_text(value: &JsValue) -> crate::JsResult<String> {
    if let Some(string) = value.as_string() {
        return Ok(string.to_std_string_escaped());
    }
    value.display().render()
}

/// Escapes characters in a regex source string so the result is valid inside `/.../` syntax.
///
/// Per `RegExp.prototype.toString` (ES2022 §21.2.5.14):
/// - `/` → `\/`  (avoids prematurely closing the literal)
/// - Line terminators (`\n`, `\r`, U+2028, U+2029) are escaped so the literal
///   does not span lines: `\n` and `\r` become `\\n`/`\\r`, U+2028/U+2029
///   become `\u2028`/`\u2029`.
fn escape_regexp_source(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for ch in source.chars() {
        match ch {
            '/' => out.push_str("\\/"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out
}

/// Formats a [`JsValue`] inline and compactly, collapsing deeply-nested objects.
/// This method is used for printing arguments of a function to the console.
pub(super) fn log_value_compact(
    f: &mut super::RenderSink<'_>,
    x: &JsValue,
    depth: u32,
    print_internals: bool,
    encounters: &mut HashSet<surgeist_js_gc::heap::ObjectId>,
) -> crate::JsResult<()> {
    match x.variant() {
        JsVariant::Object(v) => {
            // Reuse the full formatter for cases that are identical in compact and non-compact modes.
            if v.is::<JsString>() || v.is::<bool>() || v.is::<f64>() {
                log_value_to(f, x, print_internals, false)
            } else if v.is::<Array>() {
                if depth >= COMPACT_DEPTH_LIMIT {
                    f.write_str("[Array]")
                } else {
                    super::array::log_array_compact(f, &v, depth, print_internals, encounters)
                }
            } else if v.is::<UnmappedArguments>() || v.is::<MappedArguments>() {
                f.write_str("[Arguments]")
            } else if v.is::<OrderedMap>() {
                super::map::log_map_compact(f, &v, depth, print_internals, encounters)
            } else if v.is::<OrderedSet>() {
                super::set::log_set_compact(f, &v, depth, print_internals, encounters)
            } else if v.is::<NativeWeakMap>() {
                f.write_str("WeakMap { <items unknown> }")
            } else if v.is::<NativeWeakSet>() {
                f.write_str("WeakSet { <items unknown> }")
            } else if v.is::<crate::builtins::date::Date>()
                || v.is::<crate::builtins::regexp::RegExp>()
            {
                log_value_to(f, x, print_internals, false)
            } else if v.is::<Error>() {
                log_value_to(f, x, print_internals, true)
            } else if let Some(promise) = v.downcast_ref::<Promise>()? {
                f.write_str("Promise { ")?;
                match promise.state() {
                    PromiseState::Pending => f.write_str("<pending>")?,
                    PromiseState::Fulfilled(val) => {
                        log_value_compact(f, &val, depth + 1, print_internals, encounters)?;
                    }
                    PromiseState::Rejected(reason) => {
                        f.write_str("<rejected> ")?;
                        JsError::from_opaque(reason.clone())?.render_to(f.0)?;
                    }
                }
                f.write_str(" }")
            } else if v.is::<TypedArray>() {
                super::typed_array::log_typed_array(
                    f,
                    &v,
                    depth < COMPACT_DEPTH_LIMIT,
                    print_internals,
                )
            } else if v.is_callable() {
                log_value_to(f, x, print_internals, false)
            } else {
                // Plain object
                if depth >= COMPACT_DEPTH_LIMIT {
                    f.write_str("[Object]")
                } else {
                    super::object::log_plain_object_compact(
                        f,
                        &v,
                        depth,
                        print_internals,
                        encounters,
                    )
                }
            }
        }
        // All non-object variants are formatted the same in compact and non-compact modes.
        _ => log_value_to(f, x, print_internals, false),
    }
}

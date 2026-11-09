use std::collections::HashSet;

use surgeist_js_gc::heap::ObjectId;

use crate::{JsObject, builtins::set::ordered_set::OrderedSet};

pub(super) fn log_set_to(
    f: &mut super::RenderSink<'_>,
    v: &JsObject,
    print_internals: bool,
    print_children: bool,
) -> crate::JsResult<()> {
    let set = v
        .downcast_ref::<OrderedSet>()?
        .ok_or(crate::error::ObjectAccessError::WrongNativeBrand)?;
    let size = set.len();

    if size == 0 {
        return f.write_str("Set(0)");
    }

    if !print_children {
        return write!(f, "Set({size})");
    }

    f.write_str("Set { ")?;
    let mut first = true;
    for value in set.iter() {
        if !first {
            f.write_str(", ")?;
        }
        first = false;
        super::value::log_value_to(f, &value, print_internals, false)?;
    }
    f.write_str(" }")
}

pub(super) fn log_set_compact(
    f: &mut super::RenderSink<'_>,
    v: &JsObject,
    depth: u32,
    print_internals: bool,
    encounters: &mut HashSet<ObjectId>,
) -> crate::JsResult<()> {
    let set = v
        .downcast_ref::<OrderedSet>()?
        .ok_or(crate::error::ObjectAccessError::WrongNativeBrand)?;
    let size = set.len();

    if size == 0 {
        return f.write_str("Set(0)");
    }

    if depth >= super::value::COMPACT_DEPTH_LIMIT {
        return write!(f, "Set({size})");
    }

    f.write_str("Set { ")?;
    let mut first = true;
    for value in set.iter() {
        if !first {
            f.write_str(", ")?;
        }
        first = false;
        super::value::log_value_compact(f, &value, depth + 1, print_internals, encounters)?;
    }
    f.write_str(" }")
}

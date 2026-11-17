//! Finite admission policy for one DOM owner and its usage accounting.
//!
//! `Dom::new` takes an explicit `DomLimits`, and `DomLimits::new` requires a
//! `DomNameLimits` policy as its last argument; no hidden or unlimited default exists.
//! `DomLimits` bounds managed nodes, stored string code units, commands per batch, Slot
//! references, raw Slot assignment inputs per batch and retained change history, and its
//! `with_*` constructors add the optional capacities of the other families.
//! `DomNameLimits` bounds retained name bytes, registry entries, operation work and
//! temporary storage. `DomUsage` reports what an owner holds, and `DomResource` names the
//! configured capacity whose admission would be exceeded.
//!
//! Detached nodes count toward the live limits. `docs/canonical-tree.md` and
//! `docs/storage.md` list what each budget counts.

pub use crate::internal::name_limits::DomNameLimits;
pub use crate::internal::store::{DomLimits, DomResource, DomUsage};

//! Mutable authored CSS ownership and immutable, revisioned captures.
//!
//! CSS owns checked syntax, grammar and formatting. This crate owns live identity,
//! ancestry and atomic publication. It performs no cascade, evaluation or loading.
#![forbid(unsafe_code)]

mod counter_operations;
pub mod css;
mod css_adapter;
mod declaration;
mod declaration_budget;
mod declaration_support;
mod font_operations;
mod identity;
mod media_operations;
mod model;
mod named_supports_operations;
mod objects;
mod owner_effect;
mod parser_resources;
mod publication;
mod readonly_extensions;
mod rule_operations;
mod rule_serialization;
mod sheet_operations;
mod store;

pub use declaration::*;
pub use declaration_budget::CssomDeclarationRequestLimits;
pub use declaration_support::*;
pub use font_operations::*;
pub use identity::*;
pub use media_operations::CssomMediaEdit;
pub use model::*;
pub use objects::*;
pub use owner_effect::*;
pub use publication::*;
pub use readonly_extensions::*;
pub use rule_operations::{
    CssomContainerCondition, CssomKeyframeIndex, CssomQueryFacts, CssomQueryMatchError,
    CssomQueryResult, CssomQueryResultRole,
};
pub use rule_serialization::CssomRuleFormatLimits;
pub use sheet_operations::*;
pub use store::*;

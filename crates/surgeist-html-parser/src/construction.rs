//! Real construction records with controlled tokenizer payload transfer.
//!
//! HTML start-tag names and no-namespace attributes retain exact immutable
//! UTF-16 backing, actual vector/shared tickets, and original token text origins.
//! Character insertion merges preceding native text at an explicitly selected
//! location. Retired merge backing remains owned and fully charged until cleanup.
//! Textarea raw backing is independent of native child text; replacement disposes
//! its old backing while retaining initialized retired slots until cleanup.
//! Prologue producers retain comments, recovered PI and doctype fields in the
//! same arena, with controlled implied-html backing and exact whitespace splits.
//! Preexisting immutable element inputs retain their earlier physical exclusion.
//! Managed MathML/SVG names and qualified attributes retain lexical originals
//! and independent adjusted backing/maps; qualified clones use the same registry.
//! Actual node-document bindings, lazy inert template documents, template patch
//! facts and whole-grammar PI maps share this registry. Other resettable form
//! metadata, source reports and public parsing remain later producers. Qualified
//! hosted context, local shadow records and captured attachment effects stay in
//! this owner; they do not implement a live DOM or public parse entry.
//! Input initialization installs source-order attributes, retaining original
//! token/value-attribute ranges and lexical maps for actual conditional writeback.
//! Script facts are retained for a sealed fresh ordinary Disabled execution;
//! preparation ends at step18 and never fabricates a loaded or executed result.

use crate::HtmlParseResource;
use crate::keyed::{Table, slots};
#[cfg(test)]
use crate::parser::control::WalkSite;
use crate::parser::control::{
    ControlIdentity, DocumentStringAllocation, HtmlParseStop, HtmlParseUnsupportedState,
    HtmlParseWorkCategory, OwnedVec, ParseControl, SharedAllocation, VectorCapacity,
};
use crate::parser::{
    buffer::{FrozenUtf16, Utf16Buffer},
    tokenizer::token::{Doctype, HtmlParseTextOrigin, Tag, TagKind, Text, TokenKind},
};
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::Arc;
use std::{mem, sync::Weak};

use surgeist_html::bounded::script::HtmlScriptPreparationResult as DisabledScriptPreparation;

use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlBuilderNode},
    values::DocumentString,
};

mod activity;
mod attribute_view;
mod detail_origins;
mod dialog;
use detail_origins::DetailOrigins;

pub(crate) mod associated;
mod association_log;
mod base;
mod clone;
mod csp;
mod css_color;
mod input_color;
mod input_image;
pub(crate) mod meta;
pub(crate) mod pending_association;
pub(crate) mod positions;
pub(crate) mod style;
#[cfg(test)]
use input_image::append_native_input_value;
mod completion;
mod family_table;
use family_table::FamilyTable;
mod row_bank;
mod row_log;
use row_log::RowLog;
pub(crate) mod context;
mod details;
mod effects;
#[cfg(test)]
pub(crate) use effects::{fault_effect_drain, take_effect_fault};
pub(crate) mod fragment_document;
#[cfg(test)]
pub(crate) use context::ParserDocumentReference;
use context::{DocumentRef, FragmentContextStorage};
pub(crate) mod evidence;
use evidence::ParseEvidence;
mod foreign;
pub(crate) mod history;
mod pi;
mod radio;
mod range;
#[cfg(test)]
pub(crate) use radio::InputGroupAttribute;
mod mutation;
mod nonce;
pub(crate) mod provenance;
pub(crate) mod read;
pub(crate) mod report;
use provenance::{FieldCopy, OriginGroupRef};
#[cfg(test)]
mod no_panic_tests;
#[cfg(test)]
mod prepared_slot_tests;
#[cfg(test)]
mod read_fault_tests;
#[cfg(test)]
mod record_admission_tests;
mod select;
mod svg;
mod template;
mod text_append;
mod tree_root;
pub use completion::{HtmlParseCompletion, HtmlParseDocumentState, HtmlParseReadiness};
use effects::ConstructionEffects;
pub(crate) use foreign::ForeignNamespace;
use select::SelectMetadata;
pub use svg::HtmlParseSvgScriptOrigin;
pub(crate) use svg::SvgScriptState;
pub(crate) use template::{CreationContext, CreationLocation};
#[cfg(test)]
pub(crate) use template::{ParserDocument, TemplateState};
#[cfg(test)]
pub(crate) use tests::native_view::NativeView;

mod cleanup;
mod formatting_creation;
mod input_values;
mod metadata_rows;
mod ordinary_view;
mod script_type;
#[cfg(test)]
mod test_support;
mod text_runs;
mod token_names;
mod token_payload;
use cleanup::{
    cleanup_charge, cleanup_owner, keep_first, payload_write, release_logical, release_parts,
    release_vector, take_payload,
};
pub(crate) use formatting_creation::FormattingCreation;
use formatting_creation::{check_formatting_tag, copy_formatting_text};
#[cfg(test)]
use formatting_creation::{
    check_formatting_text, copy_formatting_tag, discard_formatting_tag, release_formatting_tag,
};
pub(crate) use input_values::input_tag_state;
use input_values::{
    InputSource, ValueSource, copy_input_value, copy_preserved_input_value,
    copy_selected_input_value, input_type_state, validate_initial_input_origins,
};
#[cfg(test)]
use input_values::{append_input_content, find_input_sources, find_input_state};
use metadata_rows::{
    admitted_input_string, find_form_association_state, find_textarea_state, script_metadata,
    set_form_association_state, set_script_metadata, textarea_raw,
};
use ordinary_view::{
    ConstructionView, next_ordinary_view_after_subtree, next_ordinary_view_node,
    ordinary_view_parent,
};
#[cfg(test)]
use ordinary_view::{append_descendant_text, find_output_state};
use script_type::{classify_script, script_attribute};
#[cfg(test)]
pub(crate) use test_support::csp_protocol_error;
#[cfg(test)]
use test_support::{inject_walk_refusal, name_units, test_owning_depth};
#[cfg(test)]
use text_runs::discard_runs;
pub(crate) use text_runs::{
    BodyTextKind, BodyTextRuns, TextRun, partition_body_text, split_first_lf,
    split_leading_ascii_whitespace,
};
use token_payload::{
    check_disposal_text, check_text_source, exact_string_equal, literal_string,
    release_token_attributes,
};
pub(crate) use token_payload::{release_token_kind, validate_token_kind};
mod node_joins;
mod payload_registry;
mod session_error;
mod string_storage;
use node_joins::{Join, NodeJoins, cell_slot, joined_cell};
pub(crate) use payload_registry::{
    FormAssociationState, InputState, OutputState, ScriptMetadata, TextareaState,
};
pub use payload_registry::{HtmlParseFormEvidence, HtmlParseOutputOverride};
use payload_registry::{
    KindList, KindSlots, ParseFacts, PayloadStorage, PreparedSlot, RetainedPayload,
    element_attribute, element_record, row_list,
};
pub(crate) use session_error::{
    ElementInsertionIgnored, SessionClass, SessionError, session_class,
};
use session_error::{
    construction_error, count_set, end_session_on, latch_creation_fault, latch_effect_fault,
    latch_owner_refusal, latch_registry_row, next_cursor, session_fault_invariant, successor,
};
use string_storage::ReleasedStorage;
pub(crate) use string_storage::{FieldRef, RecordString, TextInsertion};
mod attribute_admission;
pub(crate) use attribute_view::ElementAttributes;
use attribute_view::{AttributesIn, UnitsIn, transient_string};
mod form_effects;
mod record_admission;
mod storage;
mod storage_allocation;
mod storage_attributes;
mod storage_holds;
mod text_admission;
use record_admission::{
    NewRecord, ShadowAttachment, associated_document_parent_of, attach_shadow_of, publish_records,
    record_error,
};
use storage_holds::{NAME_INDEX_MIN_ENTRIES, NAME_INDEX_SPARE_ENTRIES, StorageHolds};
mod storage_input;
mod storage_insertion;
mod storage_release;
mod storage_script;
mod storage_strings;
use storage_strings::{FieldString, LITERALS, Literal};
mod storage_textarea;
use storage_textarea::TextareaRefresh;
mod storage_tree;
use form_effects::{FormEffectPass, FormEffectStage, FormEffects};
use storage::SessionStorage;
use storage_attributes::AttributeMergeName;
#[cfg(test)]
use storage_attributes::PreparedAttributeMerge;
use storage_input::InputAttributeInitialization;
use storage_insertion::InsertionCause;
mod frozen_arena;
mod session;
mod session_creation;
mod session_recovery;
mod session_script;
#[cfg(test)]
mod test_readers;
pub(crate) use frozen_arena::FrozenArena;
use frozen_arena::FrozenStorage;
pub(crate) use session::ParserArenaSession;
pub(crate) use session_creation::RootKind;
use session_creation::{CreationProfile, ImpliedHtmlName, OrderedCreation};
#[cfg(test)]
pub(crate) use storage_allocation::PreexistingElement;
#[derive(Clone, Copy, Debug)]
pub(crate) struct ParserNode {
    handle: HtmlBuilderNode,
    // Immutable dense parser-metadata join; only the checked handle authorizes a node.
    index: usize,
}

fn lowercase_token_name(
    control: &mut ParseControl,
    name: &DocumentString,
) -> Result<(), SessionError> {
    for &unit in name.code_units() {
        control.charge_unit_inspection()?;
        if matches!(unit, 0x41..=0x5a) {
            return Err(SessionError::WrongToken);
        }
    }
    Ok(())
}

fn exact_ascii(
    control: &mut ParseControl,
    value: Option<&DocumentString>,
    expected: &str,
) -> Result<bool, SessionError> {
    exact_ascii_units(control, value.map(DocumentString::code_units), expected)
}
/// [`exact_ascii`] over code units, for text that is not held as a `DocumentString`.
fn exact_ascii_units(
    control: &mut ParseControl,
    value: Option<&[u16]>,
    expected: &str,
) -> Result<bool, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let Some(value) = value else { return Ok(false) };
    if value.len() != expected.len() {
        return Ok(false);
    }
    for (&unit, byte) in value.iter().zip(expected.bytes()) {
        control.charge_unit_literal_comparison()?;
        if unit != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
pub(crate) mod tests;

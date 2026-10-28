//! NumberLayout pins for the stepwise protocol types.
//!
//! Hosts charge these sizes. A metered host admits `size_of::<InputCursor>()`
//! and `size_of::<TokenCursor>()` as the fixed cost of a front, and a record
//! bank admits `size_of::<Record>()` for every record it stores. Surgeist's
//! recorded charged amounts (the `scratch=160` of the UTF-16 rows in
//! `surgeist-css`'s characterization table is the size of `InputCursor` plus
//! that crate's source-position checkpoint) therefore change if any of these
//! change.
//!
//! The values were measured on the cursors at the commit that moved them into
//! this crate, before the move, so they record the sizes the hosts have always
//! been charged. They are a declared contract of the protocol, not an
//! incidental property: a later change must not alter them. A change that
//! needs more room has to keep it outside these types.
//!
//! The pins describe 64-bit targets, where `usize` is eight bytes.

#![cfg(target_pointer_width = "64")]

use std::mem::{align_of, size_of};

use surgeist_css_parser::anb::{AnPlusB, AnPlusBError};
use surgeist_css_parser::document::{Component, ListId, Metrics, NodeId};
use surgeist_css_parser::grammar::{AtRule, Declaration, Failed, QualifiedRule, RuleEnd, UnitKind};
use surgeist_css_parser::input::{
    DecodeError, DecodeHints, DecodeLimits, Encoding, EncodingSelection, InputCursor, InputError,
    InputKind, InputRequest, PreprocessError, PreprocessLimits, PreprocessResource, Scalar,
    SourceMap,
};
use surgeist_css_parser::lexer::protocol::{
    NameKind, NumberCursor, NumberKind, NumberLayout, NumberOverflow, NumberPart, NumberStep,
    ProtocolError, RawToken, RawTokenKind, Record, Span, TokenCursor, TokenRequest,
};
use surgeist_css_parser::serialize::Adjacency;
use surgeist_css_parser::stream::{Mark, StreamError};
use surgeist_css_parser::{Offset, Site, Stream, SyntaxError, Token};

/// Asserts the size and alignment of `T`, naming the type on failure.
fn pinned<T>(name: &str, size: usize, align: usize) {
    assert_eq!(size_of::<T>(), size, "size of {name}");
    assert_eq!(align_of::<T>(), align, "alignment of {name}");
}

#[test]
fn the_input_cursor_and_its_vocabulary_keep_their_measured_layouts() {
    pinned::<InputCursor>("InputCursor", 144, 8);
    pinned::<Scalar>("Scalar", 24, 8);
    pinned::<Option<Scalar>>("Option<Scalar>", 24, 8);
    pinned::<InputRequest>("InputRequest", 24, 8);
    pinned::<InputKind>("InputKind", 1, 1);
    pinned::<InputError>("InputError", 1, 1);
}

/// A host that keeps the map inside a payload of its own charges the map's size
/// as a fixed header, so the map is no larger than the vector of its 16-byte
/// entries: it stores nothing else.
#[test]
fn the_source_map_is_no_larger_than_the_vector_of_its_entries() {
    pinned::<SourceMap>("SourceMap", size_of::<Vec<(usize, usize)>>(), 8);
    assert_eq!(size_of::<SourceMap>(), 24);
}

#[test]
fn the_token_cursor_and_its_records_keep_their_measured_layouts() {
    pinned::<TokenCursor>("TokenCursor", 392, 8);
    pinned::<Record>("Record", 88, 8);
    pinned::<RawToken>("RawToken", 88, 8);
    pinned::<RawTokenKind>("RawTokenKind", 72, 8);
    pinned::<TokenRequest>("TokenRequest", 96, 8);
    pinned::<ProtocolError>("ProtocolError", 1, 1);
    pinned::<Span>("Span", 16, 8);
    pinned::<NameKind>("NameKind", 1, 1);
    pinned::<NumberKind>("NumberKind", 1, 1);
}

#[test]
fn the_number_cursor_and_its_steps_keep_their_measured_layouts() {
    pinned::<NumberCursor>("NumberCursor", 40, 8);
    pinned::<NumberLayout>("NumberLayout", 32, 8);
    pinned::<NumberStep>("NumberStep", 32, 8);
    pinned::<NumberPart>("NumberPart", 2, 1);
    pinned::<NumberOverflow>("NumberOverflow", 1, 1);
}

#[test]
fn the_document_handles_keep_their_measured_layouts() {
    pinned::<NodeId>("NodeId", 8, 8);
    pinned::<ListId>("ListId", 8, 8);
    pinned::<Offset>("Offset", 8, 8);
    pinned::<Metrics>("Metrics", 24, 8);
    pinned::<Token<'static>>("Token", 40, 8);
    pinned::<Site<'static>>("Site", 32, 8);
    pinned::<Mark<'static>>("Mark", 64, 8);
    pinned::<Stream<'static>>("Stream", 56, 8);
    pinned::<Component<'static>>("Component", 56, 8);
    pinned::<StreamError<'static>>("StreamError", 72, 8);
    pinned::<SyntaxError<'static, ()>>("SyntaxError", 72, 8);
}

#[test]
fn the_parser_entry_point_types_keep_their_measured_layouts() {
    pinned::<AnPlusB>("AnPlusB", 8, 4);
    pinned::<AnPlusBError<'static>>("AnPlusBError", 72, 8);
    pinned::<RuleEnd>("RuleEnd", 1, 1);
    pinned::<UnitKind>("UnitKind", 1, 1);
    pinned::<Failed<'static, ()>>("Failed", 8, 8);
    pinned::<AtRule<'static>>("AtRule", 200, 8);
    pinned::<QualifiedRule<'static>>("QualifiedRule", 176, 8);
    pinned::<Declaration<'static>>("Declaration", 160, 8);
    pinned::<Adjacency>("Adjacency", 4, 4);
}

#[test]
fn the_decoding_and_preprocessing_types_keep_their_measured_layouts() {
    pinned::<Encoding>("Encoding", 8, 8);
    pinned::<EncodingSelection>("EncodingSelection", 1, 1);
    pinned::<DecodeHints<'static>>("DecodeHints", 24, 8);
    pinned::<DecodeLimits>("DecodeLimits", 16, 8);
    pinned::<DecodeError>("DecodeError", 1, 1);
    pinned::<PreprocessLimits>("PreprocessLimits", 48, 8);
    pinned::<PreprocessResource>("PreprocessResource", 1, 1);
    pinned::<PreprocessError>("PreprocessError", 1, 1);
}

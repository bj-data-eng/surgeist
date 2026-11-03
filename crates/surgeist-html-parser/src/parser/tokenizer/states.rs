// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tokenizer/states.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf. The five PI states follow
// HTML 877d6146487f15b15bc8dd0e3c625063a37b456a, §13.2.5.72–76.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTokenizerEscape {
    Escaped,
    DoubleEscaped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTokenizerIdentifier {
    Public,
    System,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTokenizerRawState {
    Rcdata,
    Rawtext,
    Script,
    ScriptEscaped(HtmlTokenizerEscape),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTokenizerQuote {
    Unquoted,
    Single,
    Double,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTokenizerState {
    Data,
    Plaintext,
    TagOpen,
    EndTagOpen,
    TagName,
    Raw(HtmlTokenizerRawState),
    RawLessThan(HtmlTokenizerRawState),
    RawEndOpen(HtmlTokenizerRawState),
    RawEndName(HtmlTokenizerRawState),
    ScriptEscapeStart,
    ScriptEscapeStartDash,
    ScriptDoubleEscapeStart,
    ScriptDash(HtmlTokenizerEscape),
    ScriptDashDash(HtmlTokenizerEscape),
    ScriptDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValue(HtmlTokenizerQuote),
    AfterAttributeValue,
    SelfClosing,
    BogusComment,
    MarkupDeclaration,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThan,
    CommentLessThanBang,
    CommentLessThanBangDash,
    CommentLessThanBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    AfterDoctypeKeyword(HtmlTokenizerIdentifier),
    BeforeDoctypeIdentifier(HtmlTokenizerIdentifier),
    DoctypeIdentifier(HtmlTokenizerIdentifier, HtmlTokenizerQuote),
    AfterDoctypeIdentifier(HtmlTokenizerIdentifier),
    BetweenDoctypeIdentifiers,
    BogusDoctype,
    Cdata,
    CdataBracket,
    CdataEnd,
    PiOpen,
    PiTarget,
    AfterPiTarget,
    PiData,
    PiQuestionable,
}

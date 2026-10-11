# html5ever supplemental parser contracts

Pinned upstream d7232d746d3112f08926d169591576bd12fe2fdf, captured 2026-10-10. Partial source witnesses, not normative standards. Original complete-file hashes and line ranges identify exact bytes. Retained bodies are unmodified inside source fences. These bounded witnesses were promoted for the authorized HTML ingestion contract on 11 October 2026 UTC. They supply implementation evidence independently of the selected WHATWG HTML language baseline. The existing [html5ever notice and licenses](../licenses/html5ever/NOTICE.md) apply; this wrapper adds only selection and provenance.

## html5ever/src/tokenizer/char_ref/mod.rs

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/html5ever/src/tokenizer/char_ref/mod.rs); complete-file SHA-256 `e753058cb7ab64c63bb13e2156c583bdc60a8468efe6b9c64b8768c6effadf8f`.

Original lines 1–34:

````rust
// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{TokenSink, Tokenizer};
use crate::buffer_queue::BufferQueue;
use crate::data;
use crate::tendril::StrTendril;

use log::debug;
use std::borrow::Cow::{self, Borrowed};
use std::char::from_u32;

//§ tokenizing-character-references
pub(super) struct CharRef {
    /// The resulting character(s)
    pub(super) chars: [char; 2],

    /// How many slots in `chars` are valid?
    pub(super) num_chars: u8,
}

pub(super) enum Status {
    Stuck,
    Progress,
    Done(CharRef),
}

#[derive(Debug)]
````

Original lines 95–228:

````rust
            num_chars: 1,
        })
    }
}

impl CharRefTokenizer {
    pub(super) fn step<Sink: TokenSink>(
        &mut self,
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
    ) -> Status {
        debug!("char ref tokenizer stepping in state {:?}", self.state);
        match self.state {
            State::Begin => self.do_begin(tokenizer, input),
            State::Octothorpe => self.do_octothorpe(tokenizer, input),
            State::Numeric(base) => self.do_numeric(tokenizer, input, base),
            State::NumericSemicolon => self.do_numeric_semicolon(tokenizer, input),
            State::Named => self.do_named(tokenizer, input),
            State::BogusName => self.do_bogus_name(tokenizer, input),
        }
    }

    fn do_begin<Sink: TokenSink>(
        &mut self,
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
    ) -> Status {
        match tokenizer.peek(input) {
            Some('a'..='z' | 'A'..='Z' | '0'..='9') => {
                self.state = State::Named;
                self.name_buf_opt = Some(StrTendril::new());
                Status::Progress
            },
            Some('#') => {
                tokenizer.discard_char(input);
                self.state = State::Octothorpe;
                Status::Progress
            },
            Some(_) => Status::Done(CharRef::EMPTY),
            None => Status::Stuck,
        }
    }

    fn do_octothorpe<Sink: TokenSink>(
        &mut self,
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
    ) -> Status {
        match tokenizer.peek(input) {
            Some(c @ ('x' | 'X')) => {
                tokenizer.discard_char(input);
                self.hex_marker = Some(c);
                self.state = State::Numeric(16);
            },
            Some(_) => {
                self.hex_marker = None;
                self.state = State::Numeric(10);
            },
            None => return Status::Stuck,
        }
        Status::Progress
    }

    fn do_numeric<Sink: TokenSink>(
        &mut self,
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
        base: u32,
    ) -> Status {
        let Some(c) = tokenizer.peek(input) else {
            return Status::Stuck;
        };
        match c.to_digit(base) {
            Some(n) => {
                tokenizer.discard_char(input);
                self.num = self.num.wrapping_mul(base);
                if self.num > 0x10FFFF {
                    // We might overflow, and the character is definitely invalid.
                    // We still parse digits and semicolon, but don't use the result.
                    self.num_too_big = true;
                }
                self.num = self.num.wrapping_add(n);
                self.seen_digit = true;
                Status::Progress
            },

            None if !self.seen_digit => self.unconsume_numeric(tokenizer, input),

            None => {
                self.state = State::NumericSemicolon;
                Status::Progress
            },
        }
    }

    fn do_numeric_semicolon<Sink: TokenSink>(
        &mut self,
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
    ) -> Status {
        match tokenizer.peek(input) {
            Some(';') => tokenizer.discard_char(input),
            Some(_) => tokenizer.emit_error(Borrowed(
                "Semicolon missing after numeric character reference",
            )),
            None => return Status::Stuck,
        };
        self.finish_numeric(tokenizer)
    }

    fn unconsume_numeric<Sink: TokenSink>(
        &mut self,
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
    ) -> Status {
        let mut unconsume = StrTendril::from_char('#');
        if let Some(c) = self.hex_marker {
            unconsume.push_char(c)
        }

        input.push_front(unconsume);
        tokenizer.emit_error(Borrowed("Numeric character reference without digits"));
        Status::Done(CharRef::EMPTY)
    }

    fn finish_numeric<Sink: TokenSink>(&mut self, tokenizer: &Tokenizer<Sink>) -> Status {
        fn conv(n: u32) -> char {
            from_u32(n).expect("invalid char missed by error handling cases")
        }

        let (c, error) = match self.num {
            n if (n > 0x10FFFF) || self.num_too_big => ('\u{fffd}', true),
            0x00 | 0xD800..=0xDFFF => ('\u{fffd}', true),

````

Original lines 300–451:

````rust
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
        end_char: Option<char>,
    ) -> Status {
        match self.name_match {
            None => {
                match end_char {
                    Some(c) if c.is_ascii_alphanumeric() => {
                        // Keep looking for a semicolon, to determine whether
                        // we emit a parse error.
                        self.state = State::BogusName;
                        return Status::Progress;
                    },

                    // Check length because &; is not a parse error.
                    Some(';') if self.name_buf().len() > 1 => self.emit_name_error(tokenizer),

                    _ => (),
                }
                self.unconsume_name(input);
                Status::Done(CharRef::EMPTY)
            },

            Some((c1, c2)) => {
                // We have a complete match, but we may have consumed
                // additional characters into self.name_buf.  Usually
                // at least one, but several in cases like
                //
                //     &not    => match for U+00AC
                //     &noti   => valid prefix for &notin
                //     &notit  => can't continue match

                let name_len = self.name_len;
                assert!(name_len > 0);
                let last_matched = self.name_buf()[name_len - 1..].chars().next().unwrap();

                // There might not be a next character after the match, if
                // we had a full match and then hit EOF.
                let next_after = if name_len == self.name_buf().len() {
                    None
                } else {
                    Some(self.name_buf()[name_len..].chars().next().unwrap())
                };

                // If the character reference was consumed as part of an attribute, and the last
                // character matched is not a U+003B SEMICOLON character (;), and the next input
                // character is either a U+003D EQUALS SIGN character (=) or an ASCII alphanumeric,
                // then, for historical reasons, flush code points consumed as a character
                // reference and switch to the return state.

                let unconsume_all = match (self.is_consumed_in_attribute, last_matched, next_after)
                {
                    (_, ';', _) => false,
                    (true, _, Some('=')) => true,
                    (true, _, Some(c)) if c.is_ascii_alphanumeric() => true,
                    _ => {
                        // 1. If the last character matched is not a U+003B SEMICOLON character
                        //    (;), then this is a missing-semicolon-after-character-reference parse
                        //    error.
                        tokenizer.emit_error(Borrowed(
                            "Character reference does not end with semicolon",
                        ));
                        false
                    },
                };

                if unconsume_all {
                    self.unconsume_name(input);
                    Status::Done(CharRef::EMPTY)
                } else {
                    input.push_front(StrTendril::from_slice(&self.name_buf()[name_len..]));
                    tokenizer.ignore_lf.set(false);
                    Status::Done(CharRef {
                        chars: [from_u32(c1).unwrap(), from_u32(c2).unwrap()],
                        num_chars: if c2 == 0 { 1 } else { 2 },
                    })
                }
            },
        }
    }

    fn do_bogus_name<Sink: TokenSink>(
        &mut self,
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
    ) -> Status {
        // peek + discard skips over newline normalization, therefore making it easier to
        // un-consume
        let Some(c) = tokenizer.peek(input) else {
            return Status::Stuck;
        };
        tokenizer.discard_char(input);
        self.name_buf_mut().push_char(c);
        match c {
            _ if c.is_ascii_alphanumeric() => return Status::Progress,
            ';' => self.emit_name_error(tokenizer),
            _ => (),
        }
        self.unconsume_name(input);
        Status::Done(CharRef::EMPTY)
    }

    pub(super) fn end_of_file<Sink: TokenSink>(
        &mut self,
        tokenizer: &Tokenizer<Sink>,
        input: &BufferQueue,
    ) -> CharRef {
        loop {
            let status = match self.state {
                State::Begin => Status::Done(CharRef::EMPTY),
                State::Numeric(_) if !self.seen_digit => self.unconsume_numeric(tokenizer, input),
                State::Numeric(_) | State::NumericSemicolon => {
                    tokenizer.emit_error(Borrowed("EOF in numeric character reference"));
                    self.finish_numeric(tokenizer)
                },
                State::Named => self.finish_named(tokenizer, input, None),
                State::BogusName => {
                    self.unconsume_name(input);
                    Status::Done(CharRef::EMPTY)
                },
                State::Octothorpe => {
                    input.push_front(StrTendril::from_slice("#"));
                    tokenizer.emit_error(Borrowed("EOF after '#' in character reference"));
                    Status::Done(CharRef::EMPTY)
                },
            };

            match status {
                Status::Done(char_ref) => {
                    return char_ref;
                },
                Status::Stuck => {
                    return CharRef::EMPTY;
                },
                Status::Progress => {},
            }
        }
    }
}
````

## html5ever/src/tree_builder/rules.rs

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/html5ever/src/tree_builder/rules.rs); complete-file SHA-256 `877f811266d9d1098f07892473e583b8218d01c0895c8c2d08e40acaec98aa6d`.

Original lines 1–14:

````rust
// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// The tree builder rules, as a single, enormous nested match expression.

use crate::encoding::extract_a_character_encoding_from_a_meta_element;
use crate::interface::Quirks;
use crate::tokenizer::states::{Rawtext, Rcdata, ScriptData};
````

Original lines 95–140:

````rust
    pub(crate) fn step(&self, mode: InsertionMode, token: Token) -> ProcessResult<Handle> {
        self.debug_step(mode, &token);

        match mode {
            // § The "initial" insertion mode
            // https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
            InsertionMode::Initial => match token {
                Token::Characters(SplitStatus::NotSplit, text) => {
                    ProcessResult::SplitWhitespace(text)
                },
                Token::Characters(SplitStatus::Whitespace, _) => ProcessResult::Done,
                Token::Comment(text) => self.append_comment_to_doc(text),
                token => {
                    if !self.opts.iframe_srcdoc {
                        self.unexpected(&token);
                        self.set_quirks_mode(Quirks);
                    }
                    ProcessResult::Reprocess(InsertionMode::BeforeHtml, token)
                },
            },

            // § The "before html" insertion mode
            // https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
            InsertionMode::BeforeHtml => {
                let anything_else = |token: Token| {
                    self.create_root(vec![]);
                    ProcessResult::Reprocess(InsertionMode::BeforeHead, token)
                };

                match token {
                    Token::Comment(text) => self.append_comment_to_doc(text),

                    Token::Characters(SplitStatus::NotSplit, text) => {
                        ProcessResult::SplitWhitespace(text)
                    },

                    Token::Characters(SplitStatus::Whitespace, _) => ProcessResult::Done,
                    Token::Tag(tag @ tag!(<html>)) => {
                        self.create_root(tag.attrs);
                        self.mode.set(InsertionMode::BeforeHead);
                        ProcessResult::Done
                    },

                    Token::Tag(tag!(</head> | </body> | </html> | </br>)) => anything_else(token),
                    Token::Tag(tag @ tag!(</>)) => self.unexpected(&tag),

````

Original lines 1110–1190:

````rust
                        self.insert_and_pop_element_for(tag);
                        ProcessResult::DoneAckSelfClosing
                    } else {
                        self.foster_parent_in_body(Token::Tag(tag))
                    }
                },

                Token::Tag(tag @ tag!(<form>)) => {
                    self.unexpected(&tag);
                    if !self.in_html_elem_named(local_name!("template"))
                        && self.form_elem.borrow().is_none()
                    {
                        *self.form_elem.borrow_mut() = Some(self.insert_and_pop_element_for(tag));
                    }
                    ProcessResult::Done
                },

                Token::Eof => self.step(InsertionMode::InBody, token),

                token => {
                    self.unexpected(&token);
                    self.foster_parent_in_body(token)
                },
            },

            // § The "in table text" insertion mode
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intabletext
            InsertionMode::InTableText => match token {
                Token::NullCharacter => self.unexpected(&token),

                Token::Characters(split, text) => {
                    self.pending_table_text.borrow_mut().push((split, text));
                    ProcessResult::Done
                },

                token => {
                    let pending = self.pending_table_text.take();
                    let contains_nonspace = pending.iter().any(|&(split, ref text)| match split {
                        SplitStatus::Whitespace => false,
                        SplitStatus::NotWhitespace => true,
                        SplitStatus::NotSplit => any_not_whitespace(text),
                    });

                    if contains_nonspace {
                        self.sink.parse_error(Borrowed("Non-space table text"));
                        for (split, text) in pending.into_iter() {
                            match self.foster_parent_in_body(Token::Characters(split, text)) {
                                ProcessResult::Done => (),
                                _ => panic!("not prepared to handle this!"),
                            }
                        }
                    } else {
                        for (_, text) in pending.into_iter() {
                            self.append_text(text);
                        }
                    }

                    ProcessResult::Reprocess(self.orig_mode.take().unwrap(), token)
                },
            },

            // § The "in caption" insertion mode
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incaption
            InsertionMode::InCaption => match token {
                Token::Tag(
                    tag @ tag!(<caption> | <col> | <colgroup> | <tbody> | <td> | <tfoot> |
                                <th> | <thead> | <tr> | </table> | </caption>),
                ) => {
                    if self.in_scope_named(table_scope, local_name!("caption")) {
                        self.generate_implied_end_tags(cursory_implied_end);
                        self.expect_to_close(local_name!("caption"));
                        self.clear_active_formatting_to_marker();
                        match tag {
                            Tag {
                                kind: EndTag,
                                name: local_name!("caption"),
                                ..
                            } => {
                                self.mode.set(InsertionMode::InTable);
                                ProcessResult::Done
                            },
````

## markup5ever/util/buffer_queue.rs

[Exact source](https://github.com/servo/html5ever/blob/d7232d746d3112f08926d169591576bd12fe2fdf/markup5ever/util/buffer_queue.rs); complete-file SHA-256 `5d0bcdd8b2d0e4aa7c7c881c44ae66258e384a00f3faeadfc4dc7086a1a63f97`.

Original lines 1–20:

````rust
// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The `BufferQueue` struct and helper types.
//!
//! This type is designed for the efficient parsing of string data, especially where many
//! significant characters are from the ascii range 0-63. This includes, for example, important
//! characters in xml/html parsing.
//!
//! Good and predictable performance is achieved by avoiding allocation where possible (a.k.a. zero
//! copy).
//!
//! [`BufferQueue`]: struct.BufferQueue.html

````

Original lines 138–165:

````rust
    pub fn pop_except_from(&self, set: SmallCharSet) -> Option<SetResult> {
        let (result, now_empty) = match self.buffers.borrow_mut().front_mut() {
            None => (None, false),
            Some(buf) => {
                let n = set.nonmember_prefix_len(buf);
                if n > 0 {
                    let out;
                    unsafe {
                        out = buf.unsafe_subtendril(0, n);
                        buf.unsafe_pop_front(n);
                    }
                    (Some(NotFromSet(out)), buf.is_empty())
                } else {
                    let c = buf.pop_front_char().expect("empty buffer in queue");
                    (Some(FromSet(c)), buf.is_empty())
                }
            },
        };

        // Unborrow self for this part.
        if now_empty {
            self.buffers.borrow_mut().pop_front();
        }

        result
    }

    /// Consume bytes matching the pattern, using a custom comparison function `eq`.
````

Original lines 218–255:

````rust
            Some(ref mut buf) => buf.pop_front(consumed_from_last as u32),
        }

        Some(true)
    }

    /// Get the next character if one is available, removing it from the queue.
    ///
    /// This function manages the buffers, removing them as they become empty.
    pub fn next(&self) -> Option<char> {
        let (result, now_empty) = match self.buffers.borrow_mut().front_mut() {
            None => (None, false),
            Some(buf) => {
                let c = buf.pop_front_char().expect("empty buffer in queue");
                (Some(c), buf.is_empty())
            },
        };

        if now_empty {
            self.buffers.borrow_mut().pop_front();
        }

        result
    }

    pub fn replace_with(&self, other: BufferQueue) {
        let _ = mem::replace(&mut *self.buffers.borrow_mut(), other.buffers.take());
    }

    pub fn swap_with(&self, other: &BufferQueue) {
        mem::swap(
            &mut *self.buffers.borrow_mut(),
            &mut *other.buffers.borrow_mut(),
        );
    }

    /// Return a mutable reference to the first tendril in the queue.
    pub fn peek_front_chunk_mut(&self) -> Option<RefMut<'_, StrTendril>> {
````

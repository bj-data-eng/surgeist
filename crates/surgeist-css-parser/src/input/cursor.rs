//! Finite original-input decoding and CSS Syntax preprocessing.
//!
//! A request consumes one original unit or acknowledges one produced scalar.
//! No request scans a token, allocates storage, or substitutes decoded offsets
//! for original input positions. The grammar owner supplies admitted storage.

/// The original input's unit vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputKind {
    /// UTF-8 code units: each unit a host feeds is one byte (at most 255).
    Utf8,
    /// UTF-16 code units, including unpaired surrogates.
    Utf16,
}

/// A preprocessed scalar and its exact original half-open unit interval.
///
/// `start..end` counts the original input's units (bytes for
/// [`InputKind::Utf8`], 16-bit units for [`InputKind::Utf16`]), not scalars
/// and not offsets into any normalized text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Scalar {
    /// The scalar after section 3.3 preprocessing.
    pub value: char,
    /// The first original unit the scalar was decoded from.
    pub start: usize,
    /// One past the last original unit the scalar was decoded from.
    pub end: usize,
}

/// What an [`InputCursor`] needs its host to do next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputRequest {
    /// Read the original unit at `index` and pass it to
    /// [`InputCursor::read`].
    Read {
        /// The position of the unit in the original input.
        index: usize,
    },
    /// Process one already read unit after a preceding lone high surrogate;
    /// call [`InputCursor::process_held`].
    HeldUnit,
    /// Admit storage for this scalar, then call [`InputCursor::emitted`].
    Emit(Scalar),
    /// The original input is exhausted; call [`InputCursor::end`].
    End,
}

/// A protocol violation by the host of an [`InputCursor`].
///
/// Every error leaves the cursor unchanged, so the pending request remains
/// valid and the host may retry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputError {
    /// The call does not answer the pending [`InputRequest`].
    WrongReply,
    /// A unit outside the input kind's vocabulary, or one that cannot
    /// continue the decoder's current sequence.
    WrongUnit,
    /// The UTF-8 input is malformed.
    InvalidUtf8,
    /// An original position does not fit `usize`.
    BoundaryOverflow,
    /// The cursor already completed.
    Complete,
}

#[derive(Clone, Copy, Debug)]
enum Decoder {
    Ready,
    /// Encoding's UTF-8 decoder state: the lead's payload bits, the bytes
    /// still needed and the smallest code point of the sequence's length.
    Utf8 {
        start: usize,
        value: u32,
        left: u8,
        minimum: u32,
    },
    High {
        start: usize,
        value: u16,
    },
}

/// Fixed-register input cursor. Its caller owns all source and emitted storage.
///
/// The cursor decodes one kind of original unit and applies the preprocessing
/// of section 3.3 as scalars are produced: U+000D CARRIAGE RETURN, U+000C FORM
/// FEED and a U+000D U+000A pair become U+000A LINE FEED, and U+0000 NULL and
/// any surrogate code point become U+FFFD REPLACEMENT CHARACTER. Each scalar
/// keeps the interval of original units it came from.
///
/// This is a host-only stepwise protocol: ask [`request`](Self::request),
/// answer it, repeat until [`end`](Self::end) returns `true`.
#[derive(Debug)]
pub struct InputCursor {
    kind: InputKind,
    length: usize,
    position: usize,
    decoder: Decoder,
    cr: Option<Scalar>,
    held_unit: Option<(usize, u16)>,
    held_scalar: Option<Scalar>,
    output: Option<Scalar>,
    ended: bool,
}

impl InputCursor {
    /// Creates a cursor over `length` original units of `kind`.
    #[must_use]
    pub const fn new(kind: InputKind, length: usize) -> Self {
        Self {
            kind,
            length,
            position: 0,
            decoder: Decoder::Ready,
            cr: None,
            held_unit: None,
            held_scalar: None,
            output: None,
            ended: false,
        }
    }

    /// Reads only cached fixed state. Caller admits this header observation.
    #[must_use]
    pub fn request(&self) -> InputRequest {
        if let Some(scalar) = self.output {
            return InputRequest::Emit(scalar);
        }
        if self.held_unit.is_some() {
            return InputRequest::HeldUnit;
        }
        if self.position < self.length {
            return InputRequest::Read {
                index: self.position,
            };
        }
        InputRequest::End
    }

    /// An actual unit read, followed by its fixed decoder transition.
    /// Invalid protocol input is rejected before changing the cursor.
    ///
    /// # Errors
    ///
    /// [`InputError::WrongReply`] unless the pending request is
    /// [`InputRequest::Read`]; [`InputError::WrongUnit`] for a UTF-8 unit
    /// above 255; [`InputError::InvalidUtf8`] for a byte that cannot continue
    /// a well-formed sequence; [`InputError::Complete`] after the end.
    pub fn read(&mut self, unit: u16) -> Result<(), InputError> {
        if self.ended {
            return Err(InputError::Complete);
        }
        if !matches!(self.request(), InputRequest::Read { .. }) {
            return Err(InputError::WrongReply);
        }
        if self.kind == InputKind::Utf8 {
            if unit > 255 {
                return Err(InputError::WrongUnit);
            }
            self.validate_utf8(unit as u8)?;
        }
        let index = self.position;
        let next = index.checked_add(1).ok_or(InputError::BoundaryOverflow)?;
        self.position = next;
        self.decode(index, unit)
    }

    /// Processes the actual cached unit once; it never rereads original input.
    ///
    /// # Errors
    ///
    /// [`InputError::WrongReply`] unless the pending request is
    /// [`InputRequest::HeldUnit`]; [`InputError::Complete`] after the end.
    pub fn process_held(&mut self) -> Result<(), InputError> {
        if self.ended {
            return Err(InputError::Complete);
        }
        if !matches!(self.request(), InputRequest::HeldUnit) {
            return Err(InputError::WrongReply);
        }
        let (index, unit) = self.held_unit.take().ok_or(InputError::WrongReply)?;
        self.decode(index, unit)
    }

    /// Acknowledges actual admitted storage of the requested scalar.
    ///
    /// # Errors
    ///
    /// [`InputError::WrongReply`] unless the pending request is
    /// [`InputRequest::Emit`]; [`InputError::Complete`] after the end.
    pub fn emitted(&mut self) -> Result<(), InputError> {
        if self.ended {
            return Err(InputError::Complete);
        }
        if self.output.is_none() {
            return Err(InputError::WrongReply);
        }
        self.output = self.held_scalar.take();
        Ok(())
    }

    /// One fixed EOF transition. It can publish a pending CR/lone surrogate;
    /// the caller must acknowledge that scalar before final EOF completion.
    ///
    /// Returns `true` once the input is complete and `false` when a final
    /// scalar was published and must be acknowledged first.
    ///
    /// # Errors
    ///
    /// [`InputError::WrongReply`] unless the pending request is
    /// [`InputRequest::End`]; [`InputError::InvalidUtf8`] when the input ends
    /// inside a UTF-8 sequence; [`InputError::Complete`] after the end.
    pub fn end(&mut self) -> Result<bool, InputError> {
        if self.ended {
            return Err(InputError::Complete);
        }
        if self.request() != InputRequest::End {
            return Err(InputError::WrongReply);
        }
        match self.decoder {
            Decoder::Utf8 { .. } => return Err(InputError::InvalidUtf8),
            Decoder::High { start, .. } => {
                self.decoder = Decoder::Ready;
                self.scalar(Scalar {
                    value: '\u{fffd}',
                    start,
                    end: self.position,
                });
                return Ok(false);
            }
            Decoder::Ready => {}
        }
        if let Some(mut scalar) = self.cr.take() {
            scalar.value = '\n';
            self.output = Some(scalar);
            return Ok(false);
        }
        self.ended = true;
        Ok(true)
    }

    /// Encoding (UTF-8 decoder): a lead byte is 00..7F or C2..F4, and each
    /// continuation byte lies within the bounds its lead selected. The bounds
    /// after E0, ED, F0 and F4 reject overlong forms, surrogates and code
    /// points above U+10FFFF at the second byte, the first unit that proves
    /// the sequence malformed.
    fn validate_utf8(&self, byte: u8) -> Result<(), InputError> {
        match self.decoder {
            Decoder::Ready if byte <= 0x7f || (0xc2..=0xf4).contains(&byte) => Ok(()),
            Decoder::Utf8 {
                value,
                left,
                minimum,
                ..
            } => {
                // The second byte of a three-byte sequence still needs two
                // bytes, and of a four-byte sequence three; there `value` is
                // the lead's payload, so 0 and 0xD are E0 and ED, 0 and 4 are
                // F0 and F4. Every other continuation lies in 80..BF.
                let (lower, upper) = match (minimum, left, value) {
                    (0x800, 2, 0x0) => (0xa0, 0xbf),
                    (0x800, 2, 0xd) => (0x80, 0x9f),
                    (0x10000, 3, 0x0) => (0x90, 0xbf),
                    (0x10000, 3, 0x4) => (0x80, 0x8f),
                    _ => (0x80, 0xbf),
                };
                if (lower..=upper).contains(&byte) {
                    Ok(())
                } else {
                    Err(InputError::InvalidUtf8)
                }
            }
            _ => Err(InputError::InvalidUtf8),
        }
    }

    fn decode(&mut self, index: usize, unit: u16) -> Result<(), InputError> {
        let end = index.checked_add(1).ok_or(InputError::BoundaryOverflow)?;
        if self.kind == InputKind::Utf8 {
            let byte = unit as u8;
            match self.decoder {
                Decoder::Ready if byte <= 0x7f => self.scalar(Scalar {
                    value: char::from(byte),
                    start: index,
                    end,
                }),
                Decoder::Ready => {
                    let (value, left, minimum) = if byte <= 0xdf {
                        (u32::from(byte & 0x1f), 1, 0x80)
                    } else if byte <= 0xef {
                        (u32::from(byte & 0x0f), 2, 0x800)
                    } else {
                        (u32::from(byte & 7), 3, 0x10000)
                    };
                    self.decoder = Decoder::Utf8 {
                        start: index,
                        value,
                        left,
                        minimum,
                    };
                }
                Decoder::Utf8 {
                    start,
                    value,
                    left,
                    minimum,
                } => {
                    let value = (value << 6) | u32::from(byte & 0x3f);
                    if left == 1 {
                        self.decoder = Decoder::Ready;
                        self.scalar(Scalar {
                            value: char::from_u32(value).ok_or(InputError::InvalidUtf8)?,
                            start,
                            end,
                        });
                    } else {
                        self.decoder = Decoder::Utf8 {
                            start,
                            value,
                            left: left - 1,
                            minimum,
                        };
                    }
                }
                Decoder::High { .. } => return Err(InputError::WrongUnit),
            }
        } else {
            match self.decoder {
                Decoder::High { start, value } => {
                    self.decoder = Decoder::Ready;
                    if (0xdc00..=0xdfff).contains(&unit) {
                        let value = 0x10000 + ((u32::from(value) - 0xd800) << 10) + u32::from(unit)
                            - 0xdc00;
                        self.scalar(Scalar {
                            value: char::from_u32(value).ok_or(InputError::WrongUnit)?,
                            start,
                            end,
                        });
                    } else {
                        self.held_unit = Some((index, unit));
                        self.scalar(Scalar {
                            value: '\u{fffd}',
                            start,
                            end: index,
                        });
                    }
                }
                Decoder::Ready if (0xd800..=0xdbff).contains(&unit) => {
                    self.decoder = Decoder::High {
                        start: index,
                        value: unit,
                    };
                }
                Decoder::Ready => self.scalar(Scalar {
                    value: char::from_u32(u32::from(unit)).unwrap_or('\u{fffd}'),
                    start: index,
                    end,
                }),
                Decoder::Utf8 { .. } => return Err(InputError::WrongUnit),
            }
        }
        Ok(())
    }

    fn scalar(&mut self, mut scalar: Scalar) {
        if let Some(mut cr) = self.cr.take() {
            cr.value = '\n';
            if scalar.value == '\n' {
                cr.end = scalar.end;
            } else {
                scalar.value = preprocess(scalar.value);
                if scalar.value == '\r' {
                    self.cr = Some(scalar);
                } else {
                    self.held_scalar = Some(scalar);
                }
            }
            self.output = Some(cr);
        } else {
            scalar.value = preprocess(scalar.value);
            if scalar.value == '\r' {
                self.cr = Some(scalar);
            } else {
                self.output = Some(scalar);
            }
        }
    }
}

const fn preprocess(value: char) -> char {
    match value {
        '\0' => '\u{fffd}',
        '\u{c}' => '\n',
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(kind: InputKind, input: &[u16]) -> Vec<Scalar> {
        let mut cursor = InputCursor::new(kind, input.len());
        let mut output = Vec::new();
        loop {
            match cursor.request() {
                InputRequest::Read { index } => cursor.read(input[index]).unwrap(),
                InputRequest::HeldUnit => cursor.process_held().unwrap(),
                InputRequest::Emit(scalar) => {
                    output.push(scalar);
                    cursor.emitted().unwrap();
                }
                InputRequest::End => {
                    if cursor.end().unwrap() {
                        break;
                    }
                }
            }
        }
        output
    }

    #[test]
    fn original_utf8_intervals_and_css_preprocessing() {
        let input = "a\r\n\0\u{c}\ré";
        let bytes: Vec<_> = input.bytes().map(u16::from).collect();
        assert_eq!(
            decode(InputKind::Utf8, &bytes),
            [
                Scalar {
                    value: 'a',
                    start: 0,
                    end: 1
                },
                Scalar {
                    value: '\n',
                    start: 1,
                    end: 3
                },
                Scalar {
                    value: '\u{fffd}',
                    start: 3,
                    end: 4
                },
                Scalar {
                    value: '\n',
                    start: 4,
                    end: 5
                },
                Scalar {
                    value: '\n',
                    start: 5,
                    end: 6
                },
                Scalar {
                    value: 'é',
                    start: 6,
                    end: 8
                },
            ]
        );
    }

    #[test]
    fn utf16_pairs_and_lone_surrogates_preserve_units() {
        assert_eq!(
            decode(
                InputKind::Utf16,
                &[0xd83d, 0xde00, 0xd800, 65, 0xdc00, 0xdfff]
            ),
            [
                Scalar {
                    value: '😀',
                    start: 0,
                    end: 2
                },
                Scalar {
                    value: '\u{fffd}',
                    start: 2,
                    end: 3
                },
                Scalar {
                    value: 'A',
                    start: 3,
                    end: 4
                },
                Scalar {
                    value: '\u{fffd}',
                    start: 4,
                    end: 5
                },
                Scalar {
                    value: '\u{fffd}',
                    start: 5,
                    end: 6
                },
            ]
        );
        assert_eq!(
            decode(InputKind::Utf16, &[0xd800]),
            [Scalar {
                value: '\u{fffd}',
                start: 0,
                end: 1
            }]
        );
    }

    #[test]
    fn consecutive_cr_and_terminal_cr_are_not_lost() {
        assert_eq!(
            decode(InputKind::Utf16, &[13, 13, 10, 13]),
            [
                Scalar {
                    value: '\n',
                    start: 0,
                    end: 1
                },
                Scalar {
                    value: '\n',
                    start: 1,
                    end: 3
                },
                Scalar {
                    value: '\n',
                    start: 3,
                    end: 4
                },
            ]
        );
    }

    #[test]
    fn refused_storage_and_wrong_reply_leave_pending_request() {
        let mut cursor = InputCursor::new(InputKind::Utf16, 2);
        cursor.read(65).unwrap();
        let pending = cursor.request();
        assert_eq!(cursor.read(66), Err(InputError::WrongReply));
        assert_eq!(cursor.request(), pending);
        // No storage acknowledgement means no next read or successful EOF.
        assert_eq!(cursor.end(), Err(InputError::WrongReply));
        assert_eq!(cursor.request(), pending);
        cursor.emitted().unwrap();
        assert_eq!(cursor.request(), InputRequest::Read { index: 1 });
        cursor.read(66).unwrap();
        cursor.emitted().unwrap();
        assert!(cursor.end().unwrap());
        assert_eq!(cursor.end(), Err(InputError::Complete));
    }

    #[test]
    fn invalid_utf8_protocol_unit_is_atomic_and_retryable() {
        let mut cursor = InputCursor::new(InputKind::Utf8, 2);
        assert_eq!(cursor.read(256), Err(InputError::WrongUnit));
        assert_eq!(cursor.request(), InputRequest::Read { index: 0 });
        cursor.read(0xc3).unwrap();
        assert_eq!(cursor.read(0), Err(InputError::InvalidUtf8));
        assert_eq!(cursor.request(), InputRequest::Read { index: 1 });
        cursor.read(0xa9).unwrap();
        assert_eq!(
            cursor.request(),
            InputRequest::Emit(Scalar {
                value: 'é',
                start: 0,
                end: 2
            })
        );
    }
}

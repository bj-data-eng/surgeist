//! Finite original-input decoding and CSS Syntax preprocessing.
//!
//! A request consumes one original unit or acknowledges one produced scalar.
//! No request scans a token, allocates storage, or substitutes decoded offsets
//! for original input positions. The grammar owner supplies admitted storage.

/// The original input's unit vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputKind {
    Utf8,
    Utf16,
}

/// A preprocessed scalar and its exact original half-open unit interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Scalar {
    pub(crate) value: char,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Request {
    Read {
        index: usize,
    },
    /// Process one already read unit after a preceding lone high surrogate.
    HeldUnit,
    Emit(Scalar),
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    WrongReply,
    WrongUnit,
    InvalidUtf8,
    BoundaryOverflow,
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
pub(crate) struct InputCursor {
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
    pub(crate) const fn new(kind: InputKind, length: usize) -> Self {
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
    pub(crate) fn request(&self) -> Request {
        if let Some(scalar) = self.output {
            return Request::Emit(scalar);
        }
        if self.held_unit.is_some() {
            return Request::HeldUnit;
        }
        if self.position < self.length {
            return Request::Read {
                index: self.position,
            };
        }
        Request::End
    }

    /// An actual unit read, followed by its fixed decoder transition.
    /// Invalid protocol input is rejected before changing the cursor.
    pub(crate) fn read(&mut self, unit: u16) -> Result<(), Error> {
        if self.ended {
            return Err(Error::Complete);
        }
        if !matches!(self.request(), Request::Read { .. }) {
            return Err(Error::WrongReply);
        }
        if self.kind == InputKind::Utf8 {
            if unit > 255 {
                return Err(Error::WrongUnit);
            }
            self.validate_utf8(unit as u8)?;
        }
        let index = self.position;
        let next = index.checked_add(1).ok_or(Error::BoundaryOverflow)?;
        self.position = next;
        self.decode(index, unit)
    }

    /// Processes the actual cached unit once; it never rereads original input.
    pub(crate) fn process_held(&mut self) -> Result<(), Error> {
        if self.ended {
            return Err(Error::Complete);
        }
        if !matches!(self.request(), Request::HeldUnit) {
            return Err(Error::WrongReply);
        }
        let (index, unit) = self.held_unit.take().ok_or(Error::WrongReply)?;
        self.decode(index, unit)
    }

    /// Acknowledges actual admitted storage of the requested scalar.
    pub(crate) fn emitted(&mut self) -> Result<(), Error> {
        if self.ended {
            return Err(Error::Complete);
        }
        if self.output.is_none() {
            return Err(Error::WrongReply);
        }
        self.output = self.held_scalar.take();
        Ok(())
    }

    /// One fixed EOF transition. It can publish a pending CR/lone surrogate;
    /// the caller must acknowledge that scalar before final EOF completion.
    pub(crate) fn end(&mut self) -> Result<bool, Error> {
        if self.ended {
            return Err(Error::Complete);
        }
        if self.request() != Request::End {
            return Err(Error::WrongReply);
        }
        match self.decoder {
            Decoder::Utf8 { .. } => return Err(Error::InvalidUtf8),
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
    fn validate_utf8(&self, byte: u8) -> Result<(), Error> {
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
                    Err(Error::InvalidUtf8)
                }
            }
            _ => Err(Error::InvalidUtf8),
        }
    }

    fn decode(&mut self, index: usize, unit: u16) -> Result<(), Error> {
        let end = index.checked_add(1).ok_or(Error::BoundaryOverflow)?;
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
                            value: char::from_u32(value).ok_or(Error::InvalidUtf8)?,
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
                Decoder::High { .. } => return Err(Error::WrongUnit),
            }
        } else {
            match self.decoder {
                Decoder::High { start, value } => {
                    self.decoder = Decoder::Ready;
                    if (0xdc00..=0xdfff).contains(&unit) {
                        let value = 0x10000 + ((u32::from(value) - 0xd800) << 10) + u32::from(unit)
                            - 0xdc00;
                        self.scalar(Scalar {
                            value: char::from_u32(value).ok_or(Error::WrongUnit)?,
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
                Decoder::Utf8 { .. } => return Err(Error::WrongUnit),
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
                Request::Read { index } => cursor.read(input[index]).unwrap(),
                Request::HeldUnit => cursor.process_held().unwrap(),
                Request::Emit(scalar) => {
                    output.push(scalar);
                    cursor.emitted().unwrap();
                }
                Request::End => {
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
        assert_eq!(cursor.read(66), Err(Error::WrongReply));
        assert_eq!(cursor.request(), pending);
        // No storage acknowledgement means no next read or successful EOF.
        assert_eq!(cursor.end(), Err(Error::WrongReply));
        assert_eq!(cursor.request(), pending);
        cursor.emitted().unwrap();
        assert_eq!(cursor.request(), Request::Read { index: 1 });
        cursor.read(66).unwrap();
        cursor.emitted().unwrap();
        assert!(cursor.end().unwrap());
        assert_eq!(cursor.end(), Err(Error::Complete));
    }

    #[test]
    fn invalid_utf8_protocol_unit_is_atomic_and_retryable() {
        let mut cursor = InputCursor::new(InputKind::Utf8, 2);
        assert_eq!(cursor.read(256), Err(Error::WrongUnit));
        assert_eq!(cursor.request(), Request::Read { index: 0 });
        cursor.read(0xc3).unwrap();
        assert_eq!(cursor.read(0), Err(Error::InvalidUtf8));
        assert_eq!(cursor.request(), Request::Read { index: 1 });
        cursor.read(0xa9).unwrap();
        assert_eq!(
            cursor.request(),
            Request::Emit(Scalar {
                value: 'é',
                start: 0,
                end: 2
            })
        );
    }
}

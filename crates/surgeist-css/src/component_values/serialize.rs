use super::*;

struct Emitter {
    css: Option<String>,
    segments: Vec<CssSerializedOriginSegment>,
    bytes: usize,
    max_bytes: usize,
    previous: TokenSerializationType,
    previous_origin: Option<CssValueOrigin>,
    reverse_solidus: bool,
    cdo_prefix: u8,
    last_origin: Option<CssValueOrigin>,
}

impl Emitter {
    fn new(max_bytes: usize, retain_output: bool) -> Self {
        Self {
            css: retain_output.then(String::new),
            segments: Vec::new(),
            bytes: 0,
            max_bytes,
            previous: TokenSerializationType::Nothing,
            previous_origin: None,
            reverse_solidus: false,
            cdo_prefix: 0,
            last_origin: None,
        }
    }

    fn append(
        &mut self,
        text: &str,
        origin: CssSerializedOrigin,
    ) -> Result<(), CssComponentValueError> {
        let end = self.bytes.checked_add(text.len()).ok_or_else(|| {
            CssComponentValueError::new(
                CssComponentValueErrorKind::CapacityOverflow,
                responsible_origin(&origin),
            )
        })?;
        if end > self.max_bytes {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::ByteLimit,
                responsible_origin(&origin),
            ));
        }
        if let Some(css) = &mut self.css {
            css.push_str(text);
            if end > self.bytes {
                self.segments.push(CssSerializedOriginSegment {
                    range: self.bytes..end,
                    origin,
                });
            }
        }
        self.bytes = end;
        Ok(())
    }

    fn token(
        &mut self,
        spelling: &Lexeme,
        kind: TokenSerializationType,
        reverse_solidus: bool,
    ) -> Result<(), CssComponentValueError> {
        if self.reverse_solidus
            && !(kind == TokenSerializationType::WhiteSpace
                && spelling.text.starts_with(['\n', '\r', '\u{000c}']))
        {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::UnserializableBoundary,
                self.previous_origin
                    .clone()
                    .unwrap_or(CssValueOrigin::Programmatic),
            ));
        }
        // The category table handles pairs. CDO also has a three-token hazard:
        // delimiter '<', delimiter '!', then an identifier starting with '--'.
        if self.previous.needs_separator_when_before(kind)
            || (self.cdo_prefix == 2 && spelling.text.starts_with("--"))
        {
            self.append(
                "/**/",
                CssSerializedOrigin::Separator {
                    before: self
                        .previous_origin
                        .clone()
                        .expect("a preceding token requires a separator"),
                    after: spelling.origin.clone(),
                },
            )?;
        }
        self.append(
            &spelling.text,
            CssSerializedOrigin::Token(spelling.origin.clone()),
        )?;
        self.previous = kind;
        self.previous_origin = Some(spelling.origin.clone());
        self.last_origin = Some(spelling.origin.clone());
        self.reverse_solidus = reverse_solidus;
        self.cdo_prefix = if spelling.text.as_ref() == "<" {
            1
        } else if self.cdo_prefix == 1 && spelling.text.as_ref() == "!" {
            2
        } else {
            0
        };
        Ok(())
    }

    fn suffix(&mut self, spelling: &Lexeme) -> Result<(), CssComponentValueError> {
        self.append(
            &spelling.text,
            CssSerializedOrigin::Token(spelling.origin.clone()),
        )?;
        self.last_origin = Some(spelling.origin.clone());
        Ok(())
    }

    fn comment(
        &mut self,
        spelling: &Lexeme,
        implicit_end: Option<&Lexeme>,
    ) -> Result<(), CssComponentValueError> {
        if self.reverse_solidus {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::UnserializableBoundary,
                self.previous_origin
                    .clone()
                    .unwrap_or(CssValueOrigin::Programmatic),
            ));
        }
        self.suffix(spelling)?;
        if let Some(ending) = implicit_end {
            self.suffix(ending)?;
        }
        // A complete comment already separates both of its neighboring tokens.
        self.previous = TokenSerializationType::Nothing;
        self.previous_origin = None;
        self.cdo_prefix = 0;
        Ok(())
    }

    fn values(&mut self, values: &CssComponentValues) -> Result<(), CssComponentValueError> {
        for item in &values.items {
            match &item.data {
                ComponentData::Token(token) => {
                    self.token(
                        &token.spelling,
                        token.serialization_type(),
                        matches!(token.data, TokenData::Delim('\\')),
                    )?;
                    if let Some(ending) = &token.implicit_end {
                        self.suffix(ending)?;
                    }
                }
                ComponentData::Function(function) => {
                    self.token(&function.opening, TokenSerializationType::Function, false)?;
                    self.values(&function.values)?;
                    self.token(&function.closing, TokenSerializationType::Other, false)?;
                }
                ComponentData::Block(block) => {
                    let kind = match block.kind {
                        CssBlockKind::Parenthesis => TokenSerializationType::OpenParen,
                        CssBlockKind::SquareBracket | CssBlockKind::CurlyBracket => {
                            TokenSerializationType::Other
                        }
                    };
                    self.token(&block.opening, kind, false)?;
                    self.values(&block.values)?;
                    self.token(&block.closing, TokenSerializationType::Other, false)?;
                }
                ComponentData::Comment {
                    spelling,
                    implicit_end,
                    ..
                } => self.comment(spelling, implicit_end.as_ref())?,
            }
        }
        Ok(())
    }

    fn finish(&self) -> Result<(), CssComponentValueError> {
        if self.reverse_solidus {
            Err(CssComponentValueError::new(
                CssComponentValueErrorKind::UnserializableBoundary,
                self.previous_origin
                    .clone()
                    .unwrap_or(CssValueOrigin::Programmatic),
            ))
        } else {
            Ok(())
        }
    }
}

fn responsible_origin(origin: &CssSerializedOrigin) -> CssValueOrigin {
    match origin {
        CssSerializedOrigin::Token(origin) => origin.clone(),
        CssSerializedOrigin::Separator { after, .. } => after.clone(),
        CssSerializedOrigin::End(origin) => origin.clone().unwrap_or(CssValueOrigin::Programmatic),
    }
}

pub(super) fn validate(
    values: &CssComponentValues,
    max_bytes: usize,
) -> Result<(), CssComponentValueError> {
    let mut emitter = Emitter::new(max_bytes, false);
    emitter.values(values)?;
    emitter.finish()
}

pub(super) fn serialize(
    values: &CssComponentValues,
    max_bytes: usize,
) -> Result<CssSerializedValue, CssComponentValueError> {
    // Bound the result before allocating the output text or map.
    validate(values, max_bytes)?;
    let mut emitter = Emitter::new(max_bytes, true);
    emitter.values(values)?;
    emitter.finish()?;
    Ok(CssSerializedValue {
        css: emitter
            .css
            .expect("retained serialization requested output"),
        segments: emitter.segments.into_boxed_slice(),
        end: CssSerializedOrigin::End(emitter.last_origin),
    })
}

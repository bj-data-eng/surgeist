use super::*;

struct Emitter {
    css: Option<String>,
    segments: Vec<CssSerializedOriginSegment>,
    bytes: usize,
    max_bytes: usize,
    previous: TokenSerializationType,
    previous_origin: Option<CssValueOrigin>,
    previous_hex_escape: bool,
    reverse_solidus: bool,
    cdo_prefix: u8,
    last_origin: Option<CssValueOrigin>,
    component_paths: Vec<(usize, Vec<usize>)>,
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
            previous_hex_escape: false,
            reverse_solidus: false,
            cdo_prefix: 0,
            last_origin: None,
            component_paths: Vec::new(),
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
        // The table omits CDC after these four categories: its leading '--'
        // would be consumed as part of an identifier, at-keyword, hash, or dimension.
        if self.previous.needs_separator_when_before(kind)
            || (kind == TokenSerializationType::CDC
                && matches!(
                    self.previous,
                    TokenSerializationType::DelimMinus
                        | TokenSerializationType::DelimAt
                        | TokenSerializationType::DelimHash
                        | TokenSerializationType::Number
                ))
            || (self.cdo_prefix == 2 && spelling.text.starts_with("--"))
            || (self.previous_hex_escape && kind == TokenSerializationType::WhiteSpace)
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
        self.previous_hex_escape = ends_with_hex_escape(&spelling.text);
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
        self.previous_hex_escape = ends_with_hex_escape(&spelling.text);
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
        self.previous_hex_escape = false;
        self.cdo_prefix = 0;
        Ok(())
    }

    fn values(
        &mut self,
        values: &CssComponentValues,
        path: &mut Vec<usize>,
    ) -> Result<(), CssComponentValueError> {
        for (index, item) in values.items.iter().enumerate() {
            path.push(index);
            let first_segment = self.segments.len();
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
                    self.values(&function.values, path)?;
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
                    self.values(&block.values, path)?;
                    self.token(&block.closing, TokenSerializationType::Other, false)?;
                }
                ComponentData::Comment {
                    spelling,
                    implicit_end,
                    ..
                } => self.comment(spelling, implicit_end.as_ref())?,
            }
            if self.css.is_some()
                && let Some(segment) = self.segments[first_segment..]
                    .iter()
                    .find(|s| matches!(s.origin, CssSerializedOrigin::Token(_)))
            {
                self.component_paths
                    .push((segment.range.start, path.clone()));
            }
            path.pop();
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

// CSS consumes one whitespace after a hexadecimal escape, even after its sixth
// digit. A separately originating whitespace token must not become that terminator.
fn ends_with_hex_escape(text: &str) -> bool {
    let digits = text
        .bytes()
        .rev()
        .take(6)
        .take_while(u8::is_ascii_hexdigit)
        .count();
    digits > 0
        && text.as_bytes()[..text.len() - digits]
            .iter()
            .rev()
            .take_while(|byte| **byte == b'\\')
            .count()
            % 2
            == 1
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
    emitter.values(values, &mut Vec::new())?;
    emitter.finish()
}

pub(super) fn serialize(
    values: &CssComponentValues,
    max_bytes: usize,
) -> Result<CssSerializedValue, CssComponentValueError> {
    // Bound the result before allocating the output text or map.
    validate(values, max_bytes)?;
    let mut emitter = Emitter::new(max_bytes, true);
    emitter.values(values, &mut Vec::new())?;
    emitter.finish()?;
    Ok(CssSerializedValue {
        css: emitter
            .css
            .expect("retained serialization requested output"),
        segments: emitter.segments.into_boxed_slice(),
        end: CssSerializedOrigin::End(emitter.last_origin),
        component_paths: emitter.component_paths.into_boxed_slice(),
    })
}

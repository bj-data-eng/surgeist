use super::*;

/// Recognized grammar syntax; opaque syntax must use original components instead.
pub(crate) enum CssCanonicalToken<'a> {
    Ident(&'a str),
    Function(&'a str),
    AtKeyword(&'a str),
    Semicolon,
    Whitespace,
    Comma,
    Colon,
    Delim(char),
    OpenParen,
    CloseParen,
}

pub(crate) struct CssCanonicalBuilder {
    emitter: Emitter,
}

impl CssCanonicalBuilder {
    pub(crate) fn new(max_bytes: usize) -> Self {
        Self {
            emitter: Emitter::new(max_bytes, true),
        }
    }

    /// Measures the same joint token stream without retaining output or its map.
    pub(crate) fn counting(max_bytes: usize) -> Self {
        Self {
            emitter: Emitter::new(max_bytes, false),
        }
    }

    pub(crate) fn push_component(
        &mut self,
        component: &CssComponentValue,
    ) -> Result<(), CssComponentValueError> {
        self.emitter.component(component, &mut Vec::new())
    }

    /// Emits borrowed components iteratively; supports lexical views need no
    /// subtree clone or caller-stack recursion to serialize nested enclosures.
    pub(crate) fn push_components(
        &mut self,
        components: &[CssComponentValue],
    ) -> Result<(), CssComponentValueError> {
        enum Event<'a> {
            Component(&'a CssComponentValue),
            Closing(&'a Lexeme),
        }
        let mut pending: Vec<_> = components.iter().rev().map(Event::Component).collect();
        while let Some(event) = pending.pop() {
            let component = match event {
                Event::Closing(value) => {
                    self.emitter
                        .token(value, TokenSerializationType::Other, false)?;
                    continue;
                }
                Event::Component(value) => value,
            };
            match &component.data {
                ComponentData::Function(value) => {
                    self.emitter
                        .token(&value.opening, TokenSerializationType::Function, false)?;
                    pending.push(Event::Closing(&value.closing));
                    pending.extend(value.values.items().iter().rev().map(Event::Component));
                }
                ComponentData::Block(value) => {
                    let kind = if value.kind == CssBlockKind::Parenthesis {
                        TokenSerializationType::OpenParen
                    } else {
                        TokenSerializationType::Other
                    };
                    self.emitter.token(&value.opening, kind, false)?;
                    pending.push(Event::Closing(&value.closing));
                    pending.extend(value.values.items().iter().rev().map(Event::Component));
                }
                _ => self.emitter.component(component, &mut Vec::new())?,
            }
        }
        Ok(())
    }

    pub(crate) fn push_grammar(
        &mut self,
        token: CssCanonicalToken<'_>,
        origin: &CssValueOrigin,
    ) -> Result<(), CssComponentValueError> {
        let (text, kind, reverse_solidus) = match token {
            CssCanonicalToken::Function(name) => {
                CssComponentValue::try_ident(name)
                    .map_err(|error| CssComponentValueError::new(error.kind(), origin.clone()))?;
                (
                    Token::Function(name.into()).to_css_string().into(),
                    TokenSerializationType::Function,
                    false,
                )
            }
            CssCanonicalToken::OpenParen => ("(".into(), TokenSerializationType::OpenParen, false),
            CssCanonicalToken::CloseParen => (")".into(), TokenSerializationType::Other, false),
            other => {
                let component = match other {
                    CssCanonicalToken::Ident(value) => CssComponentValue::try_ident(value),
                    CssCanonicalToken::AtKeyword(value) => {
                        let ident = CssComponentValue::try_ident(value)?;
                        let mut text = String::from("@");
                        let ComponentData::Token(ident) = ident.data else { unreachable!("checked identifier") };
                        text.push_str(&ident.spelling.text);
                        CssComponentValue::try_token(&text)
                    },
                    CssCanonicalToken::Semicolon => CssComponentValue::try_token(";"),
                    CssCanonicalToken::Whitespace => CssComponentValue::try_token(" "),
                    CssCanonicalToken::Comma => CssComponentValue::try_token(","),
                    CssCanonicalToken::Colon => CssComponentValue::try_token(":"),
                    CssCanonicalToken::Delim(character) => {
                        let component = CssComponentValue::try_token(&character.to_string()).map_err(|error| CssComponentValueError::new(error.kind(), origin.clone()))?;
                        if !matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Delim(actual)) if actual == character) {
                            return Err(CssComponentValueError::new(CssComponentValueErrorKind::InvalidToken, origin.clone()));
                        }
                        Ok(component)
                    }
                    CssCanonicalToken::Function(_) | CssCanonicalToken::OpenParen | CssCanonicalToken::CloseParen => unreachable!(),
                }.map_err(|error| CssComponentValueError::new(error.kind(), origin.clone()))?;
                let ComponentData::Token(value) = component.data else {
                    unreachable!("checked grammar token")
                };
                let kind = value.serialization_type();
                let reverse_solidus = matches!(value.data, TokenData::Delim('\\'));
                (value.spelling.text, kind, reverse_solidus)
            }
        };
        self.emitter.token(
            &Lexeme {
                text,
                origin: origin.clone(),
            },
            kind,
            reverse_solidus,
        )
    }

    pub(crate) fn byte_len(&self) -> usize {
        self.emitter.bytes
    }

    pub(crate) fn finish(self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.emitter.finish()?;
        Ok(CssSerializedValue {
            css: self.emitter.css.expect("canonical output is retained"),
            segments: self.emitter.segments.into_boxed_slice(),
            end: CssSerializedOrigin::End(self.emitter.last_origin),
            // Canonical output combines grammar tokens and multiple graphs. Its
            // origins are complete, but it has no single source component path space.
            component_paths: Box::new([]),
        })
    }
}

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
    component_paths: Vec<(Range<usize>, Vec<usize>)>,
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
            self.component(item, path)?;
            path.pop();
        }
        Ok(())
    }

    fn component(
        &mut self,
        item: &CssComponentValue,
        path: &mut Vec<usize>,
    ) -> Result<(), CssComponentValueError> {
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
                .push((segment.range.start..self.bytes, path.clone()));
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

#[cfg(test)]
mod media_helpers_tests {
    use super::*;

    #[test]
    fn subtree_ranges_distinguish_repeated_original_components_and_keep_trivia() {
        let parsed = parse_component_values("future(a/**/b)").unwrap();
        let component = parsed.items()[0].clone();
        let values = CssComponentValues::try_new(vec![component.clone(), component]).unwrap();
        let serialized = values.serialize().unwrap();
        assert_eq!(serialized.as_css(), "future(a/**/b)future(a/**/b)");
        assert_eq!(
            serialized.component_paths_in_range(0..14).unwrap(),
            vec![&[0][..]]
        );
        assert_eq!(
            serialized.component_paths_in_range(14..28).unwrap(),
            vec![&[1][..]]
        );
        assert_eq!(
            serialized.component_paths_in_range(26..27).unwrap(),
            vec![&[1, 2][..]]
        );
        assert_eq!(serialized.component_offset_for_path(&[1, 2]), Some(26));
        assert_eq!(
            serialized.component_paths_in_range(7..13).unwrap(),
            vec![&[0, 0][..], &[0, 1][..], &[0, 2][..]]
        );
        assert!(serialized.component_paths_in_range(7..11).is_none());
        assert!(serialized.component_paths_in_range(7..27).is_none());
        assert_eq!(
            values.component_at_path(&[1, 2]),
            parsed.component_at_path(&[0, 2])
        );
    }

    #[test]
    fn attributed_grammar_uses_token_boundaries_and_original_sources() {
        let number = parse_component_values("1").unwrap();
        let identifier = parse_component_values("e3").unwrap();
        let mut builder = CssCanonicalBuilder::new(7);
        builder.push_component(&number.items()[0]).unwrap();
        builder
            .push_grammar(
                CssCanonicalToken::Ident("e3"),
                identifier.items()[0].origin(),
            )
            .unwrap();
        let serialized = builder.finish().unwrap();
        assert_eq!(serialized.as_css(), "1/**/e3");
        assert_eq!(
            serialized.value_origin_at(1),
            Some(identifier.items()[0].origin())
        );
        assert!(
            matches!(serialized.origin_at(1), Some(CssSerializedOrigin::Separator { before, after }) if before == number.items()[0].origin() && after == identifier.items()[0].origin())
        );
        assert_eq!(
            serialized.value_origin_at(7),
            Some(identifier.items()[0].origin())
        );
        let mut short = CssCanonicalBuilder::new(6);
        short.push_component(&number.items()[0]).unwrap();
        let error = short
            .push_grammar(
                CssCanonicalToken::Ident("e3"),
                identifier.items()[0].origin(),
            )
            .unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
        assert_eq!(error.origin(), identifier.items()[0].origin());
    }

    #[test]
    fn component_limit_causes_and_implicit_origins_stay_distinct() {
        let parsed = parse_component_values("future(1px").unwrap();
        let implicit = parsed.first_implicit_origin().unwrap();
        assert!(matches!(implicit, CssValueOrigin::ImplicitClosure { .. }));
        for (limits, kind) in [
            (
                CssComponentValueLimits::try_new(0, 10, 100).unwrap(),
                CssComponentValueErrorKind::NestingLimit,
            ),
            (
                CssComponentValueLimits::try_new(1, 1, 100).unwrap(),
                CssComponentValueErrorKind::ComponentLimit,
            ),
            (
                CssComponentValueLimits::try_new(1, 2, 10).unwrap(),
                CssComponentValueErrorKind::ByteLimit,
            ),
        ] {
            let error = parsed.validate_with_limits(limits).unwrap_err();
            assert_eq!(error.kind(), kind);
            assert!(!matches!(error.origin(), CssValueOrigin::Programmatic));
        }
        let mut builder = CssCanonicalBuilder::new(100);
        builder.push_component(&parsed.items()[0]).unwrap();
        let serialized = builder.finish().unwrap();
        assert_eq!(serialized.as_css(), "future(1px)");
        assert_eq!(serialized.value_origin_at(10), Some(implicit));
    }
}

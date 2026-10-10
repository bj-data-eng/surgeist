//! Bounded CSS escaping adapters; resource accounting remains context-owned.

use std::fmt;

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssSpecifiedValueSerializationError as Error, CssSpecifiedValueSerializationErrorKind,
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy)]
pub(crate) enum EscapedKind {
    Identifier,
    String,
}

impl EscapedKind {
    fn write(self, value: &str, writer: &mut impl fmt::Write) -> fmt::Result {
        match self {
            Self::Identifier => cssparser::serialize_identifier(value, writer),
            Self::String => cssparser::serialize_string(value, writer),
        }
    }
}

/// Captures bounded scratch text without charging final bytes or skipping
/// suppressed output. Callers decide whether a scratch value is needed.
pub(crate) fn capture_escaped(
    value: &str,
    kind: EscapedKind,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    let mut text = String::new();
    let mut writer = ScratchWriter {
        context,
        text: &mut text,
        error: None,
    };
    if kind.write(value, &mut writer).is_err() {
        return Err(formatting_error(writer.error));
    }
    Ok(text)
}

pub(crate) fn capture_identifier(
    value: &str,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    capture_escaped(value, EscapedKind::Identifier, context)
}

/// Streams directly into the caller's output, honoring context suppression and
/// its existing cumulative or temporary-output accounting.
pub(crate) fn append_string(
    value: &str,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> Result<()> {
    let mut writer = StreamingWriter {
        context,
        output,
        error: None,
    };
    if EscapedKind::String.write(value, &mut writer).is_err() {
        return Err(formatting_error(writer.error));
    }
    Ok(())
}

fn formatting_error(error: Option<Error>) -> Error {
    error.unwrap_or_else(|| Error::new(CssSpecifiedValueSerializationErrorKind::CapacityOverflow))
}

struct ScratchWriter<'a> {
    context: &'a SpecifiedSerializationContext,
    text: &'a mut String,
    error: Option<Error>,
}

impl fmt::Write for ScratchWriter<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.context
            .append_temporary(self.text, text)
            .map_err(|error| {
                self.error = Some(error);
                fmt::Error
            })
    }
}

struct StreamingWriter<'a> {
    context: &'a mut SpecifiedSerializationContext,
    output: &'a mut String,
    error: Option<Error>,
}

impl fmt::Write for StreamingWriter<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.context.append(self.output, text).map_err(|error| {
            self.error = Some(error);
            fmt::Error
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CssSpecifiedValueSerializationErrorKind as Kind,
        CssSpecifiedValueSerializationLimits as Limits,
    };

    #[test]
    fn scratch_escaping_retains_grammar_and_preconsumed_byte_budgets() {
        // CSSOM identifier and string escaping have different leading-digit,
        // control-character and quote rules; these expected bytes are authored.
        for (value, kind, expected) in [
            ("1a", EscapedKind::Identifier, "\\31 a"),
            ("\0", EscapedKind::Identifier, "\u{fffd}"),
            ("a b", EscapedKind::Identifier, "a\\ b"),
            ("\u{1}", EscapedKind::Identifier, "\\1 "),
            ("1a", EscapedKind::String, "\"1a\""),
            ("\n", EscapedKind::String, "\"\\a \""),
            ("\0", EscapedKind::String, "\"\u{fffd}\""),
        ] {
            let mut context =
                SpecifiedSerializationContext::new(Limits::new(0, 0, expected.len() + 1));
            let mut output = String::new();
            context.append(&mut output, "x").unwrap();
            assert_eq!(capture_escaped(value, kind, &context).unwrap(), expected);
            assert_eq!(context.remaining_bytes(), expected.len());
            context.append(&mut output, "y").unwrap();
            assert_eq!(
                capture_escaped(value, kind, &context).unwrap_err().kind(),
                Kind::ByteLimit
            );
        }
    }

    #[test]
    fn suppressed_streaming_skips_bytes_while_scratch_retains_its_bound() {
        let mut context = SpecifiedSerializationContext::new(Limits::new(0, 0, 5));
        let mut output = String::new();
        context.append(&mut output, "x").unwrap();
        assert!(!context.replace_output_suppression(true));
        append_string("long\nstring", &mut context, &mut output).unwrap();
        assert_eq!(output, "x");
        assert_eq!(context.remaining_bytes(), 4);
        assert_eq!(capture_identifier("a b", &context).unwrap(), "a\\ b");
        assert_eq!(
            capture_escaped("ab", EscapedKind::String, &context).unwrap(),
            "\"ab\""
        );
        assert_eq!(
            capture_escaped("abc", EscapedKind::String, &context)
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(
            capture_identifier("a bc", &context).unwrap_err().kind(),
            Kind::ByteLimit
        );
        assert!(context.replace_output_suppression(false));
        append_string("ab", &mut context, &mut output).unwrap();
        assert_eq!(output, "x\"ab\"");
        assert_eq!(context.remaining_bytes(), 0);
    }

    #[test]
    fn streaming_escaping_honors_preconsumed_and_temporary_output_budgets() {
        let mut context = SpecifiedSerializationContext::new(Limits::new(0, 0, 5));
        let mut output = String::new();
        context.append(&mut output, "x").unwrap();
        assert_eq!(
            append_string("abc", &mut context, &mut output)
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );

        let mut context = SpecifiedSerializationContext::new(Limits::new(0, 0, 5));
        let mut output = String::new();
        context.append(&mut output, "x").unwrap();
        assert_eq!(context.replace_temporary_output(Some(0)), None);
        let mut scratch = String::new();
        append_string("ab", &mut context, &mut scratch).unwrap();
        assert_eq!(scratch, "\"ab\"");
        assert_eq!(context.remaining_bytes(), 0);
        assert_eq!(context.replace_temporary_output(None), Some(4));
        context.append(&mut output, &scratch).unwrap();
        assert_eq!(output, "x\"ab\"");
    }
}

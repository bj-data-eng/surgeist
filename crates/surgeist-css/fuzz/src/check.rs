//! Contract checks shared by every target. Expectations come from the documented
//! coordinate, cleanliness and retained-node contracts, not from the parser.
use surgeist_css::{
    CssParseReport,
    diagnostic::CssRecoveryDiagnostic,
    source::{CssSourcePosition, CssSourceSpan},
};

pub(crate) const COORDINATES: &str = "surgeist-css::source-coordinates";
pub(crate) const CLEANLINESS: &str = "surgeist-css::report-cleanliness";

/// One violated property with a stable kebab-case discriminator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    property: &'static str,
    discriminator: &'static str,
    detail: String,
}

impl Violation {
    pub(crate) fn new(
        property: &'static str,
        discriminator: &'static str,
        detail: impl AsRef<str>,
    ) -> Self {
        let mut line: String = detail
            .as_ref()
            .chars()
            .map(|c| if c.is_control() { '?' } else { c })
            .take(1024)
            .collect();
        line = line.trim().to_owned();
        if line.is_empty() {
            line.push_str("violation");
        }
        Self {
            property,
            discriminator,
            detail: line,
        }
    }

    /// The violated property identity.
    #[must_use]
    pub const fn property(&self) -> &'static str {
        self.property
    }

    /// The stable discriminator that, with the property, identifies the failure.
    #[must_use]
    pub const fn discriminator(&self) -> &'static str {
        self.discriminator
    }

    /// Bounded one-line diagnostic text; it takes no part in failure identity.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub(crate) type Check = Result<(), Violation>;

/// Zero-based line and UTF-16 column of a byte offset under the documented
/// decoded-source contract: CR LF, CR, LF and FF each end one line, and the
/// column counts UTF-16 units since the last line end.
pub(crate) fn expected_coordinates(text: &str, offset: usize) -> (u32, u32) {
    let prefix = &text.as_bytes()[..offset];
    let mut line = 0_u32;
    let mut line_start = 0_usize;
    let mut index = 0_usize;
    while index < prefix.len() {
        match prefix[index] {
            b'\r' => {
                line += 1;
                if prefix.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
                line_start = index + 1;
            }
            b'\n' | 0x0c => {
                line += 1;
                line_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    let column = text[line_start..offset].encode_utf16().count();
    (line, u32::try_from(column).unwrap_or(u32::MAX))
}

/// The coordinate model that position checks compare against. Tests replace it
/// to plant a controlled failure; production use is always [`DOCUMENTED`].
#[derive(Clone, Copy)]
pub(crate) struct Checker {
    pub(crate) coordinates: fn(&str, usize) -> (u32, u32),
}

/// The documented decoded-source coordinate contract.
pub(crate) const DOCUMENTED: Checker = Checker {
    coordinates: expected_coordinates,
};

impl Checker {
    pub(crate) fn position(
        &self,
        property: &'static str,
        text: &str,
        position: CssSourcePosition,
        what: &str,
    ) -> Check {
        let offset = position.byte_offset().value();
        if offset > text.len() || !text.is_char_boundary(offset) {
            return Err(Violation::new(
                property,
                "offset-outside-source",
                format!(
                    "{what} offset {offset} is not a boundary of the {}-byte source",
                    text.len()
                ),
            ));
        }
        let (line, column) = (self.coordinates)(text, offset);
        if position.line().value() != line || position.column().value() != column {
            return Err(Violation::new(
                property,
                "line-column-disagree",
                format!(
                    "{what} at offset {offset} reports {}:{} where the source contract gives {line}:{column}",
                    position.line().value(),
                    position.column().value()
                ),
            ));
        }
        Ok(())
    }

    pub(crate) fn span(
        &self,
        property: &'static str,
        text: &str,
        span: CssSourceSpan,
        what: &str,
    ) -> Check {
        self.position(property, text, span.start(), what)?;
        self.position(property, text, span.end(), what)?;
        if span.start().byte_offset() > span.end().byte_offset() {
            return Err(Violation::new(
                property,
                "span-reversed",
                format!("{what} span ends before it starts"),
            ));
        }
        Ok(())
    }

    /// Diagnostics: positions and spans follow the source contract, the
    /// responsible position lies inside the recovery span, and the report orders
    /// by that offset.
    pub(crate) fn diagnostics(&self, text: &str, diagnostics: &[CssRecoveryDiagnostic]) -> Check {
        let mut previous = 0_usize;
        for (index, diagnostic) in diagnostics.iter().enumerate() {
            let error = diagnostic.error();
            let at = error.position();
            self.position(COORDINATES, text, at, "diagnostic position")?;
            self.span(COORDINATES, text, diagnostic.span(), "diagnostic span")?;
            if let Some(before) = error.previous_position() {
                self.position(COORDINATES, text, before, "previous position")?;
            }
            let span = diagnostic.span();
            if at.byte_offset() < span.start().byte_offset()
                || at.byte_offset() > span.end().byte_offset()
            {
                return Err(Violation::new(
                    COORDINATES,
                    "position-outside-recovery-span",
                    format!("diagnostic {index} position lies outside its recovery span"),
                ));
            }
            if at.byte_offset().value() < previous {
                return Err(Violation::new(
                    COORDINATES,
                    "diagnostics-unordered",
                    format!("diagnostic {index} precedes an earlier responsible offset"),
                ));
            }
            previous = at.byte_offset().value();
            if let Some(source) = diagnostic.source()
                && source.as_str() != text
            {
                return Err(Violation::new(
                    COORDINATES,
                    "diagnostic-source-differs",
                    format!("diagnostic {index} retains a snapshot other than the parsed source"),
                ));
            }
        }
        Ok(())
    }
}

/// `is_clean` is exactly an empty diagnostic slice, and validation accepts
/// exactly that report while preserving every diagnostic in order.
pub(crate) fn cleanliness<T: Clone>(report: &CssParseReport<T>) -> Check {
    let clean = report.is_clean();
    if clean != report.diagnostics().is_empty() {
        return Err(Violation::new(
            CLEANLINESS,
            "clean-disagrees-with-diagnostics",
            format!(
                "is_clean is {clean} with {} diagnostics",
                report.diagnostics().len()
            ),
        ));
    }
    match report.clone().into_validation_result() {
        Ok(_) if !clean => Err(Violation::new(
            CLEANLINESS,
            "validation-accepted-diagnostics",
            "validation accepted a report that has diagnostics",
        )),
        Err(_) if clean => Err(Violation::new(
            CLEANLINESS,
            "validation-rejected-clean",
            "validation rejected a clean report",
        )),
        Err(failure) => {
            let kept = failure.diagnostics() == report.diagnostics()
                && failure.first() == &report.diagnostics()[0]
                && failure.clone().into_diagnostics() == report.diagnostics();
            if kept {
                Ok(())
            } else {
                Err(Violation::new(
                    CLEANLINESS,
                    "validation-lost-diagnostics",
                    "the validation failure differs from the report diagnostics",
                ))
            }
        }
        Ok(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinates_follow_the_documented_line_and_utf16_column_contract() {
        let text = "a\r\nb\rc\u{c}d\n\u{1f600}x";
        let at = |needle: &str| text.find(needle).unwrap();
        assert_eq!(expected_coordinates(text, 0), (0, 0));
        assert_eq!(expected_coordinates(text, at("b")), (1, 0));
        assert_eq!(expected_coordinates(text, at("c")), (2, 0));
        assert_eq!(expected_coordinates(text, at("d")), (3, 0));
        assert_eq!(expected_coordinates(text, at("\u{1f600}")), (4, 0));
        assert_eq!(expected_coordinates(text, at("x")), (4, 2));
        // Between the CR and LF of one CRLF the break is already counted.
        assert_eq!(expected_coordinates(text, 2), (1, 0));
        assert_eq!(expected_coordinates("\u{ffff}\u{10000}z", 7), (0, 3));
    }

    #[test]
    fn violation_detail_is_one_bounded_line() {
        let long = "x".repeat(5000);
        assert_eq!(
            Violation::new("surgeist-css::p", "d", &long).detail().len(),
            1024
        );
        assert_eq!(
            Violation::new("surgeist-css::p", "d", "a\nb\0c ").detail(),
            "a?b?c"
        );
        assert_eq!(
            Violation::new("surgeist-css::p", "d", " ").detail(),
            "violation"
        );
    }
}

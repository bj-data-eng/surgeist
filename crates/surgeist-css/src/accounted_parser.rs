//! The parse fronts of [`CssParser`], charged to a host's ledger scope.

use std::sync::Arc;

use crate::accounting::{
    CssAccount, CssAccountingError, CssChange, CssFront, CssHostScope, CssWork,
};
use crate::component::{CssComponentValue, CssComponentValueError, CssComponentValues};
use crate::declaration::{CssDeclaration, CssDeclarationList, CssImportance};
use crate::parse_work;
use crate::property::CssPropertyTextTarget;
use crate::rule::CssSheet;
use crate::rule::font_face::CssFontFaceDescriptors;
use crate::rule::page::CssPageDeclarationBlock;
use crate::source::utf16_input::parse_utf16_counted;
use crate::source::{
    CssDeclarationBlockParseReport, CssDecodedStylesheetInput, CssSourceSnapshot,
    CssUtf16InputError, CssUtf16InputLimits,
};
use crate::{CssParseReport, CssParser};

/// The parse fronts of a [`CssParser`], charged below a host's scope.
///
/// [`CssParser::accounted`] makes one. Each front parses exactly as the front
/// of the same name on [`CssParser`] does, and returns the same report. When
/// the call returns it charges the host's operation once:
///
/// - to the `css-input` scope, [`CssWork::InputUnit`]: the length of the
///   input the front received;
/// - to the `css-parse` scope, [`CssWork::Document`], [`CssWork::Component`]
///   and [`CssWork::SpellingByte`]: the token documents the call built and
///   the sums of their components and spelling bytes. A front builds more
///   than one document: a sheet builds one for its structure and one for the
///   value of each declaration it reads.
///
/// The call runs inside one phase of the `css-parse` scope. The number of
/// ledger calls does not depend on the input.
///
/// A UTF-16 front prepares its input before it parses. It also charges the
/// `css-input` scope [`CssWork::PreparationStep`], the steps the preparation
/// counted against its work limit, and its input units are the UTF-16 units
/// of the original. The whole call runs inside a phase of the `css-input`
/// scope, and the grammar inside a phase of the `css-parse` scope within it.
/// A call that the preparation refuses, or that the caller's poll interrupts,
/// is charged the steps it took, the refused step included.
///
/// The component limits and every other request limit of the front are
/// checked as they are without a host, and a limit the input exceeds is
/// reported in the report as it is without a host. A document whose build a
/// limit refuses adds nothing to the charge.
///
/// # Errors
///
/// Each front returns `Ok` with exactly what the plain front returns. Where
/// that is itself a `Result`, as for the UTF-16 fronts, its error is the
/// front's own refusal and is returned inside `Ok`.
///
/// Each front returns a [`CssAccountingError`] in place of its result when
/// the host's operation does not admit the charge.
/// [`CssAccountingError::host_refusal`] is `Some` when a cap of the host's
/// scope, or of a scope above it, refused it. The parse scope is charged
/// before the input scope, so a refusal of the second charge leaves the
/// first admitted. A front also returns the error when the ledger cannot
/// open a phase, which only a defect of the host's operation causes; the
/// grammar then does not run.
///
/// ```
/// use surgeist_css::accounting::{CssFront, CssHostScope, CssLimits, CssWork};
/// use surgeist_css::CssParser;
/// use surgeist_ledger::{Limits, Operation};
///
/// let limits: CssLimits = Limits::unlimited();
/// let operation = Operation::new("host", limits, 1 + CssFront::ALL.len())?.share();
/// let host = CssHostScope::open(operation.clone(), operation.root())?;
///
/// let source = "a { color: red }";
/// let report = CssParser::default().accounted(&host).parse_sheet(source)?;
/// assert_eq!(report, CssParser::default().parse_sheet(source));
///
/// let usage = operation.usage(operation.root())?;
/// assert_eq!(usage.work(CssWork::InputUnit), 16);
/// // The structure of the sheet, and the value of its one declaration.
/// assert_eq!(usage.work(CssWork::Document), 2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CssAccountedParser<'host> {
    parser: CssParser,
    account: CssAccount<'host>,
}

impl CssParser {
    /// The fronts of this parser, charged below `host`.
    #[must_use]
    pub const fn accounted<'host>(&self, host: &'host CssHostScope) -> CssAccountedParser<'host> {
        CssAccountedParser {
            parser: *self,
            account: CssAccount::host(host),
        }
    }
}

impl CssAccountedParser<'_> {
    /// [`CssParser::parse_sheet`], charged to the host. The input units are
    /// the bytes of `source`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge.
    pub fn parse_sheet(
        &self,
        source: impl AsRef<str>,
    ) -> Result<CssParseReport<CssSheet>, CssAccountingError> {
        let source = source.as_ref();
        self.charged(source.len(), || self.parser.parse_sheet(source))
    }

    /// [`CssParser::parse_sheet_bytes`], charged to the host. The input units
    /// are the original bytes of `input`. Decoding them was
    /// [`CssDecodedStylesheetInput::decode`], which is not charged.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge.
    pub fn parse_sheet_bytes(
        &self,
        input: &CssDecodedStylesheetInput,
    ) -> Result<CssParseReport<CssSheet>, CssAccountingError> {
        self.charged(input.original_bytes().len(), || {
            self.parser.parse_sheet_bytes(input)
        })
    }

    /// [`CssParser::parse_block_contents`], charged to the host. The input
    /// units are the bytes of `source`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge.
    pub fn parse_block_contents(
        &self,
        source: impl AsRef<str>,
    ) -> Result<CssParseReport<CssDeclarationList>, CssAccountingError> {
        let source = source.as_ref();
        self.charged(source.len(), || self.parser.parse_block_contents(source))
    }

    /// [`CssParser::parse_page_block_contents`], charged to the host. The
    /// input units are the bytes of `source`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge.
    pub fn parse_page_block_contents(
        &self,
        source: impl AsRef<str>,
    ) -> Result<CssParseReport<Option<CssPageDeclarationBlock>>, CssAccountingError> {
        let source = source.as_ref();
        self.charged(source.len(), || {
            self.parser.parse_page_block_contents(source)
        })
    }

    /// [`CssParser::parse_font_face_block_contents`], charged to the host.
    /// The input units are the bytes of `source`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge.
    pub fn parse_font_face_block_contents(
        &self,
        source: impl AsRef<str>,
    ) -> Result<CssParseReport<Option<CssFontFaceDescriptors>>, CssAccountingError> {
        let source = source.as_ref();
        self.charged(source.len(), || {
            self.parser.parse_font_face_block_contents(source)
        })
    }

    /// [`CssParser::parse_property_value`], charged to the host. The input
    /// units are the bytes of `source`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge.
    pub fn parse_property_value(
        &self,
        source: impl AsRef<str>,
        target: CssPropertyTextTarget<'_>,
        importance: CssImportance,
    ) -> Result<CssParseReport<Option<CssDeclaration>>, CssAccountingError> {
        let source = source.as_ref();
        self.charged(source.len(), || {
            self.parser.parse_property_value(source, target, importance)
        })
    }

    /// [`CssComponentValue::parse`], charged to the host. The input units
    /// are the bytes of `source`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge.
    pub fn parse_component_value(
        &self,
        source: impl AsRef<str>,
    ) -> Result<CssParseReport<Option<CssComponentValue>>, CssAccountingError> {
        let source = source.as_ref();
        self.charged(source.len(), || {
            CssComponentValue::parse(&self.parser, source)
        })
    }

    /// [`CssComponentValues::parse`], charged to the host. The input units
    /// are the bytes of `source`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The front's own [`CssComponentValueError`] is returned inside
    /// `Ok`.
    pub fn parse_component_values(
        &self,
        source: impl AsRef<str>,
    ) -> Result<Result<CssComponentValues, CssComponentValueError>, CssAccountingError> {
        let source = source.as_ref();
        self.charged(source.len(), || {
            CssComponentValues::parse(&self.parser, source)
        })
    }

    /// [`CssComponentValues::parse_comma_separated`], charged to the host.
    /// The input units are the bytes of `source`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge.
    pub fn parse_comma_separated_component_values(
        &self,
        source: impl AsRef<str>,
    ) -> Result<CssParseReport<Vec<Option<CssComponentValues>>>, CssAccountingError> {
        let source = source.as_ref();
        self.charged(source.len(), || {
            CssComponentValues::parse_comma_separated(&self.parser, source)
        })
    }

    /// [`CssParser::parse_sheet_utf16`], charged to the host. The input
    /// units are the UTF-16 units of `original`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The preparation refusal or the caller's interruption is
    /// returned inside `Ok`.
    pub fn parse_sheet_utf16<E>(
        &self,
        original: Arc<Vec<u16>>,
        limits: CssUtf16InputLimits,
        poll: impl FnMut() -> Result<(), E>,
    ) -> Result<Result<CssParseReport<CssSheet>, CssUtf16InputError<E>>, CssAccountingError> {
        let parser = self.parser;
        self.charged_utf16(original, limits, poll, |source| {
            parser.sheet_of_snapshot(source)
        })
    }

    /// [`CssParser::parse_block_contents_utf16`], charged to the host. The
    /// input units are the UTF-16 units of `original`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingError`] when the host's operation does not admit the
    /// charge. The preparation refusal or the caller's interruption is
    /// returned inside `Ok`.
    pub fn parse_block_contents_utf16<E>(
        &self,
        original: Arc<Vec<u16>>,
        input_limits: CssUtf16InputLimits,
        poll: impl FnMut() -> Result<(), E>,
    ) -> Result<Result<CssDeclarationBlockParseReport, CssUtf16InputError<E>>, CssAccountingError>
    {
        let parser = self.parser;
        self.charged_utf16(original, input_limits, poll, |source| {
            parser.block_contents_of_snapshot(source)
        })
    }

    /// Prepares `original` inside a phase of the input scope and runs `parse`
    /// over the prepared source inside a phase of the parse scope, collecting
    /// the documents it builds. Then charges the documents, the units of
    /// `original` and the preparation steps, whether the call succeeded or
    /// not.
    fn charged_utf16<E, T>(
        &self,
        original: Arc<Vec<u16>>,
        limits: CssUtf16InputLimits,
        poll: impl FnMut() -> Result<(), E>,
        parse: impl FnOnce(&CssSourceSnapshot) -> T,
    ) -> Result<Result<T, CssUtf16InputError<E>>, CssAccountingError> {
        let account = self.account;
        let units = original.len();
        let ((result, steps), work) = account.timed(CssFront::Input, || {
            parse_work::collect(|| {
                parse_utf16_counted(original, limits, poll, |source| {
                    account.timed(CssFront::Parse, || parse(source))
                })
            })
        })?;
        if let Some(work) = work {
            account.charge(CssFront::Parse, &work.changes())?;
            account.charge(
                CssFront::Input,
                &[
                    CssChange::Work(CssWork::InputUnit, parse_work::amount(units)),
                    CssChange::Work(CssWork::PreparationStep, parse_work::amount(steps)),
                ],
            )?;
        }
        match result {
            Ok(Ok(value)) => Ok(Ok(value)),
            // The parse phase could not be opened, so the grammar did not run.
            Ok(Err(fault)) => Err(fault.into()),
            Err(refusal) => Ok(Err(refusal)),
        }
    }

    /// Runs `parse` inside a phase of the parse scope, collecting the
    /// documents it builds, then charges them and `input_units`.
    fn charged<T>(
        &self,
        input_units: usize,
        parse: impl FnOnce() -> T,
    ) -> Result<T, CssAccountingError> {
        let (value, work) = self
            .account
            .timed(CssFront::Parse, || parse_work::collect(parse))?;
        // An accounted parse call that runs inside another one adds to that
        // call's record, and that call charges. No front calls an accounted
        // front, so nothing reaches this today; a front that came to do so
        // would have to decide who charges the inner call's input units.
        if let Some(work) = work {
            self.account.charge(CssFront::Parse, &work.changes())?;
            self.account.charge(
                CssFront::Input,
                &[CssChange::Work(
                    CssWork::InputUnit,
                    parse_work::amount(input_units),
                )],
            )?;
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests;

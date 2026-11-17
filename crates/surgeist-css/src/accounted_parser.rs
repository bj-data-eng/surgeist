//! The parse fronts of [`CssParser`], charged to a host's ledger scope.

use crate::accounting::{
    CssAccount, CssAccountingError, CssChange, CssFront, CssHostScope, CssWork,
};
use crate::declaration::CssDeclarationList;
use crate::parse_work;
use crate::rule::CssSheet;
use crate::source::CssDecodedStylesheetInput;
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
/// The component limits and every other request limit of the front are
/// checked as they are without a host, and a limit the input exceeds is
/// reported in the report as it is without a host. A document whose build a
/// limit refuses adds nothing to the charge.
///
/// # Errors
///
/// Each front returns a [`CssAccountingError`] in place of its report when
/// the host's operation does not admit the charge.
/// [`CssAccountingError::host_refusal`] is `Some` when a cap of the host's
/// scope, or of a scope above it, refused it. The parse scope is charged
/// before the input scope, so a refusal of the second charge leaves the
/// first admitted.
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
        // call's record, and that call charges.
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

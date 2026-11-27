//! Reveal the existing dyadic builder's exact finite decimal coefficient.

use super::super::HtmlFloatingPointNumber;
use super::{
    HtmlRangeNumericError, HtmlRangeNumericHeaders, HtmlRangeNumericLayout, HtmlRangeNumericReply,
    HtmlRangeNumericRequest, HtmlRangeScratchRole, RangeDyadicCursor, finite_dyadic,
};

/// Binary64 expansion into the caller's Coefficient bank with exact digits.
#[derive(Clone, Copy, Debug)]
pub struct HtmlFiniteDecimalExpansionCursor {
    cursor: RangeDyadicCursor,
    negative: bool,
}

impl HtmlFiniteDecimalExpansionCursor {
    /// Start with a finite number and the caller's admitted banks.
    pub fn new(
        number: HtmlFloatingPointNumber,
        headers: HtmlRangeNumericHeaders,
    ) -> Result<Self, HtmlRangeNumericError> {
        let (mantissa, exponent) = finite_dyadic(number.value().to_bits())?;
        Ok(Self {
            cursor: RangeDyadicCursor::new(
                headers,
                HtmlRangeScratchRole::Coefficient,
                mantissa,
                exponent,
            )?,
            negative: number.value().is_sign_negative(),
        })
    }

    /// The current fixed request, or none after the coefficient is ready.
    pub fn request(&self) -> Option<HtmlRangeNumericRequest> {
        self.cursor.request()
    }

    /// The qualified extents of the current caller-owned banks.
    pub fn headers(&self) -> HtmlRangeNumericHeaders {
        self.cursor.headers()
    }

    /// The sign and fractional places of the finished coefficient.
    pub fn result(&self) -> Option<(bool, usize)> {
        let places = self.cursor.result()?.checked_neg()?;
        Some((self.negative, usize::try_from(places).ok()?))
    }

    /// Apply the matching admitted action, preserving this cursor on error.
    pub fn reply(&mut self, reply: HtmlRangeNumericReply) -> Result<(), HtmlRangeNumericError> {
        let mut next = *self;
        next.cursor.accept_in_place(reply)?;
        *self = next;
        Ok(())
    }

    /// Fixed state and message sizes needed by the caller.
    pub const fn layout() -> HtmlRangeNumericLayout {
        HtmlRangeNumericLayout {
            registers: size_of::<Self>(),
        }
    }
}

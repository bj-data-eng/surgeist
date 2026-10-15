//! Intrinsic SVG script at-most-once state, separate from HTML preparation.
//! SVG Tiny 1.2 §15.2.1 supplies the processed flag. SOURCE-EDITIONS.md's selected
//! WebKit clone determination copies its actual true or false value.

/// The actual SVG script processed flag, without parser or execution history.
///
/// This value contains no origin, node/document identity, connectivity, loader,
/// execution or notification capability. Its existence does not establish the
/// conditions needed to process a script. Missing retained evidence must remain
/// outside this type and must not be replaced by a fresh false value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgScriptFlags {
    already_processed: bool,
}

impl SvgScriptFlags {
    /// A newly created SVG script has not yet been processed. This establishes
    /// no parser insertion history or processing eligibility.
    pub const fn fresh() -> Self {
        Self {
            already_processed: false,
        }
    }

    /// Retains an actual qualified flag supplied by its owning adapter.
    /// This neither qualifies an external fact nor substitutes for unknown state.
    pub const fn from_retained(already_processed: bool) -> Self {
        Self { already_processed }
    }

    pub const fn already_processed(self) -> bool {
        self.already_processed
    }

    /// Performs the intrinsic flag write of processing step 3. The adapter must
    /// first fulfill the source guards, environment qualification and admission;
    /// this method performs no fetching, execution or event notification.
    pub fn mark_processed(&mut self) {
        self.already_processed = true;
    }

    /// The selected SVG clone rule preserves the actual at-most-once bit.
    /// A false source produces false, a true source produces true. Clone identity
    /// and non-parser insertion history remain with the consuming owner.
    pub const fn clone_for_element(self) -> Self {
        self
    }
}

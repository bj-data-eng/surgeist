//! Native visible-control presentation, independent of original host editing.

use icu_properties::props::{BidiClass, GeneralCategory, GraphemeClusterBreak, Script};
use parley_data::Properties;

pub(crate) mod font;
#[cfg(test)]
mod integration_tests;

/// Phase I has already consumed genuine document segment-break occurrences.
/// TAB/LF/CR keep their own whitespace operations; every other Cc is a symbol.
pub(crate) const fn is_visible(code_point: u32) -> bool {
    matches!(code_point, 0..=8 | 11..=12 | 14..=31 | 127..=159)
}

/// ICU segmentation input only. Original/provider text retains the actual Cc.
/// DIAMETER SIGN supplies the selected Other/Other/Other/AL/N segmentation
/// profile, rather than a control boundary or a rendering replacement.
pub(crate) const SEGMENTATION_SYMBOL: char = '\u{2300}';

pub(crate) fn properties() -> Properties {
    Properties::new(
        Script::Common,
        GeneralCategory::OtherSymbol,
        GraphemeClusterBreak::Other,
        BidiClass::OtherNeutral,
        false,
        false,
        false,
        false,
    )
}

#[cfg(test)]
mod tests {
    use icu_properties::{
        CodePointMapData,
        props::{EastAsianWidth, LineBreak, SentenceBreak, WordBreak},
    };

    use super::*;

    #[test]
    fn segmentation_symbol_has_the_selected_native_profile() {
        let symbol = SEGMENTATION_SYMBOL;
        assert_eq!(
            CodePointMapData::<WordBreak>::new().get(symbol),
            WordBreak::Other
        );
        assert_eq!(
            CodePointMapData::<SentenceBreak>::new().get(symbol),
            SentenceBreak::Other
        );
        assert_eq!(
            CodePointMapData::<LineBreak>::new().get(symbol),
            LineBreak::Alphabetic
        );
        assert_eq!(
            CodePointMapData::<EastAsianWidth>::new().get(symbol),
            EastAsianWidth::Neutral
        );
        assert_eq!(
            CodePointMapData::<GraphemeClusterBreak>::new().get(symbol),
            GraphemeClusterBreak::Other
        );
    }
}

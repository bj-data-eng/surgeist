//! Real direction-resolved Text captures, Forced separation and NoWrap.
use super::*;
#[test]
fn pretty_real_text_ltr_rtl_captures_keep_forced_and_nowrap_authority_f32() {
    avoid_short_last_line::with_style::<f32>(l::TextWrapStyle::Pretty);
}
#[test]
fn pretty_real_text_ltr_rtl_captures_keep_forced_and_nowrap_authority_f64() {
    avoid_short_last_line::with_style::<f64>(l::TextWrapStyle::Pretty);
}

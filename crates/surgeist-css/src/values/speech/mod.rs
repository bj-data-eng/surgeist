//! CSS Speech values.
//!
//! CSS Speech values: `speak`, `speak-as`, speech breaks, cues, and the voice properties
//! (family, stress, balance, rate, pitch, range, volume and duration). They retain authored
//! syntax; no speech is synthesized.

pub(crate) mod values;

#[doc(inline)]
pub use values::{
    CssAudioCue, CssCue, CssDecibelLiteral, CssGenericVoice, CssSemitoneLiteral, CssSpeak,
    CssSpeakAs, CssSpeakAsPunctuation, CssSpeechBreak, CssSpeechBreakStrength, CssVoiceAge,
    CssVoiceBalance, CssVoiceBalanceKeyword, CssVoiceDuration, CssVoiceFamily, CssVoiceFamilyEntry,
    CssVoiceFamilyList, CssVoiceFamilyName, CssVoiceFamilyNameRef, CssVoiceGender, CssVoiceLevel,
    CssVoiceOffset, CssVoicePitchRange, CssVoiceRate, CssVoiceRateKeyword, CssVoiceStress,
    CssVoiceVolume, CssVoiceVolumeLevel,
};
